use bevy::prelude::*;

#[derive(Clone,Component,Copy, Debug, PartialEq)]
pub enum Direction {
    UpRight,
    Right,
    DownRight,
    DownLeft,
    Left,
    UpLeft,
    None
}

#[derive(Component)]
pub struct Player {
    pub handle: usize,
}

#[derive(Component)]
pub struct Following(pub Entity);

#[derive(Component)]
pub struct Follower(pub Entity);

#[derive(Component, Clone, Copy)]
pub struct Hex {
    pub q: f32,
    pub r: f32,
    pub z: f32,
}

#[derive(Component)]
pub struct HexHistory(pub Vec<Hex>);

#[derive(Component)]
pub struct Head {
    pub direction: Direction,
    pub last_direction: Direction
}

#[derive(Component)]
pub struct Body(pub Vec<Entity>);