#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy_vfx_bag::{BevyVfxBagPlugin, post_processing::blur::Blur};

fn main() {
    let mut app = App::new();

    app.add_plugins(examples_common::SaneDefaultsPlugin)
        .add_plugins(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugins(BevyVfxBagPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, (examples_common::print_on_change::<Blur>, update))
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        Blur::default(),
    ));
}

fn update(keyboard_input: Res<ButtonInput<KeyCode>>, mut blur: Query<&mut Blur>) {
    let mut blur = blur.single_mut().expect("exactly one camera");

    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        blur.kernel_radius -= 0.001;
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        blur.kernel_radius += 0.001;
    }

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        blur.amount += 0.1;
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        blur.amount -= 0.1;
    }
}
