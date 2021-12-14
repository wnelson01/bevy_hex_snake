use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use rand::{thread_rng, Rng};

#[derive(Debug)]
struct Hex {
    q: f32,
    r: f32,
}


fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
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
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let map = generate_map(3);
    let texture_handle = asset_server.load_folder("HK-Heightend Sensory Input v2/HSI - Indigo/").unwrap();
    for hex in map {
        commands.spawn_bundle(SpriteBundle {
            material: materials.add((texture_handle[thread_rng().gen_range(0..texture_handle.len())]).clone().typed().into()),
            transform: Transform {
                translation: Vec3::new(
                    52.5 * (3.0_f32.sqrt() * hex.q + (3 as f32).sqrt() / 2. * hex.r),
                    -52.5 * (3./2. * hex.r),
                    0.0,
                ),
                scale: Vec3::new(0.5, 0.5, 1.0),
                ..Default::default()},
            ..Default::default()
        }).insert(hex);
    }
}