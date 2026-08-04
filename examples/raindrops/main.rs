#[path = "../examples_common.rs"]
mod examples_common;

use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_vfx_bag::{BevyVfxBagPlugin, post_processing::raindrops::Raindrops};

fn main() {
    let mut app = App::new();

    app.add_plugins(examples_common::SaneDefaultsPlugin)
        .add_plugins(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugins(BevyVfxBagPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (examples_common::print_on_change::<Raindrops>, update),
        )
        .run();
}

fn startup(mut commands: Commands) {
    info!("Press [up|down|left|right|mouse scroll] to change settings");

    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ))
        .insert(Raindrops::default());
}

fn update(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut query: Query<&mut Raindrops, With<Camera>>,
) {
    let mut raindrops = query.single_mut().expect("exactly one camera");

    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        raindrops.speed -= 0.1;
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        raindrops.speed += 0.1;
    }

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        raindrops.warping += 0.01;
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        raindrops.warping -= 0.01;
    }

    for scroll in mouse_wheel_events.read() {
        if scroll.y > 0.0 {
            raindrops.zoom += 0.1;
        } else if scroll.y < 0.0 {
            raindrops.zoom -= 0.1;
        }
    }
}
