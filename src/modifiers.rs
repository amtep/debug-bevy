use bevy::{ecs::system::SystemParam, prelude::*};

use chrono::NaiveDate;
use moonshine_save::save::Save;
use serde::Deserialize;

use crate::{
    common::EndDate,
    constants::ui::colors::{TEXT_NEGATIVE, TEXT_POSITIVE},
    state::GameState,
    text::TextKey,
};

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
#[derive(Component, Reflect, Clone)]
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

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct IncomeCategoryModifier(String);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ExpenseCategoryModifier(String);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct IntelligenceSuspicionModifier;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScientificSuspicionModifier;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PoliceSuspicionModifier;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MediaSuspicionModifier;

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

    #[expect(dead_code)]
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

#[derive(Deserialize, Clone, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct ModifierValue {
    #[serde(flatten)]
    op: OperationValue,
    #[serde(flatten)]
    kind: ModifierKindValue,
    duration: Option<u32>,
}

#[derive(Deserialize, Clone, Debug, Reflect)]
#[serde(rename_all = "kebab-case")]
enum OperationValue {
    Add(f64),
    Mult(f64),
}

#[derive(Deserialize, Clone, Debug, Reflect)]
#[serde(rename_all = "kebab-case")]
enum ModifierKindValue {
    Income {
        category: Option<String>,
    },
    Expense {
        category: Option<String>,
    },
    Recruit {
        by: Option<String>,
        of: Option<String>,
    },
    IntelligenceSuspicion,
    ScientificSuspicion,
    PoliceSuspicion,
    MediaSuspicion,
}

impl ModifierValue {
    pub fn text_bundle(&self, end_date: Option<&EndDate>, shown: bool) -> (TextKey, TextColor) {
        let mut text_key = TextKey::new("modifier");
        let value_positive = match self.op {
            OperationValue::Add(amount) => {
                text_key.add_arg("op", "add").add_arg("amount", amount);
                amount.is_sign_positive()
            }
            OperationValue::Mult(amount) => {
                text_key
                    .add_arg("op", "mult")
                    .add_arg("percent", ((amount - 1.0) * 100.0).round());
                amount >= 1.0
            }
        };
        if let Some(duration) = self.duration {
            if let Some(end_date) = end_date {
                text_key.add_arg("duration", -1).add_arg("date", end_date.0);
            } else {
                text_key.add_arg("duration", duration as f64);
            }
        } else {
            text_key.add_arg("duration", 0);
        }

        let (positive, modifier) = match self.kind.clone() {
            ModifierKindValue::Income { category: None } => (true, "income"),
            ModifierKindValue::Income {
                category: Some(cat),
            } => {
                text_key.add_arg("cat", cat);
                (true, "income-category")
            }
            ModifierKindValue::Expense { category: None } => (false, "expense"),
            ModifierKindValue::Expense {
                category: Some(cat),
            } => {
                text_key.add_arg("cat", cat);
                (false, "expense-category")
            }
            ModifierKindValue::Recruit {
                by: Some(by),
                of: None,
            } => {
                text_key.add_arg("by", by);
                (true, "recruit-by")
            }
            ModifierKindValue::Recruit {
                by: None,
                of: Some(of),
            } => {
                text_key.add_arg("of", of);
                (true, "recruit-of")
            }
            ModifierKindValue::Recruit {
                by: Some(by),
                of: Some(of),
            } => {
                text_key.add_arg("by", by);
                text_key.add_arg("of", of);
                (true, "recruit-by-of")
            }
            ModifierKindValue::IntelligenceSuspicion => (false, "intelligence-suspicion"),
            ModifierKindValue::ScientificSuspicion => (false, "scientific-suspicion"),
            ModifierKindValue::PoliceSuspicion => (false, "police-suspicion"),
            ModifierKindValue::MediaSuspicion => (false, "media-suspicion"),
            _ => unimplemented!(),
        };

        if shown {
            text_key.add_arg("modifier", TextKey::new(format!("modifier-{modifier}")));
        } else {
            text_key.add_arg("modifier", "");
        }

        let text_color = TextColor::from(if positive ^ value_positive {
            TEXT_NEGATIVE
        } else {
            TEXT_POSITIVE
        });

        (text_key, text_color)
    }
}

pub fn spawn_modifier(
    mut commands: Commands,
    entity: Option<Entity>,
    current_date: Option<NaiveDate>,
    modifier: &ModifierValue,
    source: Source,
) {
    let bundle = (
        match modifier.op {
            OperationValue::Add(value) => (Operation::Add, Value(value)),
            OperationValue::Mult(value) => (Operation::Multiply, Value(value)),
        },
        modifier.clone(),
        source,
    );

    let mut commands = if let Some(entity) = entity {
        commands.spawn((ChildOf(entity), bundle))
    } else {
        commands.spawn(bundle)
    };

    if let Some(duration) = modifier.duration {
        if let Some(current_date) = current_date {
            commands.insert(EndDate::new(current_date, duration));
        } else {
            error!("timed modifier without current date supplied");
        }
    }

    match modifier.kind.clone() {
        ModifierKindValue::Income { category: None } => {
            commands.insert(IncomeModifier);
        }
        ModifierKindValue::Income {
            category: Some(category),
        } => {
            commands.insert(IncomeCategoryModifier(category));
        }
        ModifierKindValue::Expense { category: None } => {
            commands.insert(ExpenseModifier);
        }
        ModifierKindValue::Expense {
            category: Some(category),
        } => {
            commands.insert(ExpenseCategoryModifier(category));
        }
        ModifierKindValue::Recruit {
            by: None,
            of: Some(of),
        } => {
            commands.insert(RecruitmentOf(of));
        }
        ModifierKindValue::Recruit {
            by: Some(by),
            of: None,
        } => {
            commands.insert(RecruitmentBy(by));
        }
        ModifierKindValue::Recruit {
            by: Some(by),
            of: Some(of),
        } => {
            commands.insert(RecruitmentByOf(by, of));
        }
        ModifierKindValue::IntelligenceSuspicion => {
            commands.insert(IntelligenceSuspicionModifier);
        }
        ModifierKindValue::ScientificSuspicion => {
            commands.insert(ScientificSuspicionModifier);
        }
        ModifierKindValue::PoliceSuspicion => {
            commands.insert(PoliceSuspicionModifier);
        }
        ModifierKindValue::MediaSuspicion => {
            commands.insert(MediaSuspicionModifier);
        }
        _ => error!("incorrect modifier combination"),
    }
}
