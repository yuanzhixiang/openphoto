//! UI layer: layout, panels, toolbar and document windows.
//!
//! It only turns user input into operations on `op-core` documents and draws
//! the resulting state.

mod actions;
#[cfg(target_os = "macos")]
mod app_kit;
mod clipboard;
#[cfg(target_os = "macos")]
mod color_management;
mod commands;
mod crop_tool;
mod dialogs;
mod doc_tabs;
mod document_view;
mod free_transform;
mod icons;
#[cfg(target_os = "macos")]
mod menu;
mod options_bar;
mod options_kit;
mod options_tools;
mod panels;
mod ps_icons;
mod rulers;
mod snap;
mod state;
mod theme;
mod titlebar;
mod tool_icons;
mod toolbar;
mod type_tool;
mod widgets;

#[cfg(test)]
mod ui_tests;

use std::path::PathBuf;

use egui::Frame;

use state::AppState;
use theme::{color, size};

pub struct OpenPhotoApp {
    state: AppState,
    panels: panels::Panels,
    /// The native menu bar; `None` in headless tests.
    #[cfg(target_os = "macos")]
    menu: Option<menu::NativeMenu>,
}

impl OpenPhotoApp {
    pub fn new(cc: &eframe::CreationContext<'_>, files: Vec<PathBuf>) -> Self {
        let mut app = Self::new_headless(cc, files);
        app.state.clipboard = clipboard::Clipboard::new(true);
        #[cfg(target_os = "macos")]
        {
            app.menu = Some(menu::NativeMenu::install(&cc.egui_ctx));
            if let Some(window) = cc.winit_window() {
                color_management::use_srgb(window.as_ref());
            }
        }
        app
    }

    /// Like [`Self::new`] but without installing the native menu bar, which
    /// needs a running macOS application. Used by UI tests.
    #[doc(hidden)]
    pub fn new_headless(cc: &eframe::CreationContext<'_>, files: Vec<PathBuf>) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        theme::apply_style(&cc.egui_ctx);
        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("OpenPhoto requires the wgpu renderer");
        op_render::install(render_state);

        let mut app = Self {
            state: AppState::default(),
            panels: panels::Panels::default(),
            #[cfg(target_os = "macos")]
            menu: None,
        };
        if files.is_empty() {
            actions::new_document(&mut app.state);
        } else {
            actions::open_paths(&mut app.state, files);
        }
        app
    }
}

impl OpenPhotoApp {
    /// Runs commands from the native menu bar (macOS) or from egui shortcuts
    /// (other platforms, where there is no native menu).
    fn run_commands(&mut self, ctx: &egui::Context) {
        self.state.typing = ctx.egui_wants_keyboard_input();
        #[cfg(target_os = "macos")]
        let commands = match &self.menu {
            Some(menu) => {
                let mut c = menu.poll();
                c.extend(commands::from_shortcuts_beside_menu(ctx));
                c
            }
            None => commands::from_shortcuts(ctx, &self.state),
        };
        #[cfg(not(target_os = "macos"))]
        let commands = commands::from_shortcuts(ctx, &self.state);
        for command in commands {
            commands::run(command, ctx, &mut self.state);
        }
        if !self.state.forward_events.is_empty() {
            ctx.request_repaint();
        }
    }

    fn canvas_size_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.canvas_size_dialog.take() else {
            return;
        };
        let (fg, bg) = (self.state.foreground, self.state.background);
        let active = self.state.color_picker.is_none();
        let outcome = dialog.show(ctx, fg, bg, active);
        if let Some(color) = dialog.take_color_picker_request(fg, bg) {
            self.state.color_picker = Some(state::PickerSession {
                picker: dialogs::ColorPicker::new("Color Picker", color),
                target: state::PickerTarget::CanvasExtension,
            });
        }
        match outcome {
            dialogs::Outcome::Open => self.state.canvas_size_dialog = Some(dialog),
            dialogs::Outcome::Cancel => {}
            dialogs::Outcome::Apply {
                width,
                height,
                anchor,
                fill,
            } => {
                if let Some(state) = self.state.active()
                    && (width, height) != (state.doc.width, state.doc.height)
                {
                    state.doc.resize_canvas(width, height, anchor, fill);
                    state.record("Canvas Size");
                }
            }
        }
    }

    fn fill_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.fill_dialog.take() else {
            return;
        };
        let (fg, bg) = (self.state.foreground, self.state.background);
        let active = self.state.color_picker.is_none();
        let outcome = dialog.show(ctx, fg, bg, active);
        if let Some(color) = dialog.take_color_picker_request() {
            self.state.color_picker = Some(state::PickerSession {
                picker: dialogs::ColorPicker::new("Color Picker (Fill Color)", color),
                target: state::PickerTarget::FillColor,
            });
        }
        match outcome {
            dialogs::FillOutcome::Open => self.state.fill_dialog = Some(dialog),
            dialogs::FillOutcome::Cancel => {}
            dialogs::FillOutcome::Apply { color, options } => {
                let [r, g, b, _] = color.to_rgba8();
                if let Some(state) = self.state.active() {
                    match op_core::fill::fill(&mut state.doc, [r, g, b], options) {
                        Ok(()) => state.record("Fill"),
                        Err(e) => self.state.alert = Some(e.message("Fill")),
                    }
                }
            }
        }
    }

    /// Adjustment and filter dialogs: preview on the document while open.
    fn adjust_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.adjust_dialog.take() else {
            return;
        };
        let outcome = dialog.show(ctx);
        let [r, g, b, _] = self.state.background.to_rgba8();
        let background = [r, g, b];
        let Some(state) = self
            .state
            .active_doc
            .and_then(|id| self.state.docs.get_mut(&id))
        else {
            return;
        };
        let undo_preview = |state: &mut state::DocState, dialog: &mut dialogs::AdjustDialog| {
            if dialog.previewing.take().is_some() {
                state.doc.restore(&dialog.before);
            }
        };
        match outcome {
            dialogs::AdjustOutcome::Open => {
                let wanted = dialog.effect().filter(|_| dialog.preview);
                if wanted != dialog.previewing {
                    undo_preview(state, &mut dialog);
                    if let Some(effect) = wanted
                        && effect.apply(&mut state.doc, background).is_ok()
                    {
                        dialog.previewing = Some(effect);
                    }
                    dialog.pane = None;
                }
                if dialog.wants_pane() && dialog.pane.is_none() {
                    dialog.pane = Some(pane_texture(ctx, &state.doc));
                }
                self.state.adjust_dialog = Some(dialog);
            }
            dialogs::AdjustOutcome::Cancel => undo_preview(state, &mut dialog),
            dialogs::AdjustOutcome::Apply(effect) => {
                undo_preview(state, &mut dialog);
                if let Some(values) = dialog.settings() {
                    self.state.filter_settings.insert(dialog.kind, values);
                }
                match effect.apply(&mut state.doc, background) {
                    Ok(()) => {
                        state.record(effect.name());
                        if let dialogs::Effect::Filter(filter) = effect {
                            self.state.last_filter = Some(filter);
                        }
                    }
                    Err(e) => self.state.alert = Some(e.message(effect.name())),
                }
            }
        }
    }

    /// "Save changes?" for the document being closed, and closing the
    /// window once quitting is approved.
    fn save_prompt(&mut self, ctx: &egui::Context) {
        if let Some(id) = self.state.save_prompt {
            let title = self.state.docs.get(&id).map(|d| d.doc.title.clone());
            match title {
                Some(title) => {
                    if let Some(choice) = dialogs::save_changes::show(ctx, &title) {
                        actions::answer_save_prompt(&mut self.state, choice);
                    }
                }
                None => {
                    self.state.save_prompt = None;
                    actions::continue_closing(&mut self.state);
                }
            }
        }
        if self.state.quit_approved {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn image_size_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.image_size_dialog.take() else {
            return;
        };
        match dialog.show(ctx) {
            dialogs::ImageSizeOutcome::Open => self.state.image_size_dialog = Some(dialog),
            dialogs::ImageSizeOutcome::Cancel => {}
            dialogs::ImageSizeOutcome::Apply {
                width,
                height,
                resolution,
                resample,
            } => {
                let Some(state) = self.state.active() else {
                    return;
                };
                let mut changed = false;
                if let Some(method) = resample
                    && (width, height) != (state.doc.width, state.doc.height)
                {
                    op_core::image_ops::resize(&mut state.doc, width, height, method);
                    changed = true;
                }
                if resolution != state.doc.resolution {
                    state.doc.resolution = resolution;
                    state.doc.mark_dirty();
                    changed = true;
                }
                if changed {
                    state.record("Image Size");
                }
            }
        }
    }

    fn modify_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.modify_dialog.take() else {
            return;
        };
        match dialog.show(ctx) {
            dialogs::ModifyOutcome::Open => self.state.modify_dialog = Some(dialog),
            dialogs::ModifyOutcome::Cancel => {}
            dialogs::ModifyOutcome::Apply { value, at_bounds } => {
                if let Some(state) = self.state.active()
                    && let Some(s) = state.doc.selection()
                {
                    let modified = dialog.kind.apply(s, value, at_bounds);
                    state.doc.set_selection(Some(modified));
                    state.record(dialog.kind.name());
                }
            }
        }
    }

    fn new_document_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.new_document_dialog.take() else {
            return;
        };
        let outcome = dialog.show(ctx);
        self.state.new_document_saved.extend(dialog.take_saved());
        self.state.new_document_welcome_closed = dialog.welcome_closed();
        match outcome {
            dialogs::NewDocumentOutcome::Open => self.state.new_document_dialog = Some(dialog),
            dialogs::NewDocumentOutcome::Cancel => {}
            dialogs::NewDocumentOutcome::Create {
                name,
                width,
                height,
                resolution,
                contents,
                preset,
            } => {
                // Recent: newest first, without repeats, at most 20
                let recent = &mut self.state.new_document_recent;
                recent.retain(|p| *p != preset);
                recent.insert(0, preset);
                recent.truncate(20);
                actions::create_document(
                    &mut self.state,
                    name,
                    (width, height),
                    resolution,
                    contents,
                );
            }
        }
    }

    fn new_layer_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.new_layer_dialog.take() else {
            return;
        };
        match dialog.show(ctx) {
            dialogs::NewLayerOutcome::Open => self.state.new_layer_dialog = Some(dialog),
            dialogs::NewLayerOutcome::Cancel => {}
            dialogs::NewLayerOutcome::Create(layer) => {
                if let Some(state) = self.state.active() {
                    match dialog.kind() {
                        dialogs::NewLayerKind::Layer => panels::new_layer_from(state, layer),
                        dialogs::NewLayerKind::FromBackground => {
                            panels::layer_from_background_with(state, layer)
                        }
                        dialogs::NewLayerKind::Group => panels::new_group_from(state, layer, false),
                        dialogs::NewLayerKind::GroupFromLayers => {
                            panels::new_group_from(state, layer, true)
                        }
                    }
                }
            }
        }
    }

    fn duplicate_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.duplicate_dialog.take() else {
            return;
        };
        match dialog.show(ctx) {
            dialogs::DuplicateOutcome::Open => self.state.duplicate_dialog = Some(dialog),
            dialogs::DuplicateOutcome::Cancel => {}
            dialogs::DuplicateOutcome::Duplicate { name, to } => {
                actions::duplicate_layer(&mut self.state, &name, to);
            }
        }
    }

    fn lock_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.lock_dialog.take() else {
            return;
        };
        match dialog.show(ctx) {
            dialogs::LockOutcome::Open => self.state.lock_dialog = Some(dialog),
            dialogs::LockOutcome::Cancel => {}
            dialogs::LockOutcome::Apply(locks) => {
                if let Some(state) = self.state.active()
                    && op_core::layer_ops::set_selected_locks(&mut state.doc, locks)
                {
                    state.record("Lock Layers");
                }
            }
        }
    }

    fn rotate_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.rotate_dialog.take() else {
            return;
        };
        match dialog.show(ctx) {
            dialogs::RotateOutcome::Open => self.state.rotate_dialog = Some(dialog),
            dialogs::RotateOutcome::Cancel => {}
            dialogs::RotateOutcome::Rotate(degrees) => {
                let background = self.state.background;
                if let Some(state) = self.state.active()
                    && op_core::image_ops::rotate_arbitrary(&mut state.doc, degrees, background)
                {
                    state.record("Rotate Canvas");
                }
            }
        }
    }

    fn new_guide_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.new_guide_dialog.take() else {
            return;
        };
        match dialog.show(ctx) {
            dialogs::NewGuideOutcome::Open => self.state.new_guide_dialog = Some(dialog),
            dialogs::NewGuideOutcome::Cancel => {}
            dialogs::NewGuideOutcome::Apply(guide) => {
                if let Some(state) = self.state.active() {
                    state.doc.guides.push(guide);
                    state.record("New Guide");
                }
            }
        }
    }

    fn trim_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.trim_dialog.take() else {
            return;
        };
        match dialog.show(ctx) {
            dialogs::TrimOutcome::Open => self.state.trim_dialog = Some(dialog),
            dialogs::TrimOutcome::Cancel => {}
            dialogs::TrimOutcome::Apply { basis, sides } => {
                if let Some(state) = self.state.active()
                    && op_core::image_ops::trim(&mut state.doc, basis, sides)
                {
                    state.record("Trim");
                }
            }
        }
    }

    fn equalize_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.equalize_dialog.take() else {
            return;
        };
        match dialog.show(ctx) {
            dialogs::EqualizeOutcome::Open => self.state.equalize_dialog = Some(dialog),
            dialogs::EqualizeOutcome::Cancel => {}
            dialogs::EqualizeOutcome::Apply { entire_image } => {
                let adjustment = if entire_image {
                    op_core::adjust::Adjustment::EqualizeEntireImage
                } else {
                    op_core::adjust::Adjustment::Equalize
                };
                if let Some(state) = self.state.active() {
                    match op_core::adjust::apply(&mut state.doc, adjustment) {
                        Ok(()) => state.record("Equalize"),
                        Err(e) => self.state.alert = Some(e.message("Equalize")),
                    }
                }
            }
        }
    }

    /// The Color Picker, drawn after (on top of) Canvas Size.
    fn color_picker(&mut self, ctx: &egui::Context) {
        let Some(mut session) = self.state.color_picker.take() else {
            return;
        };
        let outcome = session.picker.show(ctx);
        self.state
            .swatches
            .extend(session.picker.take_added_swatches());
        match outcome {
            dialogs::ColorPickerOutcome::Open => self.state.color_picker = Some(session),
            dialogs::ColorPickerOutcome::Cancel => {}
            dialogs::ColorPickerOutcome::Ok(color) => match session.target {
                state::PickerTarget::Foreground => self.state.foreground = color,
                state::PickerTarget::Background => self.state.background = color,
                state::PickerTarget::CanvasExtension => {
                    if let Some(dialog) = &mut self.state.canvas_size_dialog {
                        dialog.set_other_color(color);
                    }
                }
                state::PickerTarget::FillColor => {
                    if let Some(dialog) = &mut self.state.fill_dialog {
                        dialog.set_color(color);
                    }
                }
            },
        }
    }

    /// Shows the History panel to the left of the icon strip, the way
    /// Photoshop pops out a collapsed panel: its right edge touches the strip
    /// and its top sits 13 pt above the History button. Clicking anywhere else
    /// closes it (Photoshop's default "Auto-Collapse Iconic Panels").
    fn history_popout(&mut self, ctx: &egui::Context, strip: egui::Rect, button: egui::Rect) {
        let size = self.state.history_panel.size();
        let pos = egui::pos2(strip.left() - size.x, button.top() - theme::pt(13.0));
        let mut collapse = false;
        let area = egui::Area::new(egui::Id::new("history-popout"))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .shadow(ui.visuals().popup_shadow)
                    .show(ui, |ui| {
                        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                        let mut panel = ui.new_child(egui::UiBuilder::new().max_rect(rect));
                        panel.set_clip_rect(rect.expand(1.0));
                        let action = panels::history::show(&mut panel, &mut self.state);
                        collapse = matches!(action, panels::history::Action::Collapse);
                    });
            });

        let clicked_outside = ctx.input(|i| {
            i.pointer.any_pressed()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|p| !area.response.rect.contains(p) && !button.contains(p))
        });
        if clicked_outside || collapse {
            self.state.history_open = false;
        }
    }
}

impl eframe::App for OpenPhotoApp {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        raw_input.events.append(&mut self.state.forward_events);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // OPENPHOTO_FRAME_LOG=1: each frame's interval (update and render)
        // on stderr, to check that dragging keeps up in the real app
        static FRAME_LOG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if *FRAME_LOG.get_or_init(|| std::env::var_os("OPENPHOTO_FRAME_LOG").is_some()) {
            eprintln!("frame {:.1} ms", ctx.input(|i| i.unstable_dt) * 1000.0);
        }

        // Closing the window with unsaved changes asks about each document
        // first, like quitting
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested
            && !self.state.quit_approved
            && self.state.docs.values().any(|d| d.is_dirty())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            if self.state.save_prompt.is_none() {
                actions::quit(&mut self.state);
            }
        }

        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .filter(|p| !p.as_os_str().is_empty())
                .collect()
        });
        if !dropped.is_empty() {
            actions::open_paths(&mut self.state, dropped);
        }
        self.run_commands(&ctx);
        actions::handle_tool_keys(&ctx, &mut self.state);

        let bar_frame = Frame::NONE.fill(color::PANEL);

        if cfg!(target_os = "macos") {
            egui::Panel::top("titlebar")
                .show_separator_line(false)
                .exact_size(size::TITLE_BAR)
                .resizable(false)
                .frame(bar_frame)
                .show(ui, titlebar::show);
        }
        egui::Panel::top("options-bar")
            .show_separator_line(false)
            .exact_size(size::OPTIONS_BAR)
            .resizable(false)
            .frame(bar_frame)
            .show(ui, |ui| options_bar::show(ui, &mut self.state));
        egui::Panel::left("toolbar")
            .show_separator_line(false)
            .exact_size(size::TOOLBAR)
            .resizable(false)
            .frame(bar_frame)
            .show(ui, |ui| toolbar::show(ui, &mut self.state));
        egui::Panel::right("panels")
            .show_separator_line(false)
            .exact_size(size::PANEL_COLUMN)
            .resizable(false)
            .frame(bar_frame)
            .show(ui, |ui| self.panels.show(ui, &mut self.state));
        let strip = egui::Panel::right("icon-strip")
            .show_separator_line(false)
            .exact_size(size::ICON_STRIP)
            .resizable(false)
            .frame(bar_frame)
            .show(ui, |ui| panels::icon_strip(ui, &mut self.state));
        let (strip_rect, history_button) = (strip.response.rect, strip.inner);

        egui::CentralPanel::no_frame()
            .frame(Frame::NONE.fill(color::PASTEBOARD))
            .show(ui, |ui| {
                let area = ui.max_rect();
                if self.state.doc_order.is_empty() {
                    return;
                }
                let tabs =
                    egui::Rect::from_min_size(area.min, egui::vec2(area.width(), doc_tabs::HEIGHT));
                doc_tabs::show(ui, &mut self.state, tabs);
                doc_tabs::ensure_active(&mut self.state);
                let view =
                    egui::Rect::from_min_max(egui::pos2(area.left(), tabs.bottom()), area.max);
                if let Some(id) = self.state.active_doc {
                    let mut child =
                        ui.new_child(egui::UiBuilder::new().max_rect(view).id_salt(("doc", id.0)));
                    child.set_clip_rect(view);
                    document_view::show(&mut child, &mut self.state, id);
                }
            });
        doc_tabs::ensure_active(&mut self.state);

        if self.state.history_open {
            self.history_popout(&ctx, strip_rect, history_button);
        }
        panels::floating::show(&ctx, &mut self.state, strip_rect.left(), strip_rect.top());

        self.canvas_size_dialog(&ctx);
        self.fill_dialog(&ctx);
        self.trim_dialog(&ctx);
        self.equalize_dialog(&ctx);
        self.image_size_dialog(&ctx);
        self.new_guide_dialog(&ctx);
        self.new_layer_dialog(&ctx);
        self.duplicate_dialog(&ctx);
        self.lock_dialog(&ctx);
        self.rotate_dialog(&ctx);
        self.new_document_dialog(&ctx);
        self.modify_dialog(&ctx);
        self.adjust_dialog(&ctx);
        self.save_prompt(&ctx);
        self.color_picker(&ctx);

        #[cfg(target_os = "macos")]
        if let Some(menu) = &mut self.menu {
            menu.update(&self.state);
        }

        if let Some(msg) = self.state.alert.clone() {
            let mut alert = dialogs::alert::Alert::error(msg);
            if dialogs::alert::show(&ctx, &mut alert).is_some() {
                self.state.alert = None;
            }
        }
        if let Some(mut prompt) = self.state.delete_group_prompt.take() {
            match dialogs::alert::show(&ctx, &mut prompt) {
                None => self.state.delete_group_prompt = Some(prompt),
                // Group and Contents, Group Only, Cancel
                Some(dialogs::alert::Answer::Choice(0)) => {
                    if let Some(state) = self.state.active() {
                        panels::delete_active_layer(state);
                    }
                }
                Some(dialogs::alert::Answer::Choice(1)) => {
                    if let Some(state) = self.state.active()
                        && op_core::layer_ops::delete_selected_keep_contents(&mut state.doc)
                    {
                        state.record("Delete Layer");
                    }
                }
                Some(_) => {}
            }
        }
        if let Some(mut prompt) = self.state.flatten_prompt.take() {
            match dialogs::alert::show(&ctx, &mut prompt) {
                None => self.state.flatten_prompt = Some(prompt),
                Some(dialogs::alert::Answer::Cancel | dialogs::alert::Answer::Choice(_)) => {}
                Some(dialogs::alert::Answer::Ok { dont_show_again }) => {
                    self.state.skip_flatten_prompt |= dont_show_again;
                    if let Some(state) = self.state.active() {
                        op_core::layer_ops::flatten(&mut state.doc);
                        state.record("Flatten Image");
                    }
                }
            }
        }
    }
}

/// The classic filter dialogs' preview pane: the middle of the document,
/// as composited now, up to 392 pixels square (the pane at 100%).
fn pane_texture(ctx: &egui::Context, doc: &op_core::Document) -> egui::TextureHandle {
    let (w, h) = (doc.width as usize, doc.height as usize);
    let (pw, ph) = (w.min(392), h.min(392));
    let (x0, y0) = ((w - pw) / 2, (h - ph) / 2);
    let all = doc.composite_rgba8();
    let mut pixels = Vec::with_capacity(pw * ph);
    for y in y0..y0 + ph {
        for x in x0..x0 + pw {
            let i = (y * w + x) * 4;
            pixels.push(egui::Color32::from_rgba_unmultiplied(
                all[i],
                all[i + 1],
                all[i + 2],
                all[i + 3],
            ));
        }
    }
    let image = egui::ColorImage::new([pw, ph], pixels);
    ctx.load_texture("filter-pane", image, egui::TextureOptions::NEAREST)
}
