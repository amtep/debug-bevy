use bevy::prelude::*;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CultName(pub String);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CultSymbol(pub char);

#[derive(Resource, Default)]
pub struct Dev;
