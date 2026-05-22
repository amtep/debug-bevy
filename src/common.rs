use bevy::prelude::*;
use chrono::{Days, NaiveDate};

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CultName(pub String);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CultSymbol(pub usize);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Difficulty(pub String);

#[derive(Resource, Default)]
pub struct Dev;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Unlocked;

#[derive(Component, Reflect, Clone, Copy)]
#[reflect(Component)]
#[reflect(opaque)]
pub struct EndDate(pub NaiveDate);

impl EndDate {
    pub fn new(current: NaiveDate, duration: u32) -> Self {
        Self(current + Days::new(duration as u64))
    }
}
