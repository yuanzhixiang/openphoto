//! Tool presets: a tool with its options saved under a name (the Tool
//! Presets panel and the options bar's Tool Presets picker).

use op_tools::Tool;

use crate::state::{AppState, PaintOptions};

#[derive(Clone, Debug)]
pub struct ToolPreset {
    pub name: String,
    pub tool: Tool,
    /// The brush (painting tools).
    pub paint: Option<PaintOptions>,
    /// The options bar's other settings, by key.
    pub settings: Vec<(&'static str, String)>,
}

/// A preset of the current tool with its options now, named `name`.
pub fn capture(app: &mut AppState, name: String) -> ToolPreset {
    let tool = app.tool;
    let paint = app.paint_options(tool).copied();
    let settings = crate::options_tools::setting_keys(tool)
        .into_iter()
        .filter_map(|key| app.tool_settings.get(key).map(|v| (key, v.clone())))
        .collect();
    ToolPreset {
        name,
        tool,
        paint,
        settings,
    }
}

/// The preset's tool becomes the current one, with the preset's options.
pub fn apply(app: &mut AppState, preset: &ToolPreset) {
    app.select_tool(preset.tool);
    if let (Some(paint), Some(options)) = (preset.paint, app.paint_options(preset.tool)) {
        *options = paint;
    }
    for (key, value) in &preset.settings {
        app.tool_settings.insert(key, value.clone());
    }
}

/// The name a new preset of `tool` starts with: the tool's name and the
/// next number, as Photoshop suggests it ("Brush Tool 1").
pub fn suggested_name(app: &AppState, tool: Tool) -> String {
    let n = app.tool_presets.iter().filter(|p| p.tool == tool).count() + 1;
    format!("{} {n}", tool.name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_and_apply() {
        let mut app = AppState::default();
        app.select_tool(Tool::Brush);
        app.brush.size = 77.0;
        *app.setting("brush.mode", "0") = "5".into();
        let preset = capture(&mut app, "Big multiply".into());
        assert_eq!(preset.paint.unwrap().size, 77.0);
        assert!(preset.settings.contains(&("brush.mode", "5".into())));
        app.select_tool(Tool::Eraser);
        app.brush.size = 10.0;
        *app.setting("brush.mode", "0") = "0".into();
        apply(&mut app, &preset);
        assert_eq!(app.tool, Tool::Brush);
        assert_eq!(app.brush.size, 77.0);
        assert_eq!(
            app.tool_settings.get("brush.mode").map(String::as_str),
            Some("5")
        );
        assert_eq!(suggested_name(&app, Tool::Brush), "Brush Tool 1");
    }
}
