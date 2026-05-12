use bevy::{platform::collections::HashSet, prelude::*};
use bevy_common_assets::toml::TomlAssetPlugin;
use indexmap::IndexMap;
use serde_derive::Deserialize;

use crate::{
    funds::{Funds, FundsAmount},
    new_game::NewGame,
    state::{GameState, MainSetupSet},
};

const DISCOVERIES_ASSET_PATH: &str = "data/define.discoveries.toml";

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<DiscoveriesAsset>::new(&[
        "discoveries.toml",
    ]))
    .add_systems(OnEnter(GameState::Load), setup_load)
    .add_systems(
        OnEnter(GameState::Main),
        new_game.in_set(MainSetupSet::Default),
    );
}

#[derive(Deserialize, Asset, TypePath)]
pub struct DiscoveriesAsset(pub IndexMap<String, DiscoverySettings>);

#[derive(Resource)]
pub struct DiscoveriesHandle(pub Handle<DiscoveriesAsset>);

#[derive(Resource, Default, Reflect, Deref)]
#[reflect(Resource)]
pub struct DiscoveriesResearched(pub HashSet<String>);

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ResearchPoints(pub usize);

#[derive(Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "kebab-case")]
pub struct DiscoverySettings {
    pub research_cost: usize,
    #[serde(default)]
    pub funds_cost: FundsAmount,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Resource)]
pub struct DiscoverySelected(pub String, pub FundsAmount, pub usize);

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(DiscoveriesHandle(asset_server.load(DISCOVERIES_ASSET_PATH)));
}

fn new_game(mut commands: Commands, _: If<Res<NewGame>>) {
    // This has to be insert_resource not init_resource, to override whatever may
    // have been there from a previous game.
    commands.insert_resource(DiscoveriesResearched::default());
    commands.insert_resource(ResearchPoints(0));
}

pub fn learn_new_discovery(
    mut funds: ResMut<Funds>,
    mut secrets: ResMut<ResearchPoints>,
    mut discoveries_researched: ResMut<DiscoveriesResearched>,
    discovery_selected: Res<DiscoverySelected>,
) {
    funds.0 -= discovery_selected.1;
    secrets.0 -= discovery_selected.2;
    discoveries_researched
        .0
        .insert(discovery_selected.0.clone());
}
