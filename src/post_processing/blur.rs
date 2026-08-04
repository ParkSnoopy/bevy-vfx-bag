use std::fmt::Display;

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

use super::Order;
use crate::post_processing::UniformBindGroup;

pub(crate) const BLUR_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("00000000-0000-0000-9945-11c86bfc7f35");

#[derive(Resource)]
pub(crate) struct BlurData {
    pub pipeline_id: CachedRenderPipelineId,
    pub uniform_layout: BindGroupLayoutDescriptor,
}

impl FromWorld for BlurData {
    fn from_world(world: &mut World) -> Self {
        let (uniform_layout, pipeline_id) = super::create_layout_and_pipeline(
            world,
            "Blur",
            &[BindGroupLayoutEntry {
                binding: 0,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(Blur::min_size()),
                },
                visibility: ShaderStages::FRAGMENT,
                count: None,
            }],
            BLUR_SHADER_HANDLE.clone(),
        );

        BlurData {
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
            BLUR_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/", "blur.wgsl"),
            Shader::from_wgsl
        );

        // This puts the uniform into the render world.
        app.add_plugins((
            ExtractComponentPlugin::<Blur>::default(),
            UniformComponentPlugin::<Blur>::default(),
        ));

        super::render_app(app)
            .init_gpu_resource::<BlurData>()
            .init_resource::<UniformBindGroup<Blur>>()
            .add_systems(Render, queue.in_set(RenderSystems::PrepareBindGroups));
    }
}

fn queue(
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    data: Res<BlurData>,
    mut bind_group: ResMut<UniformBindGroup<Blur>>,
    uniforms: Res<ComponentUniforms<Blur>>,
    views: Query<Entity, With<Blur>>,
) {
    bind_group.inner = None;

    if let Some(uniforms) = uniforms.uniforms().binding() {
        if !views.is_empty() {
            bind_group.inner = Some(render_device.create_bind_group(
                "Blur Uniform Bind Group",
                &pipeline_cache.get_bind_group_layout(&data.uniform_layout),
                &[BindGroupEntry {
                    binding: 0,
                    resource: uniforms.clone(),
                }],
            ));
        }
    }
}

/// Blur settings.
#[derive(Debug, Copy, Clone, Component, ShaderType)]
pub struct Blur {
    /// How blurry the output image should be.
    /// If `0.0`, no blur is applied.
    /// `1.0` is "fully blurred", but higher values will produce interesting results.
    pub amount: f32,

    /// How far away the blur should sample points away from the origin point
    /// when blurring.
    /// This is in UV coordinates, so small (positive) values are expected (`0.01` is a good start).
    pub kernel_radius: f32,
}

impl Default for Blur {
    fn default() -> Self {
        Self {
            amount: 0.5,
            kernel_radius: 0.01,
        }
    }
}

impl Display for Blur {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Blur amount: {}, radius: {}",
            self.amount, self.kernel_radius
        )
    }
}

impl ExtractComponent for Blur {
    type QueryData = (&'static Self, Option<&'static Order<Self>>);
    type QueryFilter = ();
    type Out = (Self, Order<Self>);

    fn extract_component(
        (settings, order): QueryItem<'_, '_, Self::QueryData>,
    ) -> Option<Self::Out> {
        Some((*settings, order.copied().unwrap_or_else(|| Order::new(0.0))))
    }
}

impl SyncComponent for Blur {
    type Target = (Self, Order<Self>);
}
