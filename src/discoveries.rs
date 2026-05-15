use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use indexmap::IndexMap;
use moonshine_save::save::Save;
use serde_derive::Deserialize;

use crate::{
    funds::{Funds, FundsAmount},
    modifiers::{Source, spawn_modifier},
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
    )
    .add_systems(FixedUpdate, research.run_if(in_state(GameState::Main)));
}

#[derive(Deserialize, Asset, TypePath)]
pub struct DiscoveriesAsset(pub IndexMap<String, DiscoverySettings>);

#[derive(Resource)]
pub struct DiscoveriesHandle(pub Handle<DiscoveriesAsset>);

#[derive(Clone, Copy, PartialEq, Eq, Reflect)]
pub enum DiscoveryVisibility {
    Hidden,
    Shown,
}

/// The discoveries that have been discovered by the player, ordered by when they were discovered.
#[derive(Resource, Default, Reflect, Deref)]
#[reflect(Resource)]
pub struct DiscoveriesResearched(pub IndexMap<String, DiscoveryVisibility>);

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ResearchPoints(pub u32);

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Save)]
pub struct Research(pub u32);

#[derive(Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "kebab-case")]
pub struct DiscoverySettings {
    pub research_cost: u32,
    #[serde(default)]
    pub funds_cost: FundsAmount,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub modifiers: IndexMap<String, f64>,
}

#[derive(Resource)]
pub struct DiscoverySelected(pub String, pub FundsAmount, pub u32);

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
    mut commands: Commands,
    mut funds: ResMut<Funds>,
    mut secrets: ResMut<ResearchPoints>,
    mut discoveries_researched: ResMut<DiscoveriesResearched>,
    discovery_selected: Res<DiscoverySelected>,
    discoveries_handle: Res<DiscoveriesHandle>,
    discoveries_assets: Res<Assets<DiscoveriesAsset>>,
) {
    let discoveries = &discoveries_assets.get(discoveries_handle.0.id()).unwrap().0;
    let discovery = discoveries.get(&discovery_selected.0).unwrap();

    funds.0 -= discovery_selected.1;
    secrets.0 -= discovery_selected.2;

    for (modifier, value) in &discovery.modifiers {
        spawn_modifier(
            commands.reborrow(),
            modifier,
            *value,
            Source::Discovery(discovery_selected.0.clone()),
        );
    }

    discoveries_researched
        .0
        .insert(discovery_selected.0.clone(), DiscoveryVisibility::Shown);
}

fn research(researches: Query<&Research>, mut points: ResMut<ResearchPoints>) {
    points.0 += researches.iter().map(|r| r.0).sum::<u32>();
}
