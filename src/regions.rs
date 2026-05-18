use std::collections::HashMap;

use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use moonshine_save::save::Save;
use serde::Deserialize;

use crate::{
    common::Unlocked,
    discoveries::{DiscoveriesResearched, DiscoveryLearned, DiscoveryVisibility},
    new_game::NewGame,
    state::{GameState, MainSetupSet},
    suspicion::{MediaSuspicion, PoliceSuspicion},
    text::TextKey,
};

const REGIONS_ASSET_PATH: &str = "data/define.regions.toml";

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<RegionsAsset>::new(&["regions.toml"]))
        .add_systems(OnEnter(GameState::Load), setup_load)
        .add_systems(OnExit(GameState::Load), cleanup_load)
        .add_systems(
            OnEnter(GameState::Main),
            (new_game, setup_main).chain().in_set(MainSetupSet::Regions),
        )
        .add_systems(FixedUpdate, reload.run_if(not(in_state(GameState::Load))));
}

#[derive(Deserialize, Asset, TypePath)]
pub struct RegionsAsset(pub HashMap<String, RegionSettings>);

#[derive(Resource)]
pub struct RegionsHandle(pub Handle<RegionsAsset>);

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Component, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct Location {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct RegionSettings {
    #[serde(flatten)]
    pub location: Location,
    pub requires_discovery: Option<String>,
    #[serde(default)]
    pub hidden: bool,
    pub base_plots: HashMap<String, Location>,
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
#[require(Save, DespawnOnExit::<GameState>(GameState::Main))]
pub struct Region {
    pub name: String,
}

impl Region {
    pub fn get_text_key(&self) -> TextKey {
        TextKey::new("region-name").add_arg("region", self.name.clone())
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Save)]
pub struct BasePlot {
    pub name: String,
}

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(RegionsHandle(asset_server.load(REGIONS_ASSET_PATH)));
}

/// Clear the message queue before transitioning out of the `Load` state.
/// This prevents spurious reload detections later.
fn cleanup_load(mut messages: ResMut<Messages<AssetEvent<RegionsAsset>>>) {
    messages.clear();
}

/// Create `Region` and `BasePlot` entities based on the regions asset file.
fn new_game(
    mut commands: Commands,
    regions_handle: Res<RegionsHandle>,
    regions_asset: Res<Assets<RegionsAsset>>,
    new_game: If<Res<NewGame>>,
    mut discoveries: ResMut<DiscoveriesResearched>,
) {
    let regions = &regions_asset.get(regions_handle.0.id()).unwrap().0;

    if let Some(discovery) = regions
        .get(&new_game.region.name)
        .and_then(|r| r.requires_discovery.as_ref())
    {
        info!("Automatically unlocking starting region");
        discoveries.research(
            commands.reborrow(),
            discovery.clone(),
            DiscoveryVisibility::Hidden,
        );
    }

    for (name, settings) in regions {
        if !settings.hidden {
            commands
                .spawn((
                    Region { name: name.clone() },
                    settings.location,
                    PoliceSuspicion(0),
                    MediaSuspicion(0),
                ))
                .insert_if(Unlocked, || {
                    *name == new_game.region.name || settings.requires_discovery.is_none()
                })
                .with_children(|parent| {
                    for (name, location) in &settings.base_plots {
                        parent.spawn((BasePlot { name: name.clone() }, *location));
                    }
                });
        }
    }
}

fn setup_main(mut commands: Commands) {
    commands
        .add_observer(
            |discovery: On<DiscoveryLearned>,
             mut commands: Commands,
             regions: Query<(Entity, &Region, Has<Unlocked>)>,
             regions_handle: Res<RegionsHandle>,
             regions_asset: Res<Assets<RegionsAsset>>| {
                let region_settings = &regions_asset.get(regions_handle.0.id()).unwrap().0;

                for (region_entity, region, unlocked) in regions {
                    if !unlocked
                        && region_settings
                            .get(&region.name)
                            .unwrap()
                            .requires_discovery
                            .as_ref()
                            .unwrap()
                            == &discovery.0
                    {
                        commands.entity(region_entity).insert(Unlocked);
                    }
                }
            },
        )
        .insert(DespawnOnExit(GameState::Main));
}

/// Adjust location settings in the game state entities if the regions asset file has changed.
/// Addition or removal of regions and base plots is ignored.
fn reload(
    mut commands: Commands,
    mut reader: MessageReader<AssetEvent<RegionsAsset>>,
    regions: Query<(Entity, &Region, &Location, &Children)>,
    base_plots: Query<(&BasePlot, &Location)>,
    regions_handle: Res<RegionsHandle>,
    regions_asset: Res<Assets<RegionsAsset>>,
) {
    if !reader.is_empty() {
        info!("regions reloaded");

        let regions_map = &regions_asset.get(regions_handle.0.id()).unwrap().0;

        for (entity, region, location, children) in regions {
            if let Some(settings) = regions_map.get(&region.name) {
                if *location != settings.location {
                    commands.entity(entity).insert(settings.location);
                }

                for child in children {
                    if let Ok((base_plot, old_location)) = base_plots.get(*child)
                        && let Some(location) = settings.base_plots.get(&base_plot.name)
                        && old_location != location
                    {
                        commands.entity(*child).insert(*location);
                    }
                }
            }
        }

        reader.clear();
    }
}
