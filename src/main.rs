use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin, RegisterInspectable};
use rand::{thread_rng, Rng};
use bevy::input::keyboard::KeyboardInput;
use bevy::core::FixedTimestep;
use bevy_editor_pls::prelude::*;
use std::time::Duration;

struct UpdateFollower {
    entity: Entity,
    hex: Hex
}

#[derive(Default)]
struct WorldSize(isize);

#[derive(Component)]
struct Following(Entity);

#[derive(Component)]
struct Follower(Entity);

#[derive(Component, Clone, Copy)]
struct Hex {
    q: f32,
    r: f32,
    z: f32,
}

#[derive(Component)]
struct HexHistory(Vec<Hex>);

#[derive(Component, Inspectable)]
struct Head {
    direction: Direction,
    last_direction: Direction
}

#[derive(Component, Inspectable)]
struct Tail;

#[derive(Component)]
struct Crumple;

#[derive(Clone, Component, Inspectable, Copy, Debug, PartialEq)]
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

#[derive(Component)]
struct Segment;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(WorldSize(4))
        // .register_inspectable::<Hex>()
        .register_inspectable::<Head>()
        .register_inspectable::<Tail>()
        .add_startup_system(generate_map)
        .add_startup_system(setup)
        .add_startup_system(spawn_snake)
        .add_system(action_system)
        .add_system_set(
            SystemSet::new()
            .label("head_movement")
            .with_run_criteria(FixedTimestep::step(0.75))
            .with_system(head_movement)
        )
        .add_system(despawn_crumple)
        // .add_system_set(
        //     SystemSet::new()
        //     .with_run_criteria(FixedTimestep::step(1.0))
        //     .with_system(spawn_crumple)
        // )
        .add_system_set(
            SystemSet::new()
            .label("spawn_segment")
            .with_run_criteria(FixedTimestep::step(3.0))
            .with_system(spawn_segment)
        )
        .add_system_set(
            SystemSet::new()
            .with_system(update_follower)
            .with_system(on_update_follower)
        )
        .add_system(hex_to_pixel.label("hex_to_pixel"))
        .add_system(keyboard_events)
        .add_event::<UpdateFollower>()
        .run();
}

fn generate_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world_size: Res<WorldSize>
) {
    let mut map: Vec<Hex> = vec![];
    for i in -world_size.0..=world_size.0 {
        for j in -world_size.0..=world_size.0 {
            if (i + j).abs() <= world_size.0 {
                map.push(Hex { q: i as f32, r: j as f32, z: 0. });
            }
        }
    }
    let texture_handle = asset_server.load_folder("HK-Heightend Sensory Input v2/HSI - Indigo/").unwrap();
    for hex in map {
        commands.spawn_bundle(
            SpriteBundle{
                texture: texture_handle[thread_rng().gen_range(0..texture_handle.len())].clone().typed().into(),
                ..Default::default()
            })
            .insert(hex);
    }
}

fn setup(
    mut commands: Commands,
) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform.scale = Vec3::new(2.0, 2.0, 1.0);
    commands.spawn_bundle(camera);
}

fn spawn_snake(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let texture_handle = asset_server.load("HK-Heightend Sensory Input v2/HSI - Icons/HSI - Icon Geometric Light/HSI_icon_123l.png");
    commands.spawn_bundle(SpriteBundle {
        texture: texture_handle,
        ..Default::default()
    })
    .insert(Hex { q: 0., r: 0., z: 1. })
    .insert(HexHistory(Vec::new()))
    .insert(Head { direction: Direction::None, last_direction: Direction::None })
    .insert(Tail);
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
    mut query: Query<(&mut Hex, &mut Head, &mut HexHistory)>
) {
    for (mut hex, mut head, mut hex_history) in query.iter_mut() {
        hex_history.0.push(hex.clone());
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

fn spawn_segment(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Hex), With<Tail>>
) {
    let (entity, hex) = query.single();
    commands.entity(entity).remove::<Tail>();
    let texture_handle = asset_server.load("HK-Heightend Sensory Input v2/HSI - Icons/HSI - Icon Geometric Light/HSI_icon_123l.png");
    let follower = commands.spawn_bundle(SpriteBundle {
        texture: texture_handle,
        ..Default::default()
    })
    .insert(Hex { q: hex.q, r: hex.r, z: 1. })
    .insert(HexHistory(Vec::new()))
    .insert(Tail)
    .insert(Following(entity)).id();
    commands.entity(entity).insert(Follower(follower));
}

fn update_follower(
    query: Query<(&HexHistory, &Follower), Changed<Hex>>,
    mut update_follower: EventWriter<UpdateFollower>
) {
    for (hex_history, follower) in query.iter() {
        let event = UpdateFollower {
            entity: follower.0,
            hex: hex_history.0.last().unwrap().clone()
        };
        update_follower.send(event);
    }
}

fn on_update_follower(
    mut query: Query<(&mut Hex, &mut HexHistory), With<Following>>,
    mut follower_update: EventReader<UpdateFollower>
) {
    for event in follower_update.iter() {
        let entity = event.entity;
        let (mut hex, mut hex_history) = query.get_mut(entity).unwrap();
        hex_history.0.push(hex.clone());
        let q = event.hex.q;
        let r = event.hex.r;
        hex.q = q;
        hex.r = r;
    }
}