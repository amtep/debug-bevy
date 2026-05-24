use bevy::{ecs::system::SystemParam, prelude::*};
use either::Either::{Left, Right};
use rand::seq::IndexedRandom;
use serde::Deserialize;

use crate::{
    achievements::AchievedEvent,
    bases::{Base, BasetypeSettings, BasetypesAsset, BasetypesHandle, spawn_base},
    common::Unlocked,
    discoveries::{DiscoveriesResearched, Research, ResearchPoints},
    followers::{Follower, FollowerCount, Recruit},
    funds::{Expense, Funds, FundsAmount, Income},
    modifiers::{ModifierValue, Source, spawn_modifier},
    regions::Region,
    rng::RandomSource,
    suspicion::{SuspicionType, add_suspicion, add_suspicion_change},
    tasks::Task,
    time::{EndDate, GameDate},
};

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Effect {
    Funds(FundsAmount),
    Income {
        amount: Income,
        duration: Option<u32>,
    },
    Expense {
        amount: Expense,
        duration: Option<u32>,
    },
    Secret(i32),
    Research(i32),
    Discovery(String),
    SpawnBase(String),
    DestroyBase,
    Suspicion {
        suspicion: SuspicionType,
        amount: i32,
    },
    SuspicionChange {
        suspicion: SuspicionType,
        amount: f32,
        duration: Option<u32>,
    },
    Recruit {
        follower: String,
        amount: f32,
    },
    Follower {
        name: String,
        count: isize,
    },
    FollowerBusy {
        name: String,
        count: isize,
        duration: u32,
    },
    Modifier(ModifierValue),
    Achievement(String),
}

#[derive(SystemParam)]
pub struct Scopes<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    regions: Query<'w, 's, EntityRef<'static>, (With<Region>, With<Unlocked>)>,
    bases: Query<'w, 's, EntityRef<'static>, With<Base>>,
    followers: Query<'w, 's, EntityRef<'static>, With<Follower>>,
    tasks: Query<'w, 's, EntityRef<'static>, With<Task>>,
    random: Res<'w, RandomSource>,

    base_types_handle: Res<'w, BasetypesHandle>,
    base_types_asset: Res<'w, Assets<BasetypesAsset>>,
}

impl Scopes<'_, '_> {
    pub fn get_region(&self, entity: Option<Entity>) -> Option<EntityRef<'_>> {
        self.get_region_if(entity, |_| true)
    }

    pub fn get_region_if(
        &self,
        entity: Option<Entity>,
        mut pred: impl FnMut(EntityRef) -> bool,
    ) -> Option<EntityRef<'_>> {
        if let Some(entity) = entity {
            if let Ok(entity_ref) = self.regions.get(entity)
                && pred(entity_ref)
            {
                Some(entity_ref)
            } else {
                None
            }
        } else {
            let entity_refs = self.regions.iter().filter(|e| pred(*e)).collect::<Vec<_>>();
            entity_refs.choose(&mut self.random.rng()).copied()
        }
    }

    pub fn get_base(&self, entity: Option<Entity>) -> Option<EntityRef<'_>> {
        self.get_base_if(entity, |_| true)
    }

    pub fn get_base_if(
        &self,
        entity: Option<Entity>,
        mut pred: impl FnMut(EntityRef) -> bool,
    ) -> Option<EntityRef<'_>> {
        if let Some(entity) = entity {
            if let Ok(entity_ref) = self.bases.get(entity) {
                pred(entity_ref).then_some(entity_ref)
            } else if self.regions.contains(entity) {
                let entity_refs = self
                    .children
                    .iter_descendants(entity)
                    .filter_map(|e| self.bases.get(e).ok())
                    .filter(|e| pred(*e))
                    .collect::<Vec<_>>();
                entity_refs.choose(&mut self.random.rng()).copied()
            } else {
                None
            }
        } else {
            let entity_refs = self.bases.iter().filter(|e| pred(*e)).collect::<Vec<_>>();
            entity_refs.choose(&mut self.random.rng()).copied()
        }
    }

    pub fn get_base_type_settings(&self, entity: Entity) -> &BasetypeSettings {
        let base = &self.bases.get(entity).unwrap().get::<Base>().unwrap().0;
        let base_type_settings = &self
            .base_types_asset
            .get(self.base_types_handle.0.id())
            .unwrap()
            .0;
        base_type_settings.get(base).unwrap()
    }

    pub fn get_followers(&self, entity: Option<Entity>) -> impl Iterator<Item = EntityRef<'_>> {
        if let Some(entity) = entity {
            Left(
                self.children
                    .iter_descendants(entity)
                    .filter_map(|e| self.followers.get(e).ok()),
            )
        } else {
            Right(self.followers.iter())
        }
    }

    pub fn get_follower(&self, entity: Option<Entity>) -> Option<EntityRef<'_>> {
        self.get_follower_if(entity, |_| true)
    }

    pub fn get_follower_if(
        &self,
        entity: Option<Entity>,
        mut pred: impl FnMut(EntityRef) -> bool,
    ) -> Option<EntityRef<'_>> {
        if let Some(entity) = entity {
            if self.regions.contains(entity) || self.bases.contains(entity) {
                let entities = self
                    .children
                    .iter_descendants(entity)
                    .filter_map(|e| self.followers.get(e).ok())
                    .filter(|e| pred(*e))
                    .collect::<Vec<_>>();
                entities.choose(&mut self.random.rng()).copied()
            } else if let Ok(entity_ref) = self.followers.get(entity) {
                pred(entity_ref).then_some(entity_ref)
            } else {
                None
            }
        } else {
            let entities = self
                .followers
                .iter()
                .filter(|e| pred(*e))
                .collect::<Vec<_>>();
            entities.choose(&mut self.random.rng()).copied()
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
pub fn apply_effect(
    In((entity, count, effect, source)): In<(Option<Entity>, Option<usize>, Effect, Source)>,
    mut commands: Commands,
    mut funds: ResMut<Funds>,
    mut secrets: ResMut<ResearchPoints>,
    mut discoveries: ResMut<DiscoveriesResearched>,
    date: Res<GameDate>,
    scopes: Scopes,
) {
    macro_rules! entity_commands {
        ($duration: expr) => {
            if let Some(duration) = $duration {
                let end_date = EndDate::new(date.0, duration);
                if let Some(entity) = entity {
                    commands.spawn((ChildOf(entity), end_date))
                } else {
                    commands.spawn(end_date)
                }
            } else {
                entity_commands!()
            }
        };
        () => {
            if let Some(entity) = entity {
                commands.spawn(ChildOf(entity))
            } else {
                commands.spawn_empty()
            }
        };
    }

    let count = count.unwrap_or(1);

    match effect {
        Effect::Funds(amount) => funds.0 += amount * count as FundsAmount,
        Effect::Income {
            mut amount,
            duration,
        } => {
            amount.2 *= count;
            entity_commands!(duration).insert(amount);
        }
        Effect::Expense {
            mut amount,
            duration,
        } => {
            amount.2 *= count;
            entity_commands!(duration).insert(amount);
        }
        Effect::Secret(s) => {
            secrets.0 = secrets.0.saturating_add_signed(s * count as i32);
        }
        Effect::Research(amount) => {
            entity_commands!().insert(Research(amount * count as i32));
        }
        Effect::Discovery(d) => discoveries.research(
            commands.reborrow(),
            d,
            crate::discoveries::DiscoveryVisibility::Shown,
        ),
        Effect::SpawnBase(base) => {
            if let Some(entity_ref) = scopes.get_region(entity) {
                commands.run_system_cached_with(spawn_base, (entity_ref.id(), base));
            }
        }
        Effect::DestroyBase => {
            if let Some(entity_ref) = scopes.get_base(entity) {
                commands.entity(entity_ref.id()).despawn();
            }
        }
        Effect::Suspicion { suspicion, amount } => {
            commands
                .run_system_cached_with(add_suspicion, (entity, suspicion, amount * count as i32));
        }
        Effect::SuspicionChange {
            suspicion,
            amount,
            duration,
        } => {
            add_suspicion_change(
                &mut entity_commands!(duration),
                suspicion,
                amount * count as f32,
            );
        }
        Effect::Recruit { follower, amount } => {
            entity_commands!().insert(Recruit(follower, amount * count as f32));
        }
        Effect::Follower { name, count } => {
            if count == 0 {
                return;
            }

            if let Some(entity_ref) = scopes.get_follower_if(entity, |entity_ref| {
                if *name != entity_ref.get::<Follower>().unwrap().0 {
                    return false;
                }
                #[allow(clippy::cast_sign_loss)]
                if count.is_positive() {
                    let base = entity_ref.get::<ChildOf>().unwrap().0;
                    let max_follower_count = scopes.get_base_type_settings(base).max_follower_count;
                    let follower_counts = scopes
                        .get_followers(Some(base))
                        .map(|e| e.get::<FollowerCount>().unwrap().0)
                        .sum::<usize>();
                    follower_counts + count as usize <= max_follower_count
                } else {
                    count.unsigned_abs() <= entity_ref.get::<FollowerCount>().unwrap().0
                }
            }) {
                let mut follower_count = *entity_ref.get::<FollowerCount>().unwrap();
                follower_count.0 = follower_count.0.saturating_add_signed(count);
                commands.entity(entity_ref.id()).insert(follower_count);
            }
            // TODO: allow adding/removing followers less than the count too.
        }
        Effect::FollowerBusy {
            name,
            count,
            duration,
        } => todo!(),
        Effect::Modifier(modifier_value) => {
            spawn_modifier(
                commands.reborrow(),
                entity,
                Some(date.0),
                &modifier_value,
                source,
            );
        }
        Effect::Achievement(name) => {
            commands.trigger(AchievedEvent { achievement: name });
        }
    }
}
