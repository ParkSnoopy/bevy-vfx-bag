#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;
use bevy_vfx_bag::{BevyVfxBagPlugin, post_processing::wave::Wave};

#[derive(Debug, Resource, Default)]
struct SlowerTime(Time);

fn main() {
    App::new()
        .add_plugins(examples_common::SaneDefaultsPlugin)
        .add_plugins(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugins(BevyVfxBagPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));
}

fn update(
    mut command: Commands,
    time: Res<Time>,
    query: Query<(Entity, Option<&Wave>), With<Camera>>,
) {
    if time.elapsed_secs().fract() < 0.8 {
        if let (entity, Some(_)) = query.single().expect("exactly one camera") {
            command.entity(entity).remove::<Wave>();
            info!("Is that a T-Rex approaching?!");
        }
    } else if let (entity, None) = query.single().expect("exactly one camera") {
        command.entity(entity).insert(Wave {
            waves_x: 2.0,
            waves_y: 0.1,
            speed_x: 30.,
            speed_y: 20.,
            amplitude_x: 0.01,
            amplitude_y: 0.01,
        });
        info!("<GROUND SHAKE>");
    }
}
