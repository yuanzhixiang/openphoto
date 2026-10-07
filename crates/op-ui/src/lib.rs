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
mod native_popup;
mod options_bar;
mod options_kit;
mod options_tools;
mod panels;
mod perspective_crop;
mod ps_icons;
mod recent;
mod rulers;
mod selection_brush;
mod smart_guides;
mod snap;
mod state;
mod status_info;
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
    /// The screen mode the window was last set up for, and whether it was
    /// maximized (zoomed) before Full Screen Mode With Menu Bar zoomed it.
    screen_applied: state::ScreenMode,
    was_maximized: bool,
}

impl OpenPhotoApp {
    pub fn new(cc: &eframe::CreationContext<'_>, files: Vec<PathBuf>) -> Self {
        let mut app = Self::new_headless(cc, files);
        app.state.clipboard = clipboard::Clipboard::new(true);
        if let Some(store) = crop_tool::CropPresets::default_store() {
            app.state.crop_presets = crop_tool::CropPresets::load(store);
        }
        if let Some(store) = recent::RecentFiles::default_store() {
            app.state.recent = recent::RecentFiles::load(store);
            // The files opened from the command line, before the list was read
            let opened: Vec<PathBuf> = app
                .state
                .doc_order
                .iter()
                .filter_map(|id| app.state.docs.get(id)?.path.clone())
                .collect();
            for path in opened {
                app.state.recent.add(&path);
            }
        }
        #[cfg(target_os = "macos")]
        {
            let mut menu = menu::NativeMenu::install(&cc.egui_ctx);
            if let Some(window) = cc.winit_window() {
                color_management::use_srgb(window.as_ref());
                use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                if let Ok(handle) = window.window_handle()
                    && let RawWindowHandle::AppKit(appkit) = handle.as_raw()
                {
                    menu.set_view(appkit.ns_view.as_ptr());
                    native_popup::set_available(&cc.egui_ctx, true);
                }
            }
            app.menu = Some(menu);
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
            screen_applied: state::ScreenMode::Standard,
            was_maximized: false,
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
    /// Sets the window up for a new screen mode: Full Screen Mode With
    /// Menu Bar zooms the window to fill the screen, Full Screen Mode makes
    /// it full screen; Standard undoes what the others did.
    fn apply_screen_mode(&mut self, ctx: &egui::Context) {
        use state::ScreenMode::*;
        let mode = self.state.screen_mode;
        if mode == self.screen_applied {
            return;
        }
        let from = std::mem::replace(&mut self.screen_applied, mode);
        if from == Standard {
            self.was_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        }
        if from == Full {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        }
        match mode {
            Full => ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true)),
            FullWithMenus => ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true)),
            Standard => {
                if !self.was_maximized {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
                }
            }
        }
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
                        Ok(()) => state.record_fadeable("Fill"),
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
        dialog.blocked = self.state.color_picker.is_some()
            || self.state.gradient_editor.is_some()
            || self.state.auto_options_dialog.is_some();
        let outcome = dialog.show(ctx);
        if let Some(rgb) = dialog.take_color_request() {
            self.state
                .open_color_picker(state::PickerTarget::PhotoFilter(rgb));
        }
        // Levels' and Curves' Options... open Auto Color Correction Options
        if let Some(options) = dialog.take_auto_options_request() {
            self.state.auto_options_dialog =
                Some(dialogs::auto_options::AutoOptionsDialog::new(options));
        }
        // Gradient Map's gradient opens the Gradient Editor
        if let Some(g) = dialog.take_editor_request() {
            let presets = self.state.gradient_presets();
            self.state.gradient_editor = Some((
                dialogs::gradient_editor::GradientEditor::new(g, presets),
                state::EditorTarget::GradientMap,
            ));
        }
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
        // An eyedropper chosen in Levels or Curves: a click on the image
        // (outside the dialog) samples it as it was before adjusting
        let click = ctx.input(|i| {
            i.pointer
                .primary_pressed()
                .then(|| i.pointer.interact_pos())
                .flatten()
        });
        if let Some(p) = click
            && (dialog.sampling() || dialog.targeting())
            && !dialog.rect.contains(p)
            && state.view.viewport.contains(p)
        {
            undo_preview(state, &mut dialog);
            let d = document_view::to_doc(state, p, ctx.pixels_per_point());
            let (w, h) = (state.doc.width as f32, state.doc.height as f32);
            if (0.0..w).contains(&d.x) && (0.0..h).contains(&d.y) {
                let pixels = state.doc.composite_rgba8();
                let i = (d.y as usize * state.doc.width as usize + d.x as usize) * 4;
                let rgb = [pixels[i], pixels[i + 1], pixels[i + 2]];
                if dialog.targeting() {
                    let hue = ctx.input(|i| i.modifiers.command);
                    dialog.target_press(rgb, hue);
                    dialog.target_from = Some(p);
                } else {
                    dialog.sample(rgb);
                }
            }
        }
        // The hand's drag: sideways changes Hue/Saturation's range
        // (saturation, or hue), up and down Curves' point
        if let Some(from) = dialog.target_from {
            let (down, pos, hue) = ctx.input(|i| {
                (
                    i.pointer.primary_down(),
                    i.pointer.interact_pos(),
                    i.modifiers.command,
                )
            });
            if let Some(pos) = pos {
                dialog.target_drag((pos - from) / theme::pt(1.0), hue);
            }
            if !down {
                dialog.target_from = None;
                dialog.target_release();
            }
        }
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
                // Displace asks for its map after OK, as Photoshop does
                let effect = match effect {
                    dialogs::Effect::Filter(op_core::filter::Filter::Displace {
                        scale,
                        stretch,
                        undefined,
                        ..
                    }) => {
                        let Some(map) = displacement_map() else {
                            return;
                        };
                        dialogs::Effect::Filter(op_core::filter::Filter::Displace {
                            map,
                            scale,
                            stretch,
                            undefined,
                        })
                    }
                    other => other,
                };
                if let Some(values) = dialog.settings() {
                    self.state.filter_settings.insert(dialog.kind, values);
                }
                if let Some(last) = dialog.remembered() {
                    self.state.last_adjustments.insert(dialog.kind, last);
                }
                // Tiles may fill with the foreground color
                let effect = match effect {
                    dialogs::Effect::Filter(op_core::filter::Filter::Tiles {
                        count,
                        offset,
                        fill,
                        ..
                    }) => {
                        let [r, g, b, _] = self.state.foreground.to_rgba8();
                        dialogs::Effect::Filter(op_core::filter::Filter::Tiles {
                            count,
                            offset,
                            fill,
                            foreground: [r, g, b],
                        })
                    }
                    other => other,
                };
                match effect.apply(&mut state.doc, background) {
                    Ok(()) => {
                        state.record_fadeable(effect.name());
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

    /// Auto Color Correction Options: its OK runs Levels' or Curves' Auto
    /// with the options (and keeps them as the defaults when asked); its
    /// target colors use the Color Picker.
    fn auto_options_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.auto_options_dialog.take() else {
            return;
        };
        let active = self.state.color_picker.is_none();
        let outcome = dialog.show(ctx, active);
        if let Some((k, rgb)) = dialog.take_color_request() {
            self.state
                .open_color_picker(state::PickerTarget::AutoTarget(k, rgb));
        }
        match outcome {
            dialogs::auto_options::Outcome::Open => self.state.auto_options_dialog = Some(dialog),
            dialogs::auto_options::Outcome::Cancel => {}
            dialogs::auto_options::Outcome::Ok(options, save) => {
                if save {
                    self.state.auto_saved = Some(options);
                }
                if let Some(adjust) = &mut self.state.adjust_dialog {
                    adjust.set_auto_options(options);
                }
            }
        }
    }

    /// The Gradient Editor: its OK goes to the Gradient tool or to
    /// Gradient Map; its color stops use the Color Picker.
    fn gradient_editor(&mut self, ctx: &egui::Context) {
        let Some((mut editor, target)) = self.state.gradient_editor.take() else {
            return;
        };
        let active = self.state.color_picker.is_none();
        let outcome = editor.show(ctx, active);
        if let Some(rgb) = editor.take_color_request() {
            self.state
                .open_color_picker(state::PickerTarget::GradientStop(rgb));
        }
        // Gradients made with New are kept for the session
        let made: Vec<_> = editor
            .presets
            .iter()
            .skip(dialogs::gradient_editor::presets([0; 3], [0; 3]).len())
            .cloned()
            .collect();
        self.state.gradient_made = made;
        match outcome {
            dialogs::gradient_editor::Outcome::Open => {
                self.state.gradient_editor = Some((editor, target));
            }
            dialogs::gradient_editor::Outcome::Cancel => {}
            dialogs::gradient_editor::Outcome::Ok(g) => match target {
                state::EditorTarget::Tool => self.state.gradient_preset = Some(g),
                state::EditorTarget::GradientMap => {
                    if let Some(dialog) = &mut self.state.adjust_dialog {
                        dialog.set_map_gradient(g);
                    }
                }
            },
        }
    }

    /// The Crop tool's New Crop Preset... and Delete Crop Preset... dialogs.
    fn crop_preset_dialogs(&mut self, ctx: &egui::Context) {
        use dialogs::new_preset::Outcome;
        use dialogs::size_presets::DeleteOutcome;
        if let Some(mut dialog) = self.state.new_crop_preset.take() {
            match dialog.show(ctx) {
                Outcome::Open => self.state.new_crop_preset = Some(dialog),
                Outcome::Cancel => {}
                Outcome::Ok(name) => {
                    let preset = self.state.crop_options.to_user(&name);
                    self.state.crop_presets.add(preset);
                    self.state.crop_options.user = Some(self.state.crop_presets.list().len() - 1);
                }
            }
        }
        if let Some(mut dialog) = self.state.delete_crop_preset.take() {
            match dialog.show(ctx, None) {
                DeleteOutcome::Open => self.state.delete_crop_preset = Some(dialog),
                DeleteOutcome::Cancel => {}
                DeleteOutcome::Delete(i) => {
                    self.state.crop_presets.remove(i);
                    let options = &mut self.state.crop_options;
                    options.user = match options.user {
                        Some(u) if u == i => None,
                        Some(u) if u > i => Some(u - 1),
                        u => u,
                    };
                }
            }
        }
    }

    fn image_size_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.state.image_size_dialog.take() else {
            return;
        };
        let outcome = dialog.show(ctx);
        self.state.image_size_extra = dialog.extra;
        match outcome {
            dialogs::ImageSizeOutcome::Open => self.state.image_size_dialog = Some(dialog),
            dialogs::ImageSizeOutcome::Cancel => {}
            dialogs::ImageSizeOutcome::Apply {
                width,
                height,
                resolution,
                resample,
                reduce_noise,
            } => {
                let Some(state) = self.state.active() else {
                    return;
                };
                let mut changed = false;
                if let Some(method) = resample
                    && (width, height) != (state.doc.width, state.doc.height)
                {
                    op_core::image_ops::resize_reducing_noise(
                        &mut state.doc,
                        width,
                        height,
                        method,
                        reduce_noise,
                    );
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

    /// Edit › Fade: the layer previews the faded result while the dialog
    /// is open (Preview on); Cancel puts the edit's result back, OK keeps
    /// the faded pixels as a new "Fade <edit>" state.
    fn fade_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut fade) = self.state.fade_dialog.take() else {
            return;
        };
        let outcome = fade.dialog.show(ctx);
        let Some(state) = self.state.active() else {
            return;
        };
        let layer = fade.source.layer;
        let set = |state: &mut state::DocState, image: op_core::tile::TiledImage| {
            if let Some(slot) = state.doc.layer_mut(layer).and_then(|l| l.image_mut()) {
                *slot = image;
            }
            state.doc.mark_dirty();
        };
        let faded = |fade: &state::FadeState, opacity: f32| {
            op_core::fade::fade(
                &fade.source.before,
                &fade.source.after,
                opacity,
                fade.dialog.mode,
            )
        };
        match outcome {
            dialogs::FadeOutcome::Open => {
                let opacity = fade.dialog.opacity().unwrap_or(1.0);
                let now = (opacity, fade.dialog.mode, fade.dialog.preview);
                if fade.shown != Some(now) {
                    let image = if fade.dialog.preview {
                        faded(&fade, opacity)
                    } else {
                        fade.source.after.clone()
                    };
                    set(state, image);
                    fade.shown = Some(now);
                }
                self.state.fade_dialog = Some(fade);
            }
            dialogs::FadeOutcome::Cancel => set(state, fade.source.after.clone()),
            dialogs::FadeOutcome::Apply => {
                let opacity = fade.dialog.opacity().unwrap_or(1.0);
                set(state, faded(&fade, opacity));
                state.record(&format!("Fade {}", fade.source.name));
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
                        Ok(()) => state.record_fadeable("Equalize"),
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
                state::PickerTarget::AutoTarget(k, _) => {
                    if let Some(dialog) = &mut self.state.auto_options_dialog {
                        let [r, g, b, _] = color.to_rgba8();
                        dialog.set_target(k, [r, g, b]);
                    }
                }
                state::PickerTarget::GradientStop(_) => {
                    if let Some((editor, _)) = &mut self.state.gradient_editor {
                        let [r, g, b, _] = color.to_rgba8();
                        editor.set_stop_color([r, g, b]);
                    }
                }
                state::PickerTarget::PhotoFilter(_) => {
                    if let Some(dialog) = &mut self.state.adjust_dialog {
                        let [r, g, b, _] = color.to_rgba8();
                        dialog.set_filter_color([r, g, b]);
                    }
                }
                state::PickerTarget::CropShield => {
                    let [r, g, b, _] = color.to_rgba8();
                    self.state.crop_options.shield_color = Some(egui::Color32::from_rgb(r, g, b));
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
        self.apply_screen_mode(&ctx);

        let bar_frame = Frame::NONE.fill(color::PANEL);
        let mode = self.state.screen_mode;
        let (hide_tools, hide_panels) = (self.state.hide_tools, self.state.hide_panels);

        if cfg!(target_os = "macos") && mode == state::ScreenMode::Standard {
            egui::Panel::top("titlebar")
                .show_separator_line(false)
                .exact_size(size::TITLE_BAR)
                .resizable(false)
                .frame(bar_frame)
                .show(ui, titlebar::show);
        }
        if !hide_tools {
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
        }
        let (strip_rect, history_button) = if hide_panels {
            (egui::Rect::NOTHING, egui::Rect::NOTHING)
        } else {
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
            (strip.response.rect, strip.inner)
        };

        // Full Screen Mode's pasteboard is black, as in Photoshop
        let pasteboard = if mode == state::ScreenMode::Full {
            egui::Color32::BLACK
        } else {
            color::PASTEBOARD
        };
        egui::CentralPanel::no_frame()
            .frame(Frame::NONE.fill(pasteboard))
            .show(ui, |ui| {
                let area = ui.max_rect();
                if self.state.doc_order.is_empty() {
                    return;
                }
                // The full screen modes show only the active document
                let tabs_height = if mode == state::ScreenMode::Standard {
                    doc_tabs::HEIGHT
                } else {
                    0.0
                };
                let tabs =
                    egui::Rect::from_min_size(area.min, egui::vec2(area.width(), tabs_height));
                if tabs_height > 0.0 {
                    doc_tabs::show(ui, &mut self.state, tabs);
                }
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

        if !hide_panels {
            if self.state.history_open {
                self.history_popout(&ctx, strip_rect, history_button);
            }
            panels::floating::show(&ctx, &mut self.state, strip_rect.left(), strip_rect.top());
        }

        self.canvas_size_dialog(&ctx);
        self.fill_dialog(&ctx);
        self.trim_dialog(&ctx);
        self.fade_dialog(&ctx);
        self.equalize_dialog(&ctx);
        self.image_size_dialog(&ctx);
        self.crop_preset_dialogs(&ctx);
        self.gradient_editor(&ctx);
        self.auto_options_dialog(&ctx);
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
        // The status bar's arrow: its native menu (without one, as in
        // tests, the request stays and the commands are run directly)
        #[cfg(target_os = "macos")]
        if let Some(menu) = &self.menu {
            if std::mem::take(&mut self.state.status_menu) {
                menu.popup_status(&self.state);
            }
            if std::mem::take(&mut self.state.ruler_menu) {
                menu.popup_ruler_units(&self.state);
            }
            // A dialog dropdown's menu
            if let Some(request) = native_popup::take_request(&ctx) {
                let scale = ctx.pixels_per_point() / ctx.native_pixels_per_point().unwrap_or(1.0);
                if let Some(index) = menu.popup_menu(&request, scale) {
                    native_popup::deliver(&ctx, request.id, index);
                }
            }
        }

        if let Some(msg) = self.state.alert.clone() {
            let mut alert = dialogs::alert::Alert::for_message(msg);
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
        if let Some(mut prompt) = self.state.full_screen_prompt.take() {
            match dialogs::alert::show(&ctx, &mut prompt) {
                None => self.state.full_screen_prompt = Some(prompt),
                Some(dialogs::alert::Answer::Ok { dont_show_again }) => {
                    self.state.skip_full_screen_prompt |= dont_show_again;
                    self.state.enter_screen_mode(state::ScreenMode::Full);
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

/// Displace's map: a PSD (or other image) chosen in the open panel,
/// registered for the filter (`op_core::more_filters::register_map`).
fn displacement_map() -> Option<u32> {
    let path = rfd::FileDialog::new()
        .set_title("Choose a displacement map.")
        .add_filter("Photoshop", &["psd"])
        .add_filter("Images", &["png", "jpg", "jpeg", "tif", "tiff"])
        .pick_file()?;
    let doc = op_io::open(&path).ok()?;
    let pixels: Vec<[u8; 4]> = doc.composite_rgba8().as_chunks::<4>().0.to_vec();
    Some(op_core::more_filters::register_map(
        doc.width as usize,
        doc.height as usize,
        pixels,
    ))
}
