use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin, RegisterInspectable};
use rand::{thread_rng, Rng};
use bevy::input::keyboard::KeyboardInput;
use bevy::core::FixedTimestep;
use bevy_editor_pls::prelude::*;
use std::time::Duration;


#[derive(Default)]
struct WorldSize(isize);

#[derive(Component, Inspectable)]
struct Hex {
    q: f32,
    r: f32,
    z: f32
}

#[derive(Component)]
struct Head {
    direction: Direction,
    last_direction: Direction
}

#[derive(Component)]
struct Tail;

#[derive(Component)]
struct Crumple;

#[derive(Clone, Component, Copy, Debug, PartialEq)]
enum Direction {
    UpRight,
    Right,
    DownRight,
    DownLeft,
    Left,
    UpLeft,
    None
}

#[derive(Component)]
struct FuseTime {
    /// track when the bomb should explode (non-repeating timer)
    timer: Timer,
}
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(WorldSize(4))
        .register_inspectable::<Hex>()
        .add_startup_system(setup)
        .add_system(action_system)
        .add_system_set(
            SystemSet::new()
            .with_run_criteria(FixedTimestep::step(0.75))
            .with_system(head_movement)
        )
        .add_system(despawn_crumple)
        .add_system_set(
            SystemSet::new()
            .with_run_criteria(FixedTimestep::step(1.0))
            .with_system(spawn_crumple)
        )
        .add_system(hex_to_pixel)
        .add_system(keyboard_events)
        .run();
}

fn generate_map(x: isize) -> Vec<Hex> {
    let mut map: Vec<Hex> = vec![];
    for i in -x..=x {
        for j in -x..=x {
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
    world_size: Res<WorldSize>
) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform.scale = Vec3::new(2.0, 2.0, 1.0);
    commands.spawn_bundle(camera);

    let map = generate_map(world_size.0);
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
    .insert(Head {direction: Direction::None, last_direction: Direction::None});
}

/// Convert hex to pixel
/// size = distance from center to corner
/// x = size * (sqrt(3) * hex.q + sqrt(3)/2 * hex.r)
/// y = size * ( 3./2 * hex.r)
fn hex_to_pixel(
    mut query: Query<(&Hex, &mut Transform), Changed<Hex>>,
) {
    for (hex, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            105. * (3.0_f32.sqrt() * hex.q + (3.0_f32).sqrt() / 2. * hex.r),
            -105. * (3./2. * hex.r),
            hex.z,
        )
    }
}

fn action_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Head>
) {
    if let Some(mut head) = query.iter_mut().next() {
        if 
            keyboard_input.pressed(KeyCode::Up) && 
            !keyboard_input.pressed(KeyCode::Right) && 
            !keyboard_input.pressed(KeyCode::Down) && 
            !keyboard_input.pressed(KeyCode::Left) {
                // head.direction = Direction::Up;
        } else if 
            keyboard_input.pressed(KeyCode::Up) && 
            keyboard_input.pressed(KeyCode::Right) && 
            !keyboard_input.pressed(KeyCode::Down) &&
            !keyboard_input.pressed(KeyCode::Left) {
                head.direction = Direction::UpRight;
        } else if 
            !keyboard_input.pressed(KeyCode::Up) &&
            keyboard_input.pressed(KeyCode::Right) && 
            !keyboard_input.pressed(KeyCode::Down) && 
            !keyboard_input.pressed(KeyCode::Left) {
                head.direction = Direction::Right;
        } else if 
            !keyboard_input.pressed(KeyCode::Up) &&
            keyboard_input.pressed(KeyCode::Right) && 
            keyboard_input.pressed(KeyCode::Down) && 
            !keyboard_input.pressed(KeyCode::Left) {
                head.direction = Direction::DownRight;
        } else if
            !keyboard_input.pressed(KeyCode::Up) &&
            !keyboard_input.pressed(KeyCode::Right) && 
            keyboard_input.pressed(KeyCode::Down) && 
            !keyboard_input.pressed(KeyCode::Left) {
                // head.direction = Direction::Down;
        } else if
            !keyboard_input.pressed(KeyCode::Up) &&
            !keyboard_input.pressed(KeyCode::Right) && 
            keyboard_input.pressed(KeyCode::Down) && 
            keyboard_input.pressed(KeyCode::Left) {
                head.direction = Direction::DownLeft;
        } else if
            !keyboard_input.pressed(KeyCode::Up) &&
            !keyboard_input.pressed(KeyCode::Right) && 
            !keyboard_input.pressed(KeyCode::Down) && 
            keyboard_input.pressed(KeyCode::Left) {
                head.direction = Direction::Left;
        } else if
            keyboard_input.pressed(KeyCode::Up) &&
            !keyboard_input.pressed(KeyCode::Right) && 
            !keyboard_input.pressed(KeyCode::Down) && 
            keyboard_input.pressed(KeyCode::Left) {
                head.direction = Direction::UpLeft
        } else if
            keyboard_input.pressed(KeyCode::Space) {
                head.direction = Direction::None;
            }
    }
}

fn head_movement(
    mut query: Query<(&mut Hex, &mut Head)>
) {
    for (mut hex, mut head) in query.iter_mut() {
        match head.direction {
            Direction::UpRight => {
                hex.q += 1.;
                hex.r -= 1.;
            },
            Direction::Right => {
                hex.q += 1.;
            },
            Direction::DownRight => {
                hex.r += 1.;
            },
            Direction::DownLeft => {
                hex.q -= 1.;
                hex.r += 1.;
            },
            Direction::Left => {
                hex.q -= 1.;
            },
            Direction::UpLeft => {
                hex.r -= 1.;
            },
            Direction::None => (),
        }
        head.last_direction = head.direction;
    }
}

fn keyboard_events(
    mut key_evr: EventReader<KeyboardInput>,
) {
    use bevy::input::ElementState;
    for ev in key_evr.iter() {
        match ev.state {
            ElementState::Pressed => {
                println!("Key press: {:?} ({})", ev.key_code, ev.scan_code);
            }
            ElementState::Released => {
                println!("Key release: {:?} ({})", ev.key_code, ev.scan_code);
            }
        }
    }
}

fn spawn_crumple(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world_size: Res<WorldSize>,
) {
    let handle: Handle<Image>= asset_server.load("HK-Heightend Sensory Input v2/HSI - Icons/HSI - Icon Geometric Light/HSI_icon_109l.png");
    commands.spawn_bundle(SpriteBundle {
            texture: handle,
            ..Default::default()
        })
        .insert(Hex {
            q: (rand::thread_rng().gen_range(-world_size.0..=world_size.0)) as f32,
            r: (rand::thread_rng().gen_range(-world_size.0..=world_size.0)) as f32,
            z: 1.
        })
        .insert(Crumple)
        .insert(FuseTime {
            timer: Timer::new(Duration::from_secs(1), false)
        });
}

fn despawn_crumple(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FuseTime), With<Crumple>>,
    time: Res<Time>,
) {
    for (entity, mut fuse_timer) in query.iter_mut() {
        fuse_timer.timer.tick(time.delta());

        if fuse_timer.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}