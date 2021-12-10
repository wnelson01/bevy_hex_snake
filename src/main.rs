use bevy::prelude::*;

#[derive(Debug)]
struct Hex {
    q: f64,
    r: f64,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .run();
}

fn generate_map(x: isize) -> Vec<Hex> {
    let mut map: Vec<Hex> = vec![];
    for i in -x..x+1 {
        for j in -x..x+1 {
            map.push(Hex{ q: i as f64, r: j as f64 });
        }
    }
    map
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let map = generate_map(4);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let texture_handle = asset_server.load_folder("HK-Heightend Sensory Input v2/HSI - Indigo/").unwrap();
    println!("{:?}", map);
    for hex in map {
        commands.spawn_bundle(SpriteBundle {
            material: materials.add((texture_handle[0]).clone().typed().into()),
            transform: Transform::from_translation(
                Vec3::new(
                    75. * ((3 as f32).sqrt() * hex.q as f32 + (3 as f32).sqrt() / 2. * hex.r as f32) + 5.,
                    75. * ((3./2. * hex.r as f32)) - 210.,
                    0.0,
                ),
            ),
            ..Default::default()
        });
    }
}
