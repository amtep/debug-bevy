use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use indexmap::IndexMap;
use moonshine_save::save::Save;
use rand::{RngExt, seq::IndexedRandom};
use serde_derive::Deserialize;
use strum::IntoEnumIterator;

use crate::{
    constants::NEW_MINION_PROGRESS,
    followers::{
        Follower, FollowerCount, FollowersAsset, FollowersHandle, GeneralFollowerSettings,
    },
    funds::{Expense, ExpenseCategory, Funds, FundsAmount},
    main_menu::NewGame,
    regions::{BasePlot, Region},
    rng::RandomSource,
    state::{GameState, MainSetupSet},
    tasks::{DEFAULT_TASK, Task, TasksAsset, TasksHandle},
};

const BASETYPES_ASSET_PATH: &str = "data/define.basetypes.toml";

const DEFAULT_BASETYPE: &str = "apartment";

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<BasetypesAsset>::new(&["basetypes.toml"]))
        .add_systems(OnEnter(GameState::Load), setup_load)
        .add_systems(
            OnEnter(GameState::Main),
            new_game
                .run_if(resource_exists::<NewGame>)
                .in_set(MainSetupSet::Bases),
        )
        .add_systems(FixedUpdate, recruitment);
}

#[derive(Deserialize, Asset, TypePath)]
pub struct BasetypesAsset(pub IndexMap<String, BasetypeSettings>);

#[derive(Resource)]
pub struct BasetypesHandle(pub Handle<BasetypesAsset>);

#[derive(Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "kebab-case")]
pub struct BasetypeSettings {
    pub max_pop: usize,
    pub cost_per_day: FundsAmount,
    pub initial_cost: FundsAmount,
    pub police_suspicion: u32,
    pub media_suspicion: u32,
    #[serde(default)]
    pub regions: Vec<String>,
    #[serde(default)]
    pub hidden: bool,
    pub color: String,
}

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BasetypesHandle(asset_server.load(BASETYPES_ASSET_PATH)));
}

/// A marker component for bases in the game state.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
#[require(Save, RecruitMinionProgress)]
pub struct Base(pub String);

/// A component to track recruitment of new minions.
/// A new minion is spawned if this rolls over the [`NEW_MINION_PROGRESS`] constant.
/// It's locked to 0 if the base is full.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct RecruitMinionProgress(usize);

fn new_game(
    commands: Commands,
    base_types_handle: Res<BasetypesHandle>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
    mut random_source: ResMut<RandomSource>,
    base_plots: Query<Entity, With<BasePlot>>,
    followers_handle: Res<FollowersHandle>,
    followers_asset: Res<Assets<FollowersAsset>>,
) {
    info!("Creating starting base");
    let i = random_source.0.random_range(0..base_plots.count());
    let base_plot = base_plots.iter().nth(i).unwrap();

    // TODO: don't hardcode this string
    let apartment = &base_types_asset.get(base_types_handle.0.id()).unwrap().0[DEFAULT_BASETYPE];
    let general = &followers_asset.get(followers_handle.0.id()).unwrap().0["general"];
    spawn_base_inner(
        commands,
        base_plot,
        DEFAULT_BASETYPE.into(),
        apartment,
        general,
        None,
    );
}

#[expect(clippy::trivially_copy_pass_by_ref)]
fn spawn_base_inner(
    mut commands: Commands,
    base_plot: Entity,
    base_type: String,
    base_type_settings: &BasetypeSettings,
    follower_settings: &GeneralFollowerSettings,
    funds: Option<ResMut<Funds>>,
) {
    let base = commands
        .spawn((
            Base(base_type),
            Expense(base_type_settings.cost_per_day, ExpenseCategory::Bases, 1),
            ChildOf(base_plot),
        ))
        .id();
    if let Some(mut funds) = funds {
        funds.0 -= base_type_settings.initial_cost;
    }
    let cost = follower_settings.cost_per_day;
    for follower in Follower::iter() {
        let follower_e = commands
            .spawn((
                ChildOf(base),
                follower,
                Expense(cost, ExpenseCategory::Followers, 0),
            ))
            .insert(FollowerCount(0))
            .id();
        if follower != Follower::Goon {
            commands.spawn((Task(String::from(DEFAULT_TASK)), ChildOf(follower_e)));
        }
    }
}

pub fn spawn_base(
    commands: Commands,
    funds: ResMut<Funds>,
    region: Entity,
    regions: Query<&Children, With<Region>>,
    base_type: String,
    base_plots: Query<Has<Children>, With<BasePlot>>,
    base_types_handle: Res<BasetypesHandle>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
    followers_handle: Res<FollowersHandle>,
    followers_asset: Res<Assets<FollowersAsset>>,
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

    if funds.0 < base_type_settings.initial_cost {
        warn!(
            "not enough funds to acquire {base_type} base ({} < {})",
            funds.0, base_type_settings.initial_cost
        );
        return;
    }

    let general = &followers_asset.get(followers_handle.0.id()).unwrap().0["general"];

    spawn_base_inner(
        commands,
        *base_plot,
        base_type,
        base_type_settings,
        general,
        Some(funds),
    );
}

fn recruitment(
    mut commands: Commands,
    tasks: Query<(&ChildOf, &Task)>,
    followers: Query<(&ChildOf, &Follower, &FollowerCount)>,
    mut bases: Query<(&Base, &mut RecruitMinionProgress, &Children)>,
    base_types_handle: Res<BasetypesHandle>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
    task_handle: Res<TasksHandle>,
    task_assets: Res<Assets<TasksAsset>>,
) {
    let base_types = &base_types_asset.get(base_types_handle.0.id()).unwrap().0;
    let task_types = &task_assets.get(task_handle.0.id()).unwrap().0;

    for (ChildOf(follower_e), Task(task)) in tasks {
        let Some(task_settings) = task_types.get(task) else {
            error!("Unknown task '{task}'");
            continue;
        };
        if task_settings.recruit_progress > 0 {
            let Ok((ChildOf(base_e), _, count)) = followers.get(*follower_e) else {
                error!("Task without followers");
                continue;
            };
            let Ok((Base(basetype), mut progress, children)) = bases.get_mut(*base_e) else {
                error!("Followers without base");
                continue;
            };
            let Some(base_type_settings) = base_types.get(basetype) else {
                error!("Unknown basetype '{basetype}'");
                continue;
            };
            let total_followers: usize = children
                .iter()
                .map(|e| {
                    if let Ok((_, _, count)) = followers.get(e) {
                        **count
                    } else {
                        0
                    }
                })
                .sum();
            if total_followers >= base_type_settings.max_pop {
                progress.0 = 0;
                continue;
            }
            progress.0 += task_settings.recruit_progress * **count;
            let mut new_minions = 0;
            while progress.0 >= NEW_MINION_PROGRESS {
                progress.0 -= NEW_MINION_PROGRESS;
                new_minions += 1;
                if total_followers + new_minions >= base_type_settings.max_pop {
                    progress.0 = 0;
                }
            }

            for e in children {
                if let Ok((_, Follower::Minion, count)) = followers.get(*e) {
                    let new_count = FollowerCount(**count + new_minions);
                    commands.entity(*e).insert(new_count);
                }
            }
        }
    }
}
