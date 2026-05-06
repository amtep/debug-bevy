use bevy::prelude::*;

use crate::{
    common::{CultName, CultSymbol, Dev},
    funds::Funds,
    main_menu::NewGame,
    state::GameState,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, listen_dev_keys.run_if(in_state(GameState::Main)));
    app.add_systems(
        Update,
        listen_dev_keys_main_menu.run_if(in_state(GameState::MainMenu)),
    );
}

#[cfg(feature = "dev")]
fn listen_dev_keys(keys: Res<ButtonInput<KeyCode>>, mut funds: ResMut<Funds>) {
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
}

#[cfg(feature = "dev")]
fn listen_dev_keys_main_menu(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::KeyG)
        && keys.pressed(KeyCode::AltLeft)
        && *game_state.get() == GameState::MainMenu
    {
        // Alt + G
        commands.insert_resource(CultName("DEV".into()));
        commands.insert_resource(CultSymbol('D'));
        commands.init_resource::<Dev>();
        commands.init_resource::<NewGame>();
        next_state.set(GameState::Main);
    }
}
