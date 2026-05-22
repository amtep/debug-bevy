mod achievements;
mod bases;
mod common;
mod config;
mod constants;
#[cfg(feature = "dev")]
mod dev;
mod discoveries;
mod effects;
mod followers;
mod funds;
mod modifiers;
mod new_game;
mod regions;
mod rng;
mod save_load;
mod state;
mod suspicion;
mod tasks;
mod text;
mod time;
mod ui;

pub fn app() {
    use bevy::{prelude::*, ui_widgets::UiWidgetsPlugins, window::WindowMode};
    use bevy_ui_text_input::TextInputPlugin;

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
            UiWidgetsPlugins,
            (
                #[cfg(feature = "dev")]
                dev::plugin,
                state::plugin,
                text::plugin,
                regions::plugin,
                bases::plugin,
                tasks::plugin,
                rng::plugin,
                funds::plugin,
                ui::plugin,
                time::plugin,
                followers::plugin,
                discoveries::plugin,
                suspicion::plugin,
                new_game::plugin,
                save_load::plugin,
            ), // need a new tuple because of length
            (config::plugin, achievements::plugin),
        ))
        .run();
}
