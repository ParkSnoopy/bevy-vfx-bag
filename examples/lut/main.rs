#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;
use bevy_vfx_bag::{BevyVfxBagPlugin, post_processing::lut::Lut};

fn main() {
    let mut app = App::new();

    app.add_plugins(examples_common::SaneDefaultsPlugin)
        .add_plugins(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugins(BevyVfxBagPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .run();
}

fn startup(mut commands: Commands) {
    info!("Press [left|right] to change which LUT is in use");

    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ))
        .insert(Lut::default());
}

// Cycle through some preset LUTs.
fn update(
    mut choice: Local<usize>,
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<Entity, With<Camera>>,
) {
    let choice_now = if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        choice.saturating_sub(1)
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        (*choice + 1).min(3)
    } else {
        *choice
    };

    if *choice != choice_now {
        let entity = query.single_mut().expect("exactly one camera");

        *choice = choice_now;
        match *choice {
            0 => {
                commands.entity(entity).insert(Lut::neo());
                info!("Neo");
            }
            1 => {
                commands.entity(entity).insert(Lut::arctic());
                info!("Arctic");
            }
            2 => {
                commands.entity(entity).insert(Lut::slate());
                info!("Slate");
            }
            3 => {
                commands.entity(entity).remove::<Lut>();
                info!("Disabled (default Bevy colors)");
            }
            _ => unreachable!(),
        }
    }
}
