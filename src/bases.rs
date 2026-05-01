use std::collections::HashMap;

use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use moonshine_save::save::Save;
use rand::{RngExt, seq::IndexedRandom};
use serde_derive::Deserialize;

use crate::{
    funds::{Expense, ExpenseCategory, Funds, FundsAmount},
    main_menu::{LoadedGame, NewGame},
    regions::{BasePlot, Region},
    rng::RandomSource,
    state::{GameState, MainSetupSet},
};

const BASETYPES_ASSET_PATH: &str = "data/define.basetypes.toml";

const DEFAULT_BASETYPE: &str = "apartment";

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<BasetypesAsset>::new(&["basetypes.toml"]))
        .add_systems(OnEnter(GameState::Load), setup_load)
        .add_systems(
            OnEnter(GameState::Main),
            (
                new_game.run_if(resource_exists::<NewGame>),
                loaded_game.run_if(resource_exists::<LoadedGame>),
            )
                .in_set(MainSetupSet::Bases),
        );
}

#[derive(Deserialize, Asset, TypePath)]
pub struct BasetypesAsset(pub HashMap<String, BasetypeSettings>);

#[derive(Resource)]
pub struct BasetypesHandle(pub Handle<BasetypesAsset>);

#[derive(Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "kebab-case")]
pub struct BasetypeSettings {
    pub max_pop: isize,
    pub cost_per_day: FundsAmount,
    pub initial_cost: FundsAmount,
    pub police_suspicion: u32,
    pub media_suspicion: u32,
    #[serde(default)]
    pub regions: Vec<String>,
}

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BasetypesHandle(asset_server.load(BASETYPES_ASSET_PATH)));
}

/// A marker component for bases in the game state.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
#[require(Save)]
pub struct Base(pub String);

fn new_game(
    mut commands: Commands,
    base_types_handle: Res<BasetypesHandle>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
    mut random_source: ResMut<RandomSource>,
    base_plots: Query<Entity, With<BasePlot>>,
) {
    info!("Creating starting base");
    let i = random_source.0.random_range(0..base_plots.count());
    let base_plot = base_plots.iter().nth(i).unwrap();

    let base_types = &base_types_asset.get(base_types_handle.0.id()).unwrap().0;
    // TODO: don't hardcode this string
    let apartment = base_types.get(DEFAULT_BASETYPE).unwrap();
    commands.entity(base_plot).with_child((
        Base(DEFAULT_BASETYPE.into()),
        Expense(apartment.cost_per_day, ExpenseCategory::Bases),
    ));
}

pub fn spawn_base(
    mut commands: Commands,
    mut funds: ResMut<Funds>,
    region: Entity,
    regions: Query<&Children, With<Region>>,
    base_type: String,
    base_plots: Query<Has<Children>, With<BasePlot>>,
    base_types_handle: Res<BasetypesHandle>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
    mut random_source: ResMut<RandomSource>,
) {
    let vacant_base_plots: Vec<Entity> = regions
        .get(region)
        .unwrap()
        .iter()
        .filter(|base_plot| base_plots.get(*base_plot) == Ok(false))
        .collect();
    let Some(base_plot) = vacant_base_plots.choose(&mut random_source.0) else {
        warn!("expected at least one vacant base plot");
        return;
    };

    let base_types = &base_types_asset.get(base_types_handle.0.id()).unwrap().0;
    let base_type_settings = base_types.get(&base_type).unwrap();

    commands.entity(*base_plot).with_child((
        Base(base_type),
        Expense(base_type_settings.cost_per_day, ExpenseCategory::Bases),
    ));
    // Assume initial funds are sufficient.
    funds.0 -= base_type_settings.initial_cost;
}

fn loaded_game(mut commands: Commands, bases: Query<(Entity, &Base)>) {
    for (entity, base) in bases {
        // Remove and re-insert the Base in order to trigger the Add observer
        // that builds the base UI.
        commands
            .entity(entity)
            .remove::<Base>()
            .insert(base.clone());
    }
}
