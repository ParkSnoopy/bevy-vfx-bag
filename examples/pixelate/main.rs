//! This example shows the pixelation effect as well as
//! how to toggle a post processing effect at runtime.
//! All post processing effects may be toggled as such.
#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;
use bevy_vfx_bag::{BevyVfxBagPlugin, post_processing::pixelate::Pixelate};

fn main() {
    let mut app = App::new();

    app.add_plugins(examples_common::SaneDefaultsPlugin)
        .add_plugins(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugins(BevyVfxBagPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (examples_common::print_on_change::<Pixelate>, update),
        )
        .run();
}

fn startup(mut commands: Commands) {
    info!("Press [t] to toggle, [up/down] to change");

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        Pixelate::default(),
    ));
}

fn update(
    mut saved_settings: Local<Pixelate>,
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, Option<&mut Pixelate>), With<Camera>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        match query.single().expect("exactly one camera") {
            (entity, None) => {
                info!("Toggling ON");
                commands.entity(entity).insert(*saved_settings);
            }
            (entity, Some(settings)) => {
                info!("Toggling OFF");
                commands.entity(entity).remove::<Pixelate>();
                *saved_settings = *settings;
            }
        };
    }

    if let (_, Some(mut settings)) = query.single_mut().expect("exactly one camera") {
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            settings.block_size += 1.0;
        } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            settings.block_size -= 1.0;
        };
    }
}
