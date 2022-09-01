use bevy::{prelude::*, tasks::IoTaskPool};
use bevy_ggrs::*;
use ggrs::InputStatus;
use rand::{thread_rng, Rng};
use bevy::input::keyboard::KeyboardInput;
use matchbox_socket::WebRtcSocket;
use input::*;
use components::{*, Direction};
use std::time::Duration;
mod components;
mod input;

struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    // 6-directions
    type Input = u8;
    type State = u8;
    // Matchbox's WebRtcSocket addresses are strings
    type Address = String;
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Matchmaking,
    InGame,
}

struct CrumpleHandle(Handle<Image>);

struct UpdateFollower {
    entity: Entity,
    hex: Hex
}

struct SpawnCrumple();

struct UpdateBody(Entity, Entity);

struct SpawnSegment(Entity, Entity);

#[derive(Default)]
struct WorldSize(isize);

fn main() {
    let mut app = App::new();

    GGRSPlugin::<GgrsConfig>::new()
        .with_input_system(input::input)
        .with_rollback_schedule(Schedule::default()
            .with_stage(
                "action",
                SystemStage::single_threaded()
                .with_system(action_system)
                .with_system(update_body)
                .with_system(spawn_segment)
                .with_system(hex_to_pixel.label("hex_to_pixel"))
                .with_system(head_movement)
                .with_system(head_crumple_collision)
                .with_system_set(
                    SystemSet::new()
                    .with_system(update_follower)
                    .with_system(on_update_follower)
                )
            )
            // .with_stage(
            //     "ROLLBACK_STAGE",
            //     SystemStage::single_threaded()
            //     .with_system(head_movement)
            //     .with_run_criteria(FixedTimestep::step(0.75))
            // )
        )
        .register_rollback_type::<Transform>()
        .register_rollback_type::<Hex>()
        .register_rollback_type::<Pcg32RandomT>()
        .register_rollback_type::<Crumple>()
        .register_rollback_type::<Segment>()
        .register_rollback_type::<MovementCooldown>()
        .build(&mut app);

    app
        .add_state(GameState::Matchmaking)
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .insert_resource(WorldSize(4))
        .add_startup_system(generate_map)
        .add_system_set(
            SystemSet::on_enter(GameState::Matchmaking)
                .with_system(start_matchbox_socket)
                .with_system(setup),
        )
        .add_startup_system(spawn_snake)
        .add_system(keyboard_events)
        .add_event::<UpdateFollower>()
        .add_event::<SpawnCrumple>()
        .add_event::<SpawnSegment>()
        .add_event::<UpdateBody>()
        .add_system_set(SystemSet::on_update(GameState::Matchmaking).with_system(wait_for_players))
        .add_system_set(SystemSet::on_enter(GameState::InGame).with_system(spawn_initial_crumple).with_system(spawn_crumple))
        .add_system_set(SystemSet::on_update(GameState::InGame).with_system(spawn_crumple))
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
    let mut camera = Camera2dBundle::default();
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
    let player_1 = commands.spawn_bundle(SpriteBundle {
        texture: texture_handle.clone(),
        ..Default::default()
    })
    .insert(Player { handle: 0 })
    .insert(Rollback::new(rip.next_id()))
    .insert(Hex { q: -2., r: 0., z: 1. })
    .insert(HexHistory(Vec::new()))
    .insert(Head { direction: Direction::None, last_direction: Direction::None })
    .insert(Tail)
    .insert(MovementCooldown { timer: Timer::new(Duration::from_millis(750), true)})
    .id();

    commands.entity(player_1).insert(Body(vec![player_1]));

    // player 2
    let player_2 = commands.spawn_bundle(SpriteBundle {
        texture: texture_handle.clone(),
        ..Default::default()
    })
    .insert(Player { handle: 1 })
    .insert(Rollback::new(rip.next_id()))
    .insert(Hex { q: 2., r: 0., z: 1. })
    .insert(HexHistory(Vec::new()))
    .insert(Head { direction: Direction::None, last_direction: Direction::None })
    .insert(Tail)
    .insert(MovementCooldown { timer: Timer::new(Duration::from_millis(750), true)})
    .id();

    commands.entity(player_2).insert(Body(vec![player_2]));
}

fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/next_2";
    info!("connecting to matchbox server: {:?}", room_url);
    let (socket, message_loop) = WebRtcSocket::new(room_url);

    // The message loop needs to be awaited, or nothing will happen.
    // We do this here using bevy's task system.
    let task_pool = IoTaskPool::get();
    task_pool.spawn(message_loop).detach();

    commands.insert_resource(Some(socket));
}

fn wait_for_players(
    mut commands: Commands, 
    mut socket: ResMut<Option<WebRtcSocket>>,
    mut state: ResMut<State<GameState>>, 
) {
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

    // rng seed
    let connected_peers = socket.connected_peers();

    let peer_id = connected_peers[0].clone().chars().filter(|c| c.is_digit(10)).collect::<String>().parse::<u128>().unwrap();
    let self_id = socket.id().chars().filter(|c| c.is_digit(10)).collect::<String>().parse::<u128>().unwrap();
    info!("peer id: {}", peer_id);
    info!("peer id: {}", self_id);
    let seed = u64::wrapping_mul(peer_id as u64, self_id as u64) as u64;
    info!("{}", seed);

    let rng = Pcg32RandomT::new(seed, 1);
    commands.insert_resource(rng);

    // start the GGRS session
    let session = session_builder
        .start_p2p_session(socket)
        .expect("failed to start session");
    
    commands.insert_resource(session);
    commands.insert_resource(SessionType::P2PSession);

    state.set(GameState::InGame).unwrap();
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
    mut query: Query<(&mut Hex, &mut Head, &mut HexHistory, &mut MovementCooldown)>,
    time: Res<Time>,
) {
    for (mut hex, mut head, mut hex_history, mut movement_cooldown) in query.iter_mut() {
        movement_cooldown.timer.tick(time.delta());
        if movement_cooldown.timer.finished() {
            hex_history.0.push(hex.clone());
            match head.direction {
                components::Direction::UpRight => {
                    hex.q += 1.;
                    hex.r -= 1.;
                },
                components::Direction::Right => {
                    hex.q += 1.;
                },
                components::Direction::DownRight => {
                    hex.r += 1.;
                },
                components::Direction::DownLeft => {
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
    mut spawn_crumple: EventWriter<SpawnCrumple>,
) {
    info!("spawn initial crumple");
    spawn_crumple.send(SpawnCrumple());
}

fn spawn_crumple(
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    world_size: Res<WorldSize>,
    mut spawn_crumple: EventReader<SpawnCrumple>,
    handle: Res<CrumpleHandle>,
    mut rng: ResMut<Pcg32RandomT>
) {
    for _ in spawn_crumple.iter() {
        info!("spawn crumple {}", rng.pcg32_random_r());
        commands.spawn_bundle(
            SpriteBundle {
                texture: handle.0.clone(),
                ..Default::default()
            }
        )
        .insert(Hex {
            q: (rng.pcg32_random_r() as f32 % world_size.0 as f32),
            r: (rng.pcg32_random_r() as f32 % world_size.0 as f32),
            // q: (rand::thread_rng().gen_range(-world_size.0..=world_size.0)) as f32,
            // r: (rand::thread_rng().gen_range(-world_size.0..=world_size.0)) as f32,
            z: 1.
        })
        .insert(Crumple)
        .insert(Rollback::new(rip.next_id()));
    }
}

fn update_body(
    mut query: Query<&mut Body>,
    mut update_body: EventReader<UpdateBody>
) {
    for ev in update_body.iter() {
        let mut body = query.get_mut(ev.0).unwrap();
        body.0.push(ev.1);
    }
}

fn spawn_segment(
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &Hex)>,
    mut spawn_segment: EventReader<SpawnSegment>,
    mut update_body: EventWriter<UpdateBody>
) {
    for ev in spawn_segment.iter() {
        let (entity, hex) = query.get_mut(ev.1).unwrap(); 
        commands.entity(entity).remove::<Tail>();
        let texture_handle = asset_server.load("HK-Heightend Sensory Input v2/HSI - Icons/HSI - Icon Geometric Light/HSI_icon_123l.png");
        let follower = commands.spawn_bundle(SpriteBundle {
            texture: texture_handle,
            ..Default::default()
        })
        .insert(Hex { q: hex.q, r: hex.r, z: 1. })
        .insert(Rollback::new(rip.next_id()))
        .insert(HexHistory(Vec::new()))
        .insert(Tail)
        .insert(Following(entity))
        .id();
        commands.entity(entity).insert(Follower(follower));
        update_body.send(UpdateBody(ev.0, follower));
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
    head_query: Query<(Entity, &Hex, &Body), With<Head>>,
    crumple_query: Query<(Entity, &Hex), With<Crumple>>,
    mut spawn_crumple: EventWriter<SpawnCrumple>,
    mut spawn_segment: EventWriter<SpawnSegment>
) {
    for (entity_head, hex_head, body) in head_query.iter() {
        for (crumple_entity, hex_crumple) in crumple_query.iter() {
            if hex_head.q == hex_crumple.q && hex_head.r == hex_crumple.r {
                commands.entity(crumple_entity).despawn();
                spawn_segment.send(SpawnSegment(entity_head, *body.0.last().unwrap()));
                spawn_crumple.send(SpawnCrumple());
            }
        }
    }
}

// fn rng_test(
//     mut rng: ResMut<Pcg32RandomT>
// ) {
//     info!("{}", rng.pcg32_random_r());
// }