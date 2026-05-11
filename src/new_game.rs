use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use indexmap::IndexMap;
use serde::Deserialize;

use crate::{
    funds::FundsAmount,
    state::{GameState, MainSetupSet},
};

const DIFFICULTIES_ASSET_PATH: &str = "data/define.difficulties.toml";

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<DifficultiesAsset>::new(&[
        "difficulties.toml",
    ]))
    .add_systems(OnEnter(GameState::Load), setup_load)
    .add_systems(
        OnEnter(GameState::Main),
        remove_new_game.in_set(MainSetupSet::Late),
    );
}

#[derive(Deserialize, Asset, TypePath)]
pub struct DifficultiesAsset(pub IndexMap<String, DifficultySettings>);

#[derive(Resource)]
pub struct DifficultiesHandle(pub Handle<DifficultiesAsset>);

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(DifficultiesHandle(
        asset_server.load(DIFFICULTIES_ASSET_PATH),
    ));
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DifficultySettings {
    // one and only one should exist
    #[serde(default)]
    pub default: bool,
    pub starting_funds: FundsAmount,
    pub starting_followers: IndexMap<String, usize>,
}

#[derive(Resource, Clone)]
pub struct NewGame {
    pub difficulty: DifficultySettings,
}

fn remove_new_game(mut commands: Commands) {
    commands.remove_resource::<NewGame>();
}
