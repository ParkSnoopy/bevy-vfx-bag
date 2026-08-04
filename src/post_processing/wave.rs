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

use crate::post_processing::UniformBindGroup;

use super::Order;

const WAVE_SHADER_HANDLE: Handle<Shader> = uuid_handle!("00000000-0000-0000-18e0-cf0729a7ef50");

/// Wave parameters.
///
/// Note that the parameters for the X axis causes a wave effect
/// towards the left- and right sides of the screen.
/// For example, if we have 1 wave in the X axis,
/// we will have one part of the screen stretched towards the right
/// horizontally, and one part stretched towards the left.
#[derive(Default, Debug, Copy, Clone, Component, ShaderType)]
pub struct Wave {
    /// How many waves in the x axis.
    pub waves_x: f32,

    /// How many waves in the y axis.
    pub waves_y: f32,

    /// How fast the x axis waves oscillate.
    pub speed_x: f32,

    /// How fast the y axis waves oscillate.
    pub speed_y: f32,

    /// How much displacement the x axis waves cause.
    pub amplitude_x: f32,

    /// How much displacement the y axis waves cause.
    pub amplitude_y: f32,
}

#[derive(Resource)]
pub(crate) struct WaveData {
    pub pipeline_id: CachedRenderPipelineId,
    pub uniform_layout: BindGroupLayoutDescriptor,
}

impl FromWorld for WaveData {
    fn from_world(world: &mut World) -> Self {
        let (uniform_layout, pipeline_id) = super::create_layout_and_pipeline(
            world,
            "Wave",
            &[BindGroupLayoutEntry {
                binding: 0,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(Wave::min_size()),
                },
                visibility: ShaderStages::FRAGMENT,
                count: None,
            }],
            WAVE_SHADER_HANDLE.clone(),
        );

        WaveData {
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
            WAVE_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/", "wave.wgsl"),
            Shader::from_wgsl
        );

        // This puts the uniform into the render world.
        app.add_plugins((
            ExtractComponentPlugin::<Wave>::default(),
            UniformComponentPlugin::<Wave>::default(),
        ));

        super::render_app(app)
            .init_gpu_resource::<WaveData>()
            .init_resource::<UniformBindGroup<Wave>>()
            .add_systems(Render, queue.in_set(RenderSystems::PrepareBindGroups));
    }
}

fn queue(
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    data: Res<WaveData>,
    mut bind_group: ResMut<UniformBindGroup<Wave>>,
    uniforms: Res<ComponentUniforms<Wave>>,
    views: Query<Entity, With<Wave>>,
) {
    bind_group.inner = None;

    if let Some(uniforms) = uniforms.uniforms().binding() {
        if !views.is_empty() {
            bind_group.inner = Some(render_device.create_bind_group(
                "Wave Uniform Bind Group",
                &pipeline_cache.get_bind_group_layout(&data.uniform_layout),
                &[BindGroupEntry {
                    binding: 0,
                    resource: uniforms.clone(),
                }],
            ));
        }
    }
}

impl ExtractComponent for Wave {
    type QueryData = (&'static Self, Option<&'static Order<Self>>);
    type QueryFilter = ();
    type Out = (Self, Order<Self>);

    fn extract_component(
        (settings, order): QueryItem<'_, '_, Self::QueryData>,
    ) -> Option<Self::Out> {
        Some((*settings, order.copied().unwrap_or_else(|| Order::new(0.0))))
    }
}

impl SyncComponent for Wave {
    type Target = (Self, Order<Self>);
}
