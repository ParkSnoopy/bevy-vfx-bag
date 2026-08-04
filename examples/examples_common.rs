use bevy::{
    asset::RenderAssetUsages,
    color::palettes::css::{GOLD, SILVER},
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use core::f32::consts::PI;
use std::fmt::Display;

/// Adds some "sane defaults" for showing examples/development:
///
/// * The default Bevy plugins
/// * Hot reloading
/// * Close on ESC button press
pub struct SaneDefaultsPlugin;

#[allow(dead_code)]
pub(crate) fn print_on_change<T: Display + Component>(things: Query<&T, Changed<T>>) {
    for thing in &things {
        info!("{thing}");
    }
}

impl Plugin for SaneDefaultsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/assets").to_string(),
                    watch_for_changes_override: Some(true),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Update, close_on_esc);
    }
}

fn close_on_esc(
    mut commands: Commands,
    windows: Query<Entity, With<Window>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        for window in &windows {
            commands.entity(window).despawn();
        }
    }
}

/// This plugin combines two Bevy examples:
///
/// https://github.com/bevyengine/bevy/blob/v0.8.1/examples/3d/shapes.rs
/// https://github.com/bevyengine/bevy/blob/v0.8.1/examples/ui/text.rs
///
/// This example can be started by just loading this plugin.
/// This is useful to separate this crate's code and role from regular upstream Bevy code.
pub struct ShapesExamplePlugin {
    add_3d_camera_bundle: bool,
}

impl ShapesExamplePlugin {
    pub fn without_3d_camera() -> Self {
        Self {
            add_3d_camera_bundle: false,
        }
    }
}

#[derive(Resource)]
pub(crate) struct ShouldAdd3dCameraBundle(bool);

impl Plugin for ShapesExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShouldAdd3dCameraBundle(self.add_3d_camera_bundle))
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, (shapes::setup, ui::setup))
            .add_systems(Update, (shapes::rotate, ui::fps_text_update));
    }
}

#[derive(Component)]
pub(crate) struct Shape;

const X_EXTENT: f32 = 14.;

mod shapes {
    use super::*;

    pub(crate) fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut images: ResMut<Assets<Image>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        add_3d_camera_bundle: Res<ShouldAdd3dCameraBundle>,
    ) {
        let debug_material = materials.add(StandardMaterial {
            base_color_texture: Some(images.add(uv_debug_texture())),
            ..default()
        });

        let shapes = [
            meshes.add(Cuboid::default()),
            meshes.add(Cuboid::new(1.0, 0.5, 1.5)),
            meshes.add(Capsule3d::default()),
            meshes.add(Torus::default()),
            meshes.add(Sphere::new(0.5)),
            meshes.add(Sphere::new(0.75)),
        ];

        let num_shapes = shapes.len();

        for (i, shape) in shapes.into_iter().enumerate() {
            commands
                .spawn((
                    Mesh3d(shape.clone()),
                    MeshMaterial3d(debug_material.clone()),
                    Transform::from_xyz(
                        -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                        2.0,
                        0.0,
                    )
                    .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                ))
                .insert(Shape);

            commands
                .spawn((
                    Mesh3d(shape),
                    MeshMaterial3d(debug_material.clone()),
                    Transform::from_xyz(
                        -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                        2.0,
                        0.0,
                    )
                    .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                ))
                .insert(Shape);
        }

        commands.spawn((
            PointLight {
                intensity: 9000.0,
                range: 100.,
                shadow_maps_enabled: true,
                ..default()
            },
            Transform::from_xyz(8.0, 16.0, 8.0),
        ));

        // ground plane
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(50., 50.))),
            MeshMaterial3d(materials.add(Color::from(SILVER))),
        ));

        if add_3d_camera_bundle.0 {
            commands.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ));
        }
    }

    pub(crate) fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
        for mut transform in &mut query {
            transform.rotate_y(time.delta_secs() / 2.);
        }
    }

    /// Creates a colorful test pattern
    fn uv_debug_texture() -> Image {
        const TEXTURE_SIZE: usize = 8;

        let mut palette: [u8; 32] = [
            255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102,
            255, 198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
        ];

        let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
        for y in 0..TEXTURE_SIZE {
            let offset = TEXTURE_SIZE * y * 4;
            texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
            palette.rotate_right(4);
        }

        Image::new_fill(
            Extent3d {
                width: TEXTURE_SIZE as u32,
                height: TEXTURE_SIZE as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &texture_data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        )
    }
}

////////////////////////////////////////////////////////////////////////////////
// UI
////////////////////////////////////////////////////////////////////////////////

mod ui {
    use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

    use super::*;

    // A unit struct to help identify the FPS UI component, since there may be many Text components
    #[derive(Component)]
    pub(crate) struct FpsText;

    pub(crate) fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        // Text with multiple sections
        commands.spawn((
            Text::new("FPS: "),
            TextFont {
                font: asset_server.load("fonts/FiraMono-Medium.ttf").into(),
                font_size: FontSize::Px(40.0),
                ..default()
            },
            TextColor(GOLD.into()),
            Node {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                top: px(5),
                right: px(15),
                ..default()
            },
            FpsText,
        ));
    }

    pub(crate) fn fps_text_update(
        diagnostics: Res<DiagnosticsStore>,
        mut query: Query<&mut Text, With<FpsText>>,
    ) {
        for mut text in &mut query {
            if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(average) = fps.average() {
                    // Update the value of the second section
                    **text = format!("FPS: {average:.2}");
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// MAIN
////////////////////////////////////////////////////////////////////////////////
#[allow(dead_code)]
fn main() {
    println!("Not an example, just shared code between examples")
}
