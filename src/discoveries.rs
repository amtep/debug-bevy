use bevy::{platform::collections::HashSet, prelude::*};
use bevy_common_assets::toml::TomlAssetPlugin;
use indexmap::IndexMap;
use serde_derive::Deserialize;

use crate::{funds::FundsAmount, state::GameState};

const DISCOVERIES_ASSET_PATH: &str = "data/define.discoveries.toml";

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<DiscoveriesAsset>::new(&[
        "discoveries.toml",
    ]))
    .init_resource::<DiscoveriesResearched>()
    .init_resource::<ResearchPoints>()
    .add_systems(OnEnter(GameState::Load), setup_load);
}

#[derive(Deserialize, Asset, TypePath)]
struct DiscoveriesAsset(IndexMap<String, DiscoverySettings>);

#[derive(Resource)]
struct DiscoveriesHandle(Handle<DiscoveriesAsset>);

#[derive(Resource, Default, Reflect, Deref)]
#[reflect(Resource)]
pub struct DiscoveriesResearched(pub HashSet<String>);

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ResearchPoints(pub usize);

#[derive(Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "kebab-case")]
pub struct DiscoverySettings {
    research_cost: usize,
    #[serde(default)]
    funds_cost: FundsAmount,
    #[serde(default)]
    requires: Vec<String>,
}

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(DiscoveriesHandle(asset_server.load(DISCOVERIES_ASSET_PATH)));
}
