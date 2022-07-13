use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_loopless_state(GameState::Start)
            .add_enter_system(GameState::Start, start_menu);
    }
}

fn start_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(TextBundle {
        text: Text::with_section(
            "Use The Arrow Keys To Move",
            TextStyle {
                font: asset_server.load("fonts/BebasNeue-Regular.ttf"),
                font_size: 100.0,
                color: Color::WHITE,
            },
            TextAlignment {
                horizontal: HorizontalAlign::Center,
                ..Default::default()
            },
        ),
        ..default()
    });
}
