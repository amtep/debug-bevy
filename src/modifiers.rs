use bevy::{ecs::system::SystemParam, prelude::*};

use moonshine_save::save::Save;

use crate::state::GameState;

/// The kind of modifier: `_add` or `_mult`.
/// `Add`s are performed before `Multiply`s.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Save, DespawnOnExit::<GameState>(GameState::Main))]
pub enum Operation {
    Add,
    Multiply,
}

/// The bonus or penalty to use with an [`Operation`].
/// For `Multiply`, the value is 1-based: A `Value(1.0)` will leave the base value unchanged.
/// So a 20% bonus will be encoded as `Value(1.2)`.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Value(pub f64);

/// A record of where a modifier came from.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub enum Source {
    Difficulty(String),
    Discovery(String),
}

/// A modifier to recruitment progress.
/// The `String` is a follower type that gets this modifier when recruiting.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RecruitmentBy(pub String);

/// A modifier to recruitment progress.
/// The `String` is a follower type that this modifier applies to when being recruited.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RecruitmentOf(pub String);

/// A modifier to recruitment progress.
/// The first `String` is a follower type that gets this modifier when recruiting.
/// The second `String` is a follower type that this modifier applies to when being recruited.
/// Both conditions must be satisfied for the modifier to apply.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RecruitmentByOf(pub String, pub String);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct IncomeModifier;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ExpenseModifier;

/// A system parameter that can be used to calculate modifiers to a base value.
/// Use it like `m: Modifier<RecruitmentBy>`, then `m.calc_with(base, |r| r.0 == follower)`
/// where `base` and `follower` are provided by the system that uses this parameter.
#[derive(SystemParam)]
pub struct Modifier<'w, 's, C>
where
    C: Component,
{
    child_ofs: Query<'w, 's, &'static ChildOf>,
    modifiers: Query<
        'w,
        's,
        (
            &'static C,
            &'static Operation,
            &'static Value,
            Option<&'static ChildOf>,
        ),
    >,
}

impl<C: Component> Modifier<'_, '_, C> {
    #[inline]
    fn entities(&self, entity: Entity) -> Vec<Entity> {
        std::iter::once(entity)
            .chain(self.child_ofs.iter_ancestors(entity))
            .collect()
    }

    /// Apply all modifiers of category `C` to the `base` value and return the result.
    pub fn calc(&self, base: f64, entity: Entity) -> f64 {
        self.calc_with(base, entity, |_| true)
    }

    /// Apply all modifiers of category `C` that match the filter `f(&C)` to the `base` value and return the result.
    pub fn calc_with<F>(&self, mut base: f64, entity: Entity, f: F) -> f64
    where
        F: Fn(&C) -> bool,
    {
        let mut factor: f64 = 1.0;

        let entities = self.entities(entity);

        for (component, operation, value, parent) in &self.modifiers {
            if parent.is_none_or(|p| entities.contains(&p.0) && f(component)) {
                match operation {
                    Operation::Add => {
                        base += value.0;
                    }
                    Operation::Multiply => {
                        factor *= value.0;
                    }
                }
            }
        }

        base * factor
    }

    #[expect(dead_code)]
    pub fn calc_add(&self, entity: Entity) -> f64 {
        self.calc_add_with(entity, |_| true)
    }

    pub fn calc_add_with<F>(&self, entity: Entity, f: F) -> f64
    where
        F: Fn(&C) -> bool,
    {
        let mut base = 0.0;

        let entities = self.entities(entity);

        for (component, operation, value, parent) in &self.modifiers {
            if parent.is_none_or(|p| entities.contains(&p.0) && f(component)) {
                match operation {
                    Operation::Add => {
                        base += value.0;
                    }
                    Operation::Multiply => (),
                }
            }
        }
        base
    }

    pub fn calc_mult(&self, base: f64, entity: Entity) -> f64 {
        self.calc_mult_with(base, entity, |_| true)
    }

    pub fn calc_mult_with<F>(&self, base: f64, entity: Entity, f: F) -> f64
    where
        F: Fn(&C) -> bool,
    {
        let mut factor = 1.0;

        let entities = self.entities(entity);

        for (component, operation, value, parent) in &self.modifiers {
            if parent.is_none_or(|p| entities.contains(&p.0) && f(component)) {
                match operation {
                    Operation::Add => (),
                    Operation::Multiply => {
                        factor *= value.0;
                    }
                }
            }
        }

        base * factor
    }
}

pub fn spawn_modifier(mut commands: Commands, modifier: &str, value: f64, source: Source) {
    let (op, name) = if let Some(name) = modifier.strip_suffix("-mult") {
        (Operation::Multiply, name)
    } else if let Some(name) = modifier.strip_suffix("-add") {
        (Operation::Add, name)
    } else {
        error!("Unknown modifier {modifier}");
        return;
    };
    let bundle = (op, Value(value), source);

    if let Some(sfx) = name.strip_prefix("recruitment-by-") {
        if let Some((follower1, follower2)) = sfx.split_once("-of-") {
            commands.spawn((
                RecruitmentByOf(follower1.to_string(), follower2.to_string()),
                bundle,
            ));
        } else {
            commands.spawn((RecruitmentBy(sfx.to_string()), bundle));
        }
    } else if let Some(follower) = name.strip_prefix("recruitment-of-") {
        commands.spawn((RecruitmentOf(follower.to_string()), bundle));
    } else if name == "income" {
        commands.spawn((IncomeModifier, bundle));
    } else if name == "expense" {
        commands.spawn((ExpenseModifier, bundle));
    } else {
        error!("Unknown modifier {modifier}");
    }
}
