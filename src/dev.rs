use bevy::prelude::*;

use crate::{
    common::{CultName, CultSymbol, Dev, Difficulty},
    discoveries::ResearchPoints,
    funds::Funds,
    new_game::{DifficultiesAsset, DifficultiesHandle, NewGame},
    regions::Region,
    state::GameState,
    text::TextKey,
    ui::toasts::WaitingToasts,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, listen_dev_keys.run_if(in_state(GameState::Main)))
        .add_systems(
            Update,
            listen_dev_keys_main_menu.run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(Update, debug_entity_count)
        .add_systems(OnExit(GameState::Main), cleanup_main);
}

fn listen_dev_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut funds: ResMut<Funds>,
    mut research_points: ResMut<ResearchPoints>,
    mut toasts: ResMut<WaitingToasts>,
) {
    if keys.just_pressed(KeyCode::KeyF) {
        if keys.pressed(KeyCode::AltLeft) {
            // Alt + F
            funds.0 += 100_000;
        }
        if keys.pressed(KeyCode::ControlLeft) {
            // Ctrl + F
            funds.0 -= 100_000;
        }
    }
    if keys.just_pressed(KeyCode::KeyD) && keys.pressed(KeyCode::AltLeft) {
        research_points.0 += 1_000;
    }
    if keys.just_pressed(KeyCode::KeyT) && keys.pressed(KeyCode::ControlLeft) {
        toasts.0.push(TextKey::new("debug-toast"));
    }
}

fn listen_dev_keys_main_menu(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    difficulties_handle: Res<DifficultiesHandle>,
    difficulties_assets: Res<Assets<DifficultiesAsset>>,
) {
    if keys.just_pressed(KeyCode::KeyG)
        && keys.pressed(KeyCode::AltLeft)
        && *game_state.get() == GameState::MainMenu
    {
        // Alt + G
        commands.insert_resource(CultName("DEV".into()));
        commands.insert_resource(CultSymbol(0));
        commands.init_resource::<Dev>();

        let (name, difficulty) = difficulties_assets
            .get(difficulties_handle.0.id())
            .unwrap()
            .0
            .iter()
            .find(|(_, settings)| settings.default)
            .unwrap();

        commands.insert_resource(Difficulty(name.clone()));
        commands.insert_resource(NewGame {
            difficulty: difficulty.clone(),
            region: Region {
                name: String::from("north-america"),
            },
        });
        next_state.set(GameState::Main);
    }
}

fn debug_entity_count(world: &World, mut count: Local<u32>) {
    let entity_count = world.entity_count();
    if *count != entity_count {
        info!(entity_count);
    }
    *count = entity_count;
}

fn cleanup_main(mut commands: Commands) {
    commands.remove_resource::<Dev>();
}
