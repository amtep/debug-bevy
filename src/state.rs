use bevy::{asset::LoadedFolder, prelude::*};

use crate::{config::Config, constants::ui::BLACK};

pub fn plugin(app: &mut App) {
    app.init_state::<GameState>()
        .add_systems(OnEnter(GameState::Load), load_setup)
        .add_systems(Update, load_update.run_if(in_state(GameState::Load)))
        .configure_sets(
            OnEnter(GameState::Main),
            (
                MainSetupSet::Default,
                MainSetupSet::Regions,
                MainSetupSet::Ui,
                MainSetupSet::Bases,
                MainSetupSet::Followers,
                MainSetupSet::Save,
                MainSetupSet::Late,
            )
                .chain(),
        );
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum GameState {
    #[default]
    Load,
    MainMenu,
    Main,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MainSetupSet {
    Default,
    Regions,
    Ui,
    Bases,
    Followers,
    Save,
    Late,
}

#[derive(Resource)]
struct LoadHandle(Handle<LoadedFolder>);

fn load_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Entered asset load state");
    commands.insert_resource(LoadHandle(asset_server.load_folder(".")));
    commands.spawn(Camera2d);
    commands.spawn((
        DespawnOnExit(GameState::Load),
        Node {
            width: percent(100.0),
            height: percent(100.0),
            ..default()
        },
        BackgroundColor::from(BLACK),
    ));
}

fn load_update(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    load_handle: Res<LoadHandle>,
    _: If<Res<Config>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if asset_server.is_loaded_with_dependencies(load_handle.0.id()) {
        info!("Exiting asset load state");
        commands.remove_resource::<LoadHandle>();
        next_state.set(GameState::MainMenu);
    }
}
