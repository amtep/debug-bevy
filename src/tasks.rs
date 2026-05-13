use bevy::prelude::*;
use bevy::reflect::Is;
use bevy_common_assets::toml::TomlAssetPlugin;
use indexmap::IndexMap;
use moonshine_save::save::Save;
use serde_derive::Deserialize;

use crate::{
    bases::{Base, BasetypesAsset, BasetypesHandle},
    discoveries::ResearchPoints,
    followers::{Follower, FollowerCount},
    funds::{Expense, FundsAmount, Income},
    modifiers::{Modifier, RecruitmentBy, RecruitmentByOf, RecruitmentOf},
    state::GameState,
    suspicion::{
        IntelligenceSuspicionChange, MediaSuspicionChange, PoliceSuspicionChange,
        ScientificSuspicionChange, SuspicionType,
    },
};

const TASKS_ASSET_PATH: &str = "data/define.tasks.toml";

pub fn plugin(app: &mut App) {
    app.add_plugins(TomlAssetPlugin::<TasksAsset>::new(&["tasks.toml"]))
        .add_systems(OnEnter(GameState::Load), setup_load)
        .add_systems(
            FixedUpdate,
            (recruit, research).run_if(in_state(GameState::Main)),
        )
        .add_observer(on_task_changed::<Task>)
        .add_observer(on_task_changed::<FollowerCount>);
}

#[derive(Deserialize, Asset, TypePath)]
pub struct TasksAsset(pub IndexMap<String, TaskSettings>);

#[derive(Resource)]
pub struct TasksHandle(pub Handle<TasksAsset>);

#[derive(Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "kebab-case")]
pub struct TaskSettings {
    pub follower_types: Vec<String>,
    pub requires_discovery: Option<String>,
    pub income_per_day: Option<(FundsAmount, String)>,
    pub expense_per_day: Option<(FundsAmount, String)>,
    #[serde(default)]
    pub suspicions: IndexMap<SuspicionType, f32>,
    #[serde(default)]
    pub recruit_progress: f64,
    #[serde(default)]
    pub research: usize,
    #[serde(default)]
    pub security: usize,
}

fn setup_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TasksHandle(asset_server.load(TASKS_ASSET_PATH)));
}

/// A component added as a child of a Follower entity, to mark this as a task those followers are doing.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Save)]
#[component(immutable)]
pub struct Task(pub String);

// update both on follower count change and task change
fn on_task_changed<C: Component>(
    insert: On<Insert, C>,
    mut commands: Commands,
    state: Res<State<GameState>>,
    task_handle: Res<TasksHandle>,
    task_assets: Res<Assets<TasksAsset>>,
    tasks: Query<(&ChildOf, &Task)>,
    followers: Query<&FollowerCount>,
    follower_children: Query<&Children, With<FollowerCount>>,
) {
    if *state != GameState::Main {
        return;
    }

    let task_settings = &task_assets.get(task_handle.0.id()).unwrap().0;

    let task_entity = if C::is::<Task>() {
        insert.entity
    } else if let Ok(children) = follower_children.get(insert.entity)
        && let Some(task_entity) = children.iter().find(|e| tasks.contains(*e))
    {
        task_entity
    } else {
        return;
    };

    let (ChildOf(follower_entity), Task(task)) = tasks.get(task_entity).unwrap();

    let Some(settings) = task_settings.get(task) else {
        error!("Task {task} not known");
        return;
    };

    let Ok(count) = followers.get(*follower_entity) else {
        error!("Task without Follower parent");
        return;
    };

    // Handle task income/expense
    if let Some((income, category)) = &settings.income_per_day {
        commands
            .entity(task_entity)
            .insert(Income(*income, category.clone(), count.0));
    } else {
        commands.entity(task_entity).try_remove::<Income>();
    }

    if let Some((expense, category)) = &settings.expense_per_day {
        commands
            .entity(task_entity)
            .insert(Expense(*expense, category.clone(), count.0));
    } else {
        commands.entity(task_entity).try_remove::<Expense>();
    }

    commands.entity(task_entity).try_remove::<(
        IntelligenceSuspicionChange,
        ScientificSuspicionChange,
        PoliceSuspicionChange,
        MediaSuspicionChange,
    )>();

    if !settings.suspicions.is_empty() {
        for (suspicion, amount) in &settings.suspicions {
            #[allow(clippy::cast_possible_truncation)]
            let amount = count.0 as f32 * *amount;
            match *suspicion {
                SuspicionType::Intelligence => commands
                    .entity(task_entity)
                    .insert(IntelligenceSuspicionChange(amount)),
                SuspicionType::Scientific => commands
                    .entity(task_entity)
                    .insert(ScientificSuspicionChange(amount)),
                SuspicionType::Police => commands
                    .entity(task_entity)
                    .insert(PoliceSuspicionChange(amount)),
                SuspicionType::Media => commands
                    .entity(task_entity)
                    .insert(MediaSuspicionChange(amount)),
            };
        }
    }
    // TODO: handle research
}

fn research(
    tasks: Query<(&ChildOf, &Task)>,
    followers: Query<&FollowerCount>,
    task_handle: Res<TasksHandle>,
    task_assets: Res<Assets<TasksAsset>>,
    mut points: ResMut<ResearchPoints>,
) {
    let task_types = &task_assets.get(task_handle.0.id()).unwrap().0;
    for (ChildOf(follower_e), Task(task)) in tasks {
        let Some(task_settings) = task_types.get(task) else {
            error!("Unknown task '{task}'");
            continue;
        };
        if task_settings.research > 0 {
            let Ok(count) = followers.get(*follower_e) else {
                error!("Task without followers");
                continue;
            };
            points.0 += task_settings.research * **count;
        }
    }
}

const NEW_MINION_PROGRESS: f64 = 100.0;

/// A component to track recruitment of new minions.
/// A new minion is spawned if this rolls over the [`NEW_MINION_PROGRESS`] constant.
/// It's locked to 0 if the base is full.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct RecruitMinionProgress(f64);

fn recruit(
    mut commands: Commands,
    tasks: Query<(&ChildOf, &Task)>,
    followers: Query<(&ChildOf, &Follower, &FollowerCount)>,
    mut bases: Query<(&Base, &mut RecruitMinionProgress, &Children)>,
    base_types_handle: Res<BasetypesHandle>,
    base_types_asset: Res<Assets<BasetypesAsset>>,
    task_handle: Res<TasksHandle>,
    task_assets: Res<Assets<TasksAsset>>,
    m_by: Modifier<RecruitmentBy>,
    m_of: Modifier<RecruitmentOf>,
    m_by_of: Modifier<RecruitmentByOf>,
) {
    let base_types = &base_types_asset.get(base_types_handle.0.id()).unwrap().0;
    let task_types = &task_assets.get(task_handle.0.id()).unwrap().0;

    for (ChildOf(follower_e), Task(task)) in tasks {
        let Some(task_settings) = task_types.get(task) else {
            error!("Unknown task '{task}'");
            continue;
        };
        if task_settings.recruit_progress > 0.0 {
            let Ok((ChildOf(base_e), follower, count)) = followers.get(*follower_e) else {
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
                progress.0 = 0.0;
                continue;
            }

            let mut base = task_settings.recruit_progress;
            base = m_by.calc_with(base, |f| f.0 == follower.0);
            base = m_of.calc_with(base, |f| f.0 == "minion");
            base = m_by_of.calc_with(base, |f| f.0 == follower.0 && f.1 == "minion");

            progress.0 += base * **count as f64;
            let mut new_minions = 0;
            #[expect(clippy::while_float)]
            while progress.0 >= NEW_MINION_PROGRESS {
                progress.0 -= NEW_MINION_PROGRESS;
                new_minions += 1;
                if total_followers + new_minions >= base_type_settings.max_pop {
                    progress.0 = 0.0;
                }
            }

            // FIXME: remove hardcode
            for e in children {
                if let Ok((_, Follower(f), count)) = followers.get(*e)
                    && f == "minion"
                {
                    let new_count = FollowerCount(**count + new_minions);
                    commands.entity(*e).insert(new_count);
                }
            }
        }
    }
}
