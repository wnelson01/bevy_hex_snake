use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin, RegisterInspectable};
use rand::{thread_rng, Rng};

#[derive(Component, Inspectable)]
struct Hex {
    q: f32,
    r: f32,
    z: f32
}

#[derive(Component)]
struct Head;

#[derive(Component)]
struct Tail;

#[derive(Component)]
struct Food;

#[derive(Component)]
struct InputHandler {
    action: Action,
}

#[derive(Component, Debug)]
enum Action {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
    None,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<Hex>()
        .add_startup_system(setup)
        .add_system(hex_to_pixel)
        .add_system(action_system)
        .add_system(head_movement)
        .run();
}

fn generate_map(x: isize) -> Vec<Hex> {
    let mut map: Vec<Hex> = vec![];
    for i in (-1 * x)..x+1 {
        for j in -x..x+1 {
            if (i + j).abs() <= x {
                map.push(Hex{ q: i as f32, r: j as f32, z: 0. });
            }
        }
    }
    map
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform.scale = Vec3::new(2.0, 2.0, 1.0);
    commands.spawn_bundle(camera);

    let map = generate_map(4);
    let texture_handle = asset_server.load_folder("HK-Heightend Sensory Input v2/HSI - Indigo/").unwrap();
    for hex in map {
        commands.spawn_bundle(SpriteBundle {
            texture: texture_handle[thread_rng().gen_range(0..texture_handle.len())].clone().typed().into(),
            ..Default::default()
        }).insert(hex);
    }

    let texture_handle = asset_server.load("HK-Heightend Sensory Input v2/HSI - Icons/HSI - Icon Geometric Light/HSI_icon_123l.png");
    commands.spawn_bundle(SpriteBundle {
        texture: texture_handle,
        ..Default::default()
    })
    .insert(Hex { q: 0., r: 0., z: 1. })
    .insert(Head)
    .insert(InputHandler { action: Action::None });
}

/// Convert hex to pixel
fn hex_to_pixel(
    mut query: Query<(&Hex, &mut Transform), Changed<Hex>>,
) {
    for (hex, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            105. * (3.0_f32.sqrt() * hex.q + (3 as f32).sqrt() / 2. * hex.r),
            -105. * (3./2. * hex.r),
            hex.z,
        )
    }
}

fn action_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut InputHandler>
) {
    if let Some(mut input_handler) = query.iter_mut().next() {
        if 
            keyboard_input.pressed(KeyCode::Up) && 
            !keyboard_input.pressed(KeyCode::Right) && 
            !keyboard_input.pressed(KeyCode::Down) && 
            !keyboard_input.pressed(KeyCode::Left) {
                input_handler.action = Action::Up;
        } else if 
            keyboard_input.pressed(KeyCode::Up) && 
            keyboard_input.pressed(KeyCode::Right) && 
            !keyboard_input.pressed(KeyCode::Down) &&
            !keyboard_input.pressed(KeyCode::Left) {
                input_handler.action = Action::UpRight;
        } else if 
            !keyboard_input.pressed(KeyCode::Up) &&
            keyboard_input.pressed(KeyCode::Right) && 
            !keyboard_input.pressed(KeyCode::Down) && 
            !keyboard_input.pressed(KeyCode::Left) {
                input_handler.action = Action::Right;
        } else if 
            !keyboard_input.pressed(KeyCode::Up) &&
            keyboard_input.pressed(KeyCode::Right) && 
            keyboard_input.pressed(KeyCode::Down) && 
            !keyboard_input.pressed(KeyCode::Left) {
                input_handler.action = Action::DownRight;
        } else if
            !keyboard_input.pressed(KeyCode::Up) &&
            !keyboard_input.pressed(KeyCode::Right) && 
            keyboard_input.pressed(KeyCode::Down) && 
            !keyboard_input.pressed(KeyCode::Left) {
                input_handler.action = Action::Down;
        } else if
            !keyboard_input.pressed(KeyCode::Up) &&
            !keyboard_input.pressed(KeyCode::Right) && 
            keyboard_input.pressed(KeyCode::Down) && 
            keyboard_input.pressed(KeyCode::Left) {
                input_handler.action = Action::DownLeft;
        } else if
            !keyboard_input.pressed(KeyCode::Up) &&
            !keyboard_input.pressed(KeyCode::Right) && 
            !keyboard_input.pressed(KeyCode::Down) && 
            keyboard_input.pressed(KeyCode::Left) {
                input_handler.action = Action::Left;
        } else if
            keyboard_input.pressed(KeyCode::Up) &&
            !keyboard_input.pressed(KeyCode::Right) && 
            !keyboard_input.pressed(KeyCode::Down) && 
            keyboard_input.pressed(KeyCode::Left) {
                input_handler.action = Action::UpLeft;
        } else {
            input_handler.action = Action::None;
        }
    }
}

fn head_movement(
    mut query: Query<(&mut Hex, &InputHandler), With<Head>>
) {
    for (mut hex, input_handler) in query.iter_mut() {
        match input_handler.action {
            Action::Up => (),
            Action::UpRight => {
                hex.q += 1.;
                hex.r -= 1.;
            },
            Action::Right => {
                hex.q += 1.;
            },
            Action::DownRight => {
                hex.r += 1.;
            },
            Action::DownLeft => {
                hex.q -= 1.;
                hex.r += 1.;
            },
            Action::Down => (),
            Action::Left => {
                hex.q -= 1.;
            },
            Action::UpLeft => {
                hex.r -= 1.;
            },
            Action::None => (),
        }
    }
}