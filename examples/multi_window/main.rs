#[path = "../examples_common.rs"]
mod examples_common;

use bevy::{
    camera::RenderTarget,
    prelude::*,
    window::{Window, WindowRef},
};

use bevy_vfx_bag::{
    BevyVfxBagPlugin,
    post_processing::{
        PostProcessingOrder, blur::Blur, chromatic_aberration::ChromaticAberration, flip::Flip,
        lut::Lut, masks::Mask, pixelate::Pixelate, raindrops::Raindrops, wave::Wave,
    },
};

fn main() {
    let mut app = App::new();

    app.add_plugins(examples_common::SaneDefaultsPlugin)
        .add_plugins(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugins(BevyVfxBagPlugin::default())
        .add_systems(Startup, setup);

    app.run();
}

fn setup(mut commands: Commands) {
    let transform = Transform::from_xyz(-5.0, 12., 10.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y);

    // First window: Clean, no effects
    commands.spawn((Camera3d::default(), transform));

    // Second window: Camera has effects
    let window_2 = commands.spawn(Window::default()).id();
    commands.spawn((
        Camera3d::default(),
        Camera { ..default() },
        RenderTarget::Window(WindowRef::Entity(window_2)),
        transform,
        Wave::default().order(0.),
        Pixelate::default().order(1.),
        Mask::default().order(2.),
        Lut::default().order(3.),
        Blur::default().order(4.),
        Flip::default().order(5.),
    ));

    // Third window: Camera has other effects
    let window_3 = commands.spawn(Window::default()).id();
    commands.spawn((
        Camera3d::default(),
        Camera { ..default() },
        RenderTarget::Window(WindowRef::Entity(window_3)),
        transform,
        Mask::crt().order(0.),
        Lut::arctic().order(1.),
        ChromaticAberration::default().order(2.),
        Raindrops::default().order(3.),
    ));
}
