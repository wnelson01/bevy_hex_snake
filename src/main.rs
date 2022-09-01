use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin, RegisterInspectable};
use rand::{thread_rng, Rng};
use bevy::input::keyboard::KeyboardInput;
use bevy::time::FixedTimestep;
use bevy_editor_pls::prelude::*;

struct CrumpleHandle(Handle<Image>);

struct UpdateFollower {
    entity: Entity,
    hex: Hex
}

struct SpawnCrumple();

struct SpawnSegment();

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
        .add_startup_system(spawn_initial_crumple)
        .add_system(spawn_crumple)
        .add_startup_system(setup)
        .add_startup_system(spawn_snake)
        .add_system(action_system)
        .add_system_set(
            SystemSet::new()
            .label("head_movement")
            .with_run_criteria(FixedTimestep::step(0.75))
            .with_system(head_movement)
        )
        .add_system(spawn_segment)
        .add_system_set(
            SystemSet::new()
            .with_system(update_follower)
            .with_system(on_update_follower)
        )
        .add_system(hex_to_pixel.label("hex_to_pixel"))
        .add_system(keyboard_events)
        .add_event::<UpdateFollower>()
        .add_event::<SpawnCrumple>()
        .add_event::<SpawnSegment>()
        .add_system(head_crumple_collision)
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
    server: Res<AssetServer>
) {
    let mut camera = Camera2dBundle::default();
    camera.transform.scale = Vec3::new(3.0, 3.0, 1.0);
    commands.spawn_bundle(camera);
    let handle: Handle<Image> = server.load("HK-Heightend Sensory Input v2/HSI - Icons/HSI - Icon Geometric Light/HSI_icon_109l.png");
    commands.insert_resource(CrumpleHandle(handle));
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
            keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) && 
            !keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) && 
            !keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]) && 
            !keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
                // head.direction = Direction::Up;
        } else if 
            keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) && 
            keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) && 
            !keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]) &&
            !keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
                head.direction = Direction::UpRight;
        } else if 
            !keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) &&
            keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) && 
            !keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]) && 
            !keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
                head.direction = Direction::Right;
        } else if 
            !keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) &&
            keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) && 
            keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]) && 
            !keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
                head.direction = Direction::DownRight;
        } else if
            !keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) &&
            !keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) && 
            keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]) && 
            !keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
                // head.direction = Direction::Down;
        } else if
            !keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) &&
            !keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) && 
            keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]) && 
            keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
                head.direction = Direction::DownLeft;
        } else if
            !keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) &&
            !keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) && 
            !keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]) && 
            keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
                head.direction = Direction::Left;
        } else if
            keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) &&
            !keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) && 
            !keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]) && 
            keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
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
    use bevy::input::ButtonState;
    for ev in key_evr.iter() {
        match ev.state {
            ButtonState::Pressed => {
                println!("Key press: {:?} ({})", ev.key_code, ev.scan_code);
            }
            ButtonState::Released => {
                println!("Key release: {:?} ({})", ev.key_code, ev.scan_code);
            }
        }
    }
}

fn spawn_initial_crumple(
    mut spawn_crumple: EventWriter<SpawnCrumple>
) {
    spawn_crumple.send(SpawnCrumple());
}

fn spawn_crumple(
    mut commands: Commands,
    world_size: Res<WorldSize>,
    mut spawn_crumple: EventReader<SpawnCrumple>,
    handle: Res<CrumpleHandle>
) {
    for _ in spawn_crumple.iter() {
        commands.spawn_bundle(
            SpriteBundle {
                texture: handle.0.clone(),
                ..Default::default()
            }
        )
        .insert(Hex {
            q: (rand::thread_rng().gen_range(-world_size.0..=world_size.0)) as f32,
            r: (rand::thread_rng().gen_range(-world_size.0..=world_size.0)) as f32,
            z: 1.
        })
        .insert(Crumple);
    }
}

fn spawn_segment(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Hex), With<Tail>>,
    mut spawn_segment: EventReader<SpawnSegment>
) {
    for _ in spawn_segment.iter() {
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

fn head_crumple_collision(
    mut commands: Commands,
    head_query: Query<&Hex, With<Head>>,
    crumple_query: Query<(Entity, &Hex), With<Crumple>>,
    mut spawn_crumple: EventWriter<SpawnCrumple>,
    mut spawn_segment: EventWriter<SpawnSegment>
) {
    for head_hex in head_query.iter() {
        for (crumple_entity, crumple_hex) in crumple_query.iter() {
            if head_hex.q == crumple_hex.q && head_hex.r == crumple_hex.r {
                commands.entity(crumple_entity).despawn();
                spawn_segment.send(SpawnSegment());
                spawn_crumple.send(SpawnCrumple());
            }
        }
    }
}