use bevy::{prelude::*, tasks::IoTaskPool};
use bevy_ggrs::*;
use ggrs::InputStatus;
// use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin, RegisterInspectable};
use rand::{thread_rng, Rng};
use bevy::input::keyboard::KeyboardInput;
use bevy::core::FixedTimestep;
// use bevy_editor_pls::prelude::*;
use matchbox_socket::WebRtcSocket;

#[derive(Component)]
struct Player {
    handle: usize,
}

struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    // 6-directions
    type Input = u8;
    type State = u8;
    // Matchbox's WebRtcSocket addresses are strings
    type Address = String;
}

const INPUT_UP_RIGHT: u8 = 1 << 0;
const INPUT_RIGHT: u8 = 1 << 1;
const INPUT_DOWN_RIGHT: u8 = 1 << 2;
const INPUT_DOWN_LEFT: u8 = 1 << 3;
const INPUT_LEFT: u8 = 1 << 4;
const INPUT_UP_LEFT: u8 = 1 << 5;

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

#[derive(Component)]
struct Head {
    direction: Direction,
    last_direction: Direction
}

#[derive(Component)]
struct Tail;

#[derive(Component)]
struct Crumple;

#[derive(Clone, Component,Copy, Debug, PartialEq)]
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
    let mut app = App::new();

    GGRSPlugin::<GgrsConfig>::new()
        .with_input_system(input)
        .with_rollback_schedule(Schedule::default()
            .with_stage(
                "action",
                SystemStage::single_threaded()
                .with_system(action_system)
            )
            .with_stage(
                "ROLLBACK_STAGE",
                SystemStage::single_threaded()
                .with_system(head_movement)
                .with_run_criteria(FixedTimestep::step(0.75))
            )
        )
        .register_rollback_type::<Transform>()
        .build(&mut app);

    app
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .insert_resource(WorldSize(4))
        // .register_inspectable::<Hex>()
        .add_startup_system(generate_map)
        .add_startup_system(spawn_initial_crumple)
        .add_startup_system(start_matchbox_socket)
        .add_system(wait_for_players)
        .add_system(spawn_crumple)
        .add_startup_system(setup)
        .add_startup_system(spawn_snake)
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
    // let texture_handle = asset_server.load_folder("HK-Heightend Sensory Input v2/HSI - Indigo/").unwrap();
    let mut texture_handle = Vec::new();
    for i in 1..13 {
        let path = format!("HK-Heightend Sensory Input v2/HSI - Indigo/HSI_indigo_{:03}.png", i);
        texture_handle.push(asset_server.load(&path));
    }

    for hex in map {
        commands.spawn_bundle(
            SpriteBundle{
                texture: texture_handle[thread_rng().gen_range(0..texture_handle.len())].clone(),
                ..Default::default()
            })
            .insert(hex);
    }
}

fn setup(
    mut commands: Commands,
    server: Res<AssetServer>
) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform.scale = Vec3::new(3.0, 3.0, 1.0);
    commands.spawn_bundle(camera);
    let handle: Handle<Image> = server.load("HK-Heightend Sensory Input v2/HSI - Icons/HSI - Icon Geometric Light/HSI_icon_109l.png");
    commands.insert_resource(CrumpleHandle(handle));
}

fn spawn_snake(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rip: ResMut<RollbackIdProvider>
) {
    let texture_handle = asset_server.load("HK-Heightend Sensory Input v2/HSI - Icons/HSI - Icon Geometric Light/HSI_icon_123l.png");

    // player 1
    commands.spawn_bundle(SpriteBundle {
        texture: texture_handle.clone(),
        ..Default::default()
    })
    .insert(Player { handle: 0 })
    .insert(Rollback::new(rip.next_id()))
    .insert(Hex { q: -2., r: 0., z: 1. })
    .insert(HexHistory(Vec::new()))
    .insert(Head { direction: Direction::None, last_direction: Direction::None })
    .insert(Tail);

    // player 2
    commands.spawn_bundle(SpriteBundle {
        texture: texture_handle.clone(),
        ..Default::default()
    })
    .insert(Player { handle: 1 })
    .insert(Rollback::new(rip.next_id()))
    .insert(Hex { q: 2., r: 0., z: 1. })
    .insert(HexHistory(Vec::new()))
    .insert(Head { direction: Direction::None, last_direction: Direction::None })
    .insert(Tail);
}

fn start_matchbox_socket(mut commands: Commands, task_pool: Res<IoTaskPool>) {
    let room_url = "ws://127.0.0.1:3536/next_2";
    info!("connecting to matchbox server: {:?}", room_url);
    let (socket, message_loop) = WebRtcSocket::new(room_url);

    // The message loop needs to be awaited, or nothing will happen.
    // We do this here using bevy's task system.
    task_pool.spawn(message_loop).detach();

    commands.insert_resource(Some(socket));
}

fn wait_for_players(mut commands: Commands, mut socket: ResMut<Option<WebRtcSocket>>) {
    let socket = socket.as_mut();

    // If there is no socket we've already started the game
    if socket.is_none() {
        return;
    }

    // Check for new connections
    socket.as_mut().unwrap().accept_new_connections();
    let players = socket.as_ref().unwrap().players();

    let num_players = 2;
    if players.len() < num_players {
        return; // wait for more players
    }

    info!("All peers have joined, going in-game");

    // create a GGRS P2P session
    let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
        .with_num_players(num_players)
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    // move the socket out of the resource (required because GGRS takes ownership of it)
    let socket = socket.take().unwrap();

    // start the GGRS session
    let session = session_builder
        .start_p2p_session(socket)
        .expect("failed to start session");

    commands.insert_resource(session);
    commands.insert_resource(SessionType::P2PSession);
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

fn input(_: In<ggrs::PlayerHandle>, keys: Res<Input<KeyCode>>) -> u8 {
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

fn action_system(
    inputs: Res<Vec<(u8, InputStatus)>>,
    mut query: Query<(&mut Head, &Player)>
) {
    for (mut head, player) in query.iter_mut() {
        let (input, _) = inputs[player.handle];

        match input {
            INPUT_UP_RIGHT => {
                head.direction = Direction::UpRight;
            },
            INPUT_RIGHT => {
                head.direction = Direction::Right;
            },
            INPUT_DOWN_RIGHT => {
                head.direction = Direction::DownRight;
            },
            INPUT_DOWN_LEFT => {
                head.direction = Direction::DownLeft;
            },
            INPUT_LEFT => {
                head.direction = Direction::Left;
            },
            INPUT_UP_LEFT => {
                head.direction = Direction::UpLeft;
            },
            _ => (),
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
