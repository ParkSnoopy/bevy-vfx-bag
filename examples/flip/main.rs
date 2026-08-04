#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;
use bevy_vfx_bag::{BevyVfxBagPlugin, post_processing::flip::Flip};

fn main() {
    let mut app = App::new();

    app.add_plugins(examples_common::SaneDefaultsPlugin)
        .add_plugins(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugins(BevyVfxBagPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, examples_common::print_on_change::<Flip>)
        .add_systems(FixedUpdate, update)
        .insert_resource(Time::<Fixed>::from_seconds(1.5))
        .run();
}

fn startup(mut commands: Commands) {
    info!("Flips the screen orientation every interval.");

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        Flip::default(),
    ));
}

// Switch flip modes every second.
fn update(mut query: Query<&mut Flip, With<Camera>>) {
    let mut flip = query.single_mut().expect("exactly one camera");

    *flip = match *flip {
        Flip::None => Flip::Horizontal,
        Flip::Horizontal => Flip::Vertical,
        Flip::Vertical => Flip::HorizontalVertical,
        Flip::HorizontalVertical => Flip::None,
    };
}
