use std::collections::HashMap;

use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use moonshine_save::save::Save;
use rand::RngExt;
use serde::Deserialize;

use crate::{
    bases::Base,
    funds::{Expense, FundsAmount},
    main_menu::NewGame,
    rng::RandomSource,
    state::{GameState, MainSetupSet},
};

const FOLLOWERS_ASSET_PATH: &str = "data/define.followers.toml";

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<FollowersAsset>::new(&["followers.toml"]))
        .add_systems(OnEnter(GameState::Load), setup_load)
        .add_systems(
            OnEnter(GameState::Main),
            new_game
                .run_if(resource_exists::<NewGame>)
                .in_set(MainSetupSet::Followers),
        );
}

#[derive(Deserialize, Asset, TypePath)]
pub struct FollowersAsset(pub HashMap<String, GeneralFollowerSettings>);

#[derive(Resource)]
pub struct FollowersHandle(pub Handle<FollowersAsset>);

/// These are the general settings for all follower types.
/// Once there are also specific follower settings, there
/// will need to be an enum to distinguish them.
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub struct GeneralFollowerSettings {
    pub cost_per_day: FundsAmount,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect)]
#[reflect(Component)]
#[require(Save)]
pub enum Follower {
    Priest,
    Goon,
    Minion,
}

#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deref, DerefMut, Reflect,
)]
#[reflect(Component)]
pub struct FollowerCount(pub usize);

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(FollowersHandle(asset_server.load(FOLLOWERS_ASSET_PATH)));
}

/// Create the starting priest for the cult.
fn new_game(
    bases: Query<Entity, With<Base>>,
    followers: Query<(&ChildOf, &Follower, &mut FollowerCount, &mut Expense)>,
    mut random_source: ResMut<RandomSource>,
) {
    info!("Creating starting priest");
    let i = random_source.0.random_range(0..bases.count());
    let base = bases.iter().nth(i).unwrap();

    // Generally we should check whether the base has room
    // for another follower, but this is a new game and it
    // will be empty.

    for (ChildOf(parent), follower, mut count, mut expense) in followers {
        if *parent != base || *follower != Follower::Priest {
            continue;
        }
        count.0 += 1;
        expense.2 = count.0;
    }
}
