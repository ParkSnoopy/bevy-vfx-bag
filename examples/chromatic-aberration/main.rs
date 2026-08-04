//! This example shows the chromatic aberration effect as well as
//! changing a post processing effect's settings over time.

#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;
use bevy_vfx_bag::{BevyVfxBagPlugin, post_processing::chromatic_aberration::ChromaticAberration};

fn main() {
    let mut app = App::new();

    app.add_plugins(examples_common::SaneDefaultsPlugin)
        .add_plugins(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugins(BevyVfxBagPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                examples_common::print_on_change::<ChromaticAberration>,
                update,
            ),
        )
        .run();
}

fn startup(mut commands: Commands) {
    info!("Press [up/down] to change");

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ChromaticAberration::default(),
    ));
}

fn update(
    time: Res<Time>,
    mut query: Query<&mut ChromaticAberration, With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut chromatic_aberration = query.single_mut().expect("exactly one camera");

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        chromatic_aberration.add_magnitude(0.001);
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        chromatic_aberration.add_magnitude(-0.001);
    }

    let t = time.elapsed_secs();

    chromatic_aberration.dir_r = Vec2::from_angle(t);
    chromatic_aberration.dir_g = Vec2::from_angle(2. * t);
    chromatic_aberration.dir_b = Vec2::from_angle(3. * t);
}
