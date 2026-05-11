use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use indexmap::IndexMap;
use moonshine_save::save::Save;
use serde::Deserialize;

use crate::{
    bases::Base,
    funds::{Expense, FundsAmount},
    new_game::NewGame,
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
pub struct FollowersAsset(pub IndexMap<String, FollowerSettings>);

#[derive(Resource)]
pub struct FollowersHandle(pub Handle<FollowersAsset>);

/// These are the general settings for all follower types.
/// Once there are also specific follower settings, there
/// will need to be an enum to distinguish them.
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub struct FollowerSettings {
    pub cost_per_day: FundsAmount,
    pub symbol: char,
}

#[derive(Component, Reflect, Clone, Deserialize, Debug, Deref, PartialEq, Eq)]
#[reflect(Component)]
#[require(Save)]
pub struct Follower(pub String);

#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deref, DerefMut, Reflect,
)]
#[reflect(Component)]
#[component(immutable)]
pub struct FollowerCount(pub usize);

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(FollowersHandle(asset_server.load(FOLLOWERS_ASSET_PATH)));
}

/// Create the starting priest for the cult.
fn new_game(
    mut commands: Commands,
    base: Single<&Children, With<Base>>,
    mut followers: Query<(&Follower, &FollowerCount, &Expense)>,
    new_game: Res<NewGame>,
) {
    info!("Creating starting follower");

    let starting_followers = &new_game.difficulty.starting_followers;

    for child in base.iter() {
        if let Ok((follower, follower_count, expense)) = followers.get_mut(child)
            && let Some(count) = starting_followers.get(&follower.0)
        {
            let mut follower_count = *follower_count;
            follower_count.0 = *count;
            let mut expense = expense.clone();
            expense.2 = *count;
            commands.entity(child).insert(follower_count);
            commands.entity(child).insert(expense);
        }
    }
}
