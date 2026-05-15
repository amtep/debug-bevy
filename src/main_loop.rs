use bevy::{prelude::*, ui_widgets::ScrollbarPlugin, window::WindowMode};
use bevy_ui_text_input::TextInputPlugin;

pub fn main_loop() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // During development: use the assets from the source dir.
                    // This is the default, but here the path is set regardless
                    // of the current directory.
                    file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                        ..default()
                    }),
                    ..default()
                }),
            TextInputPlugin,
            ScrollbarPlugin,
            (
                #[cfg(feature = "dev")]
                crate::dev::plugin,
                crate::state::plugin,
                crate::text::plugin,
                crate::regions::plugin,
                crate::bases::plugin,
                crate::tasks::plugin,
                crate::rng::plugin,
                crate::funds::plugin,
                crate::ui::plugin,
                crate::time::plugin,
                crate::followers::plugin,
                crate::discoveries::plugin,
                crate::suspicion::plugin,
                crate::new_game::plugin,
                crate::save_load::plugin,
            ), // need a new tuple because of length
            (crate::config::plugin, crate::achievements::plugin),
        ))
        .run();
}
