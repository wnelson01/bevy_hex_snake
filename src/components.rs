use bevy::prelude::*;
use rand_pcg::Pcg64;

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

#[derive(Component, Reflect, Default, Clone, Copy)]
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

#[derive(Component)]
pub struct Tail;

#[derive(Component)]
pub struct Crumple;

#[derive(Component)]
pub struct Segment;

#[derive(Component)]
pub struct RandomNumberGenerator(pub Pcg64);