use std::fmt::Display;

pub(crate) use bevy::{
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

use super::Order;
use crate::post_processing::UniformBindGroup;

pub(crate) const PIXELATE_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("00000000-0000-0000-99f5-ba26625aae60");

#[derive(Resource)]
pub(crate) struct PixelateData {
    pub pipeline_id: CachedRenderPipelineId,
    pub uniform_layout: BindGroupLayoutDescriptor,
}

impl FromWorld for PixelateData {
    fn from_world(world: &mut World) -> Self {
        let (uniform_layout, pipeline_id) = super::create_layout_and_pipeline(
            world,
            "Pixelate",
            &[BindGroupLayoutEntry {
                binding: 0,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(Pixelate::min_size()),
                },
                visibility: ShaderStages::FRAGMENT,
                count: None,
            }],
            PIXELATE_SHADER_HANDLE.clone(),
        );

        PixelateData {
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
            PIXELATE_SHADER_HANDLE,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/",
                "pixelate.wgsl"
            ),
            Shader::from_wgsl
        );

        // This puts the uniform into the render world.
        app.add_plugins((
            ExtractComponentPlugin::<Pixelate>::default(),
            UniformComponentPlugin::<Pixelate>::default(),
        ));

        super::render_app(app)
            .init_gpu_resource::<PixelateData>()
            .init_resource::<UniformBindGroup<Pixelate>>()
            .add_systems(Render, queue.in_set(RenderSystems::PrepareBindGroups));
    }
}

fn queue(
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    data: Res<PixelateData>,
    mut bind_group: ResMut<UniformBindGroup<Pixelate>>,
    uniforms: Res<ComponentUniforms<Pixelate>>,
    views: Query<Entity, With<Pixelate>>,
) {
    bind_group.inner = None;

    if let Some(uniforms) = uniforms.uniforms().binding() {
        if !views.is_empty() {
            bind_group.inner = Some(render_device.create_bind_group(
                "Pixelate Uniform Bind Group",
                &pipeline_cache.get_bind_group_layout(&data.uniform_layout),
                &[BindGroupEntry {
                    binding: 0,
                    resource: uniforms.clone(),
                }],
            ));
        }
    }
}

/// Pixelate settings.
#[derive(Debug, ShaderType, Component, Clone, Copy)]
pub struct Pixelate {
    /// How many pixels in the width and height in a block after pixelation. One block has a constant color within it.
    ///
    /// The shader sets a lower bound to 1.0, since that would not change the outcome.
    pub block_size: f32,
}

impl Default for Pixelate {
    fn default() -> Self {
        Self { block_size: 8.0 }
    }
}

impl Display for Pixelate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pixelate block size: {}", self.block_size)
    }
}

impl ExtractComponent for Pixelate {
    type QueryData = (&'static Self, Option<&'static Order<Self>>);
    type QueryFilter = ();
    type Out = (Self, Order<Self>);

    fn extract_component(
        (settings, order): QueryItem<'_, '_, Self::QueryData>,
    ) -> Option<Self::Out> {
        Some((*settings, order.copied().unwrap_or_else(|| Order::new(0.0))))
    }
}

impl SyncComponent for Pixelate {
    type Target = (Self, Order<Self>);
}
