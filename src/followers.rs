use std::collections::HashMap;

use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use moonshine_save::save::Save;
use serde::Deserialize;
use strum::{EnumIter, IntoStaticStr};

use crate::{
    bases::Base,
    funds::{Expense, FundsAmount},
    main_menu::{LoadedGame, NewGame},
    state::{GameState, MainSetupSet},
};

const FOLLOWERS_ASSET_PATH: &str = "data/define.followers.toml";

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<FollowersAsset>::new(&["followers.toml"]))
        .add_systems(OnEnter(GameState::Load), setup_load)
        .add_systems(
            OnEnter(GameState::Main),
            (
                new_game.run_if(resource_exists::<NewGame>),
                loaded_game.run_if(resource_exists::<LoadedGame>),
            )
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

#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, IntoStaticStr, Reflect,
)]
#[reflect(Component)]
#[require(Save)]
#[strum(serialize_all = "kebab-case")]
pub enum Follower {
    Priest,
    Goon,
    Minion,
}

impl Follower {
    pub fn to_symbol(self) -> char {
        match self {
            Follower::Priest => '♀',
            Follower::Goon => '♁',
            Follower::Minion => '♂',
        }
    }
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
    mut commands: Commands,
    base: Single<&Children, With<Base>>,
    mut followers: Query<(&Follower, &FollowerCount, &mut Expense)>,
) {
    info!("Creating starting priest");

    // Generally we should check whether the base has room
    // for another follower, but this is a new game and it
    // will be empty.

    for child in base.iter() {
        if let Ok((follower, follower_count, mut expense)) = followers.get_mut(child)
            && *follower == Follower::Priest
        {
            let mut follower_count = *follower_count;
            *follower_count += 1;
            commands.entity(child).insert(follower_count);
            expense.2 += 1;
        }
    }
}

fn loaded_game(mut commands: Commands, follower_counts: Query<(Entity, &FollowerCount)>) {
    for (entity, follower_count) in follower_counts {
        // Remove and re-insert the Base in order to trigger the Add observer
        // that builds the base UI.
        commands
            .entity(entity)
            .remove::<FollowerCount>()
            .insert(*follower_count);
    }
}
