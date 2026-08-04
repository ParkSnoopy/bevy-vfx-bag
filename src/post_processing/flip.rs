use bevy::{
    asset::{load_internal_asset, uuid_handle},
    ecs::query::QueryItem,
    prelude::*,
    render::{
        GpuResourceAppExt, Render, RenderSystems,
        extract_component::{
            ComponentUniforms, ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin,
        },
        render_resource::{
            BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            BufferBindingType, CachedRenderPipelineId, PipelineCache, ShaderStages, ShaderType,
        },
        renderer::RenderDevice,
        sync_component::SyncComponent,
    },
    shader::Shader,
};
use std::fmt::Display;

use super::Order;
use crate::post_processing::UniformBindGroup;

pub(crate) const FLIP_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("00000000-0000-0000-16e5-811cca736853");

#[derive(Resource)]
pub(crate) struct FlipData {
    pub pipeline_id: CachedRenderPipelineId,
    pub uniform_layout: BindGroupLayoutDescriptor,
}

impl FromWorld for FlipData {
    fn from_world(world: &mut World) -> Self {
        let (uniform_layout, pipeline_id) = super::create_layout_and_pipeline(
            world,
            "Flip",
            &[BindGroupLayoutEntry {
                binding: 0,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(FlipUniform::min_size()),
                },
                visibility: ShaderStages::FRAGMENT,
                count: None,
            }],
            FLIP_SHADER_HANDLE.clone(),
        );

        FlipData {
            pipeline_id,
            uniform_layout,
        }
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            FLIP_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/", "flip.wgsl"),
            Shader::from_wgsl
        );

        // This puts the uniform into the render world.
        app.add_plugins((
            ExtractComponentPlugin::<Flip>::default(),
            UniformComponentPlugin::<FlipUniform>::default(),
        ));

        super::render_app(app)
            .init_gpu_resource::<FlipData>()
            .init_resource::<UniformBindGroup<FlipUniform>>()
            .add_systems(Render, queue.in_set(RenderSystems::PrepareBindGroups));
    }
}

fn queue(
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    data: Res<FlipData>,
    mut bind_group: ResMut<UniformBindGroup<FlipUniform>>,
    uniforms: Res<ComponentUniforms<FlipUniform>>,
    views: Query<Entity, With<FlipUniform>>,
) {
    bind_group.inner = None;

    if let Some(uniforms) = uniforms.uniforms().binding() {
        if !views.is_empty() {
            bind_group.inner = Some(render_device.create_bind_group(
                "Flip Uniform Bind Group",
                &pipeline_cache.get_bind_group_layout(&data.uniform_layout),
                &[BindGroupEntry {
                    binding: 0,
                    resource: uniforms.clone(),
                }],
            ));
        }
    }
}

#[doc(hidden)]
/// The uniform representation of [`Flip`].
#[derive(Debug, ShaderType, Clone, Component)]
pub struct FlipUniform {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl From<Flip> for FlipUniform {
    fn from(flip: Flip) -> Self {
        let uv = match flip {
            Flip::None => [0.0, 0.0],
            Flip::Horizontal => [1.0, 0.0],
            Flip::Vertical => [0.0, 1.0],
            Flip::HorizontalVertical => [1.0, 1.0],
        };

        Self { x: uv[0], y: uv[1] }
    }
}

/// Which way to flip the texture.
#[derive(Debug, Default, Copy, Clone, Component)]
pub enum Flip {
    /// Don't flip.
    None,

    /// Flip horizontally.
    #[default]
    Horizontal,

    /// Flip vertically.
    Vertical,

    /// Flip both axes.
    HorizontalVertical,
}

impl Display for Flip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl ExtractComponent for Flip {
    type QueryData = (&'static Self, Option<&'static Order<Self>>);
    type QueryFilter = ();
    type Out = (FlipUniform, Order<Self>);

    fn extract_component(
        (settings, order): QueryItem<'_, '_, Self::QueryData>,
    ) -> Option<Self::Out> {
        Some((
            (*settings).into(),
            order.copied().unwrap_or_else(|| Order::new(0.0)),
        ))
    }
}

impl SyncComponent for Flip {
    type Target = (FlipUniform, Order<Self>);
}
