use bevy::prelude::*;

use crate::{
    funds::FundsAmount,
    state::{GameState, MainSetupSet},
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::Main),
        remove_new_game.in_set(MainSetupSet::Late),
    );
}

#[derive(Resource, Clone)]
pub struct NewGame {
    pub starting_fund: FundsAmount,
    pub starting_followers: &'static [(&'static str, usize)],
}

impl NewGame {
    pub const EASY: NewGame = NewGame {
        starting_fund: 50000,
        starting_followers: &[("priest", 1), ("minion", 1)],
    };

    pub const NORMAL: NewGame = NewGame {
        starting_fund: 20000,
        starting_followers: &[("priest", 1)],
    };

    pub const HARD: NewGame = NewGame {
        starting_fund: 10000,
        starting_followers: &[("priest", 1)],
    };
}

fn remove_new_game(mut commands: Commands) {
    commands.remove_resource::<NewGame>();
}
