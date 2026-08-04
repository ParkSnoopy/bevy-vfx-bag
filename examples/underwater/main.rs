#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;
use bevy_vfx_bag::{
    BevyVfxBagPlugin,
    post_processing::{
        chromatic_aberration::ChromaticAberration, lut::Lut, masks::Mask, raindrops::Raindrops,
        wave::Wave,
    },
};

fn main() {
    let mut app = App::new();

    app.add_plugins(examples_common::SaneDefaultsPlugin)
        .add_plugins(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugins(BevyVfxBagPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        Raindrops::default(),
        ChromaticAberration {
            magnitude_r: 0.003,
            magnitude_g: 0.003,
            magnitude_b: 0.003,
            ..default()
        },
        Wave {
            waves_x: 1.,
            speed_x: 0.1,
            amplitude_x: 0.07,
            waves_y: 10.,
            speed_y: 0.3,
            amplitude_y: 0.01,
        },
        Lut::arctic(),
        Mask::vignette(),
    ));
}
