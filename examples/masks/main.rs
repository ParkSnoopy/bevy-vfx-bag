#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;
use bevy_vfx_bag::{
    BevyVfxBagPlugin,
    post_processing::masks::{Mask, MaskVariant},
};

fn main() {
    let mut app = App::new();

    app.add_plugins(examples_common::SaneDefaultsPlugin)
        .add_plugins(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugins(BevyVfxBagPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, (update, examples_common::print_on_change::<Mask>))
        .run();
}

fn startup(mut commands: Commands) {
    info!(
        "Press [1|2|3] to change which mask is in use, [Up|Down] to change strenght, [L|H] to go low/high [PgUp/PgDown] to fade in/out the mask"
    );

    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ))
        .insert(Mask::default());
}

fn update(keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Mask, With<Camera>>) {
    let mut mask = query.single_mut().expect("exactly one camera");

    if keyboard_input.just_pressed(KeyCode::Digit1) {
        *mask = Mask::square();
    } else if keyboard_input.just_pressed(KeyCode::Digit2) {
        *mask = Mask::crt();
    } else if keyboard_input.just_pressed(KeyCode::Digit3) {
        *mask = Mask::vignette();
    };

    // Let user change strength in increments via up, down arrows
    let increment = || match mask.variant {
        MaskVariant::Square => 1.,
        MaskVariant::Crt => 1000.,
        MaskVariant::Vignette => 0.05,
    };

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        mask.strength += increment();
    } else if keyboard_input.pressed(KeyCode::ArrowDown) {
        mask.strength -= increment();
    };

    if keyboard_input.pressed(KeyCode::PageUp) {
        mask.fade += 0.01;
    } else if keyboard_input.pressed(KeyCode::PageDown) {
        mask.fade -= 0.01;
    };

    mask.fade = mask.fade.clamp(0.0, 1.0);

    // Let user go to low- and high strength values directly via L and H keys
    let low = || match mask.variant {
        MaskVariant::Square => 3.,
        MaskVariant::Crt => 3000.,
        MaskVariant::Vignette => 0.1,
    };

    let high = || match mask.variant {
        MaskVariant::Square => 100.,
        MaskVariant::Crt => 500000.,
        MaskVariant::Vignette => 1.5,
    };

    if keyboard_input.just_pressed(KeyCode::KeyL) {
        mask.strength = low();
    } else if keyboard_input.just_pressed(KeyCode::KeyH) {
        mask.strength = high();
    };
}
