use bevy::prelude::*;

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
