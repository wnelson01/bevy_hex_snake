use bevy::prelude::*;

pub const INPUT_UP_RIGHT: u8 = 1 << 0;
pub const INPUT_RIGHT: u8 = 1 << 1;
pub const INPUT_DOWN_RIGHT: u8 = 1 << 2;
pub const INPUT_DOWN_LEFT: u8 = 1 << 3;
pub const INPUT_LEFT: u8 = 1 << 4;
pub const INPUT_UP_LEFT: u8 = 1 << 5;

pub fn input(_: In<ggrs::PlayerHandle>, keys: Res<Input<KeyCode>>) -> u8 {
    let mut input = 0u8;
    if 
        keys.any_pressed([KeyCode::Up, KeyCode::W]) && 
        !keys.any_pressed([KeyCode::Right, KeyCode::D]) && 
        !keys.any_pressed([KeyCode::Down, KeyCode::S]) && 
        !keys.any_pressed([KeyCode::Left, KeyCode::A]) {
            // head.direction = Direction::Up;
    } else if 
        keys.any_pressed([KeyCode::Up, KeyCode::W]) && 
        keys.any_pressed([KeyCode::Right, KeyCode::D]) && 
        !keys.any_pressed([KeyCode::Down, KeyCode::S]) &&
        !keys.any_pressed([KeyCode::Left, KeyCode::A]) {
            input |= INPUT_UP_RIGHT;
    } else if 
        !keys.any_pressed([KeyCode::Up, KeyCode::W]) &&
        keys.any_pressed([KeyCode::Right, KeyCode::D]) && 
        !keys.any_pressed([KeyCode::Down, KeyCode::S]) && 
        !keys.any_pressed([KeyCode::Left, KeyCode::A]) {
            input |= INPUT_RIGHT;
    } else if 
        !keys.any_pressed([KeyCode::Up, KeyCode::W]) &&
        keys.any_pressed([KeyCode::Right, KeyCode::D]) && 
        keys.any_pressed([KeyCode::Down, KeyCode::S]) && 
        !keys.any_pressed([KeyCode::Left, KeyCode::A]) {
            input |= INPUT_DOWN_RIGHT;
    } else if
        !keys.any_pressed([KeyCode::Up, KeyCode::W]) &&
        !keys.any_pressed([KeyCode::Right, KeyCode::D]) && 
        keys.any_pressed([KeyCode::Down, KeyCode::S]) && 
        !keys.any_pressed([KeyCode::Left, KeyCode::A]) {
            // head.direction = Direction::Down;
    } else if
        !keys.any_pressed([KeyCode::Up, KeyCode::W]) &&
        !keys.any_pressed([KeyCode::Right, KeyCode::D]) && 
        keys.any_pressed([KeyCode::Down, KeyCode::S]) && 
        keys.any_pressed([KeyCode::Left, KeyCode::A]) {
            input |= INPUT_DOWN_LEFT;
    } else if
        !keys.any_pressed([KeyCode::Up, KeyCode::W]) &&
        !keys.any_pressed([KeyCode::Right, KeyCode::D]) && 
        !keys.any_pressed([KeyCode::Down, KeyCode::S]) && 
        keys.any_pressed([KeyCode::Left, KeyCode::A]) {
            input |= INPUT_LEFT;
    } else if
        keys.any_pressed([KeyCode::Up, KeyCode::W]) &&
        !keys.any_pressed([KeyCode::Right, KeyCode::D]) && 
        !keys.any_pressed([KeyCode::Down, KeyCode::S]) && 
        keys.any_pressed([KeyCode::Left, KeyCode::A]) {
            input |= INPUT_UP_LEFT;
    } else if
        keys.pressed(KeyCode::Space) {
            // head.direction = Direction::None;
    }

    input
}