use bevy::prelude::*;

#[derive(Debug)]
struct Hex {
    q: f32,
    r: f32,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .run();
}

fn generate_map(x: isize) -> Vec<Hex> {
    let mut map: Vec<Hex> = vec![];
    for i in (-1 * x)..x+1 {
        for j in -x..x+1 {
            if (i + j).abs() <= x {
                map.push(Hex{ q: i as f32, r: j as f32 });
            }
        }
    }
    map
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let map = generate_map(3);
    // let mut map: Vec<Hex> =  vec![Hex { q: 0.0, r: 0.0 }];
    // map.push(Hex {q: 1.0, r: 0.0});
    // map.push(Hex {q: 0.0, r: 1.0});
    // map.push(Hex {q: 1.0, r: 1.0 });
    // map.push(Hex {q: -3.0, r: 3.0});
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let texture_handle = asset_server.load_folder("HK-Heightend Sensory Input v2/HSI - Indigo/").unwrap();
    println!("{:?}", map);
    for hex in map {
        commands.spawn_bundle(SpriteBundle {
            material: materials.add((texture_handle[0]).clone().typed().into()),
            transform: Transform {
                translation: Vec3::new(
                    52.5 * (3.0_f32.sqrt() * hex.q + (3 as f32).sqrt() / 2. * hex.r),
                    -52.5 * (3./2. * hex.r),
                    0.0,
                ),
                scale: Vec3::new(0.5, 0.5, 1.0),
                ..Default::default()},
            // transform: Transform::from_translation(
            //     Vec3::new(
            //         105. * ((3 as f32).sqrt() * hex.q as f32 + (3 as f32).sqrt() / 2. * hex.r as f32),
            //         105. * (3./2. * hex.r as f32),
            //         0.0,
            //     ),
            // ),
            ..Default::default()
        });
    }
}