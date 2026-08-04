use std::{collections::HashMap, marker::PhantomData};

use bevy::{
    core_pipeline::{
        Core2dSystems, Core3dSystems, FullscreenShader,
        schedule::{Core2d, Core3d},
    },
    ecs::{query::QueryData, system::SystemParam},
    prelude::*,
    render::{
        GpuResourceAppExt, RenderApp,
        extract_component::DynamicUniformIndex,
        globals::{GlobalsBuffer, GlobalsUniform},
        render_resource::{
            BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
            BindingResource, BindingType, BufferBindingType, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, FilterMode, FragmentState, PipelineCache,
            RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
            SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType, TextureFormat,
            TextureSampleType, TextureViewDimension, TextureViewId,
        },
        renderer::{RenderContext, ViewQuery},
        view::ViewTarget,
    },
    shader::{Shader, ShaderDefVal},
};

/// Blur
pub mod blur;
/// Chromatic Aberration
pub mod chromatic_aberration;
/// Flip
pub mod flip;
/// LUT
pub mod lut;
/// Masks
pub mod masks;
/// Pixelate
pub mod pixelate;
/// Raindrops
pub mod raindrops;
/// Wave
pub mod wave;

use blur::{Blur, BlurData};
use chromatic_aberration::{ChromaticAberration, ChromaticAberrationData};
use flip::{Flip, FlipData, FlipUniform};
use lut::{Lut, LutBindGroup, LutData};
use masks::{Mask, MaskData, MaskUniform, MaskVariant};
use pixelate::{Pixelate, PixelateData};
use raindrops::{Raindrops, RaindropsData};
use wave::{Wave, WaveData};

#[derive(Resource)]
pub(crate) struct UniformBindGroup<U: ShaderType> {
    pub inner: Option<BindGroup>,
    marker: PhantomData<U>,
}

impl<U: ShaderType> Default for UniformBindGroup<U> {
    fn default() -> Self {
        Self {
            inner: None,
            marker: PhantomData,
        }
    }
}

/// Adds a `.order` helper method to a component.
/// When used on a post processing effect, it determines the order in which the effect is applied.
///
/// See [`Order`] for more information.
pub trait PostProcessingOrder: Sized {
    /// Sets the order value on a component and returns it with its ordering component.
    fn order(self, order: f32) -> (Self, Order<Self>);
}

impl<U: Component> PostProcessingOrder for U {
    fn order(self, order: f32) -> (Self, Order<Self>) {
        (self, Order::new(order))
    }
}

pub(crate) fn create_layout(
    world: &mut World,
    label: &'static str,
    layout_entries: &[BindGroupLayoutEntry],
) -> BindGroupLayoutDescriptor {
    let _ = world;
    BindGroupLayoutDescriptor::new(label, layout_entries)
}

pub(crate) fn render_pipeline_descriptor(
    world: &World,
    label: &'static str,
    shared_layout: &BindGroupLayoutDescriptor,
    uniform_layout: &BindGroupLayoutDescriptor,
    shader: Handle<Shader>,
    shader_defs: Vec<ShaderDefVal>,
) -> RenderPipelineDescriptor {
    let target_format = if cfg!(feature = "hdr") {
        TextureFormat::Rgba16Float
    } else {
        TextureFormat::Rgba8UnormSrgb
    };

    RenderPipelineDescriptor {
        label: Some(format!("{label} Render Pipeline").into()),
        layout: vec![shared_layout.clone(), uniform_layout.clone()],
        vertex: world.resource::<FullscreenShader>().to_vertex_state(),
        fragment: Some(FragmentState {
            shader,
            shader_defs,
            entry_point: Some("fragment".into()),
            targets: vec![Some(ColorTargetState {
                format: target_format,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
        }),
        ..default()
    }
}

pub(crate) fn create_pipeline(
    world: &mut World,
    label: &'static str,
    uniform_layout: &BindGroupLayoutDescriptor,
    shader: Handle<Shader>,
    shader_definitions: Vec<ShaderDefVal>,
) -> CachedRenderPipelineId {
    let shared_layout = world
        .resource::<PostProcessingSharedLayout>()
        .shared_layout
        .clone();
    let descriptor = render_pipeline_descriptor(
        world,
        label,
        &shared_layout,
        uniform_layout,
        shader,
        shader_definitions,
    );
    world
        .resource::<PipelineCache>()
        .queue_render_pipeline(descriptor)
}

pub(crate) fn create_layout_and_pipeline(
    world: &mut World,
    label: &'static str,
    layout_entries: &[BindGroupLayoutEntry],
    shader: Handle<Shader>,
) -> (BindGroupLayoutDescriptor, CachedRenderPipelineId) {
    let uniform_layout = create_layout(world, label, layout_entries);
    let pipeline_id = create_pipeline(world, label, &uniform_layout, shader, vec![]);
    (uniform_layout, pipeline_id)
}

#[derive(Debug, Resource, Clone)]
pub(crate) struct PostProcessingSharedLayout {
    shared_layout: BindGroupLayoutDescriptor,
}

impl FromWorld for PostProcessingSharedLayout {
    fn from_world(world: &mut World) -> Self {
        let _ = world;
        let shared_layout = BindGroupLayoutDescriptor::new(
            "PostProcessing texture bind group layout",
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GlobalsUniform::min_size()),
                    },
                    count: None,
                },
            ],
        );
        Self { shared_layout }
    }
}

#[derive(Default)]
struct SourceBindGroupCache {
    bind_groups: HashMap<TextureViewId, BindGroup>,
}

struct RenderEffect<'a> {
    order: f32,
    pipeline: CachedRenderPipelineId,
    bind_group: &'a BindGroup,
    dynamic_offset: Option<u32>,
}

#[derive(QueryData)]
struct PostProcessingView {
    view_target: &'static ViewTarget,
    blur: Option<(
        &'static Blur,
        &'static DynamicUniformIndex<Blur>,
        &'static Order<Blur>,
    )>,
    chromatic_aberration: Option<(
        &'static ChromaticAberration,
        &'static DynamicUniformIndex<ChromaticAberration>,
        &'static Order<ChromaticAberration>,
    )>,
    flip: Option<(
        &'static FlipUniform,
        &'static DynamicUniformIndex<FlipUniform>,
        &'static Order<Flip>,
    )>,
    lut: Option<(&'static Lut, &'static LutBindGroup, &'static Order<Lut>)>,
    mask: Option<(
        &'static MaskUniform,
        &'static DynamicUniformIndex<MaskUniform>,
        &'static MaskVariant,
        &'static Order<Mask>,
    )>,
    pixelate: Option<(
        &'static Pixelate,
        &'static DynamicUniformIndex<Pixelate>,
        &'static Order<Pixelate>,
    )>,
    raindrops: Option<(
        &'static Raindrops,
        &'static DynamicUniformIndex<Raindrops>,
        &'static Order<Raindrops>,
    )>,
    wave: Option<(
        &'static Wave,
        &'static DynamicUniformIndex<Wave>,
        &'static Order<Wave>,
    )>,
}

#[derive(SystemParam)]
struct PostProcessingResources<'w> {
    pipeline_cache: Res<'w, PipelineCache>,
    globals: Res<'w, GlobalsBuffer>,
    shared_layout: Res<'w, PostProcessingSharedLayout>,
    blur_data: Res<'w, BlurData>,
    blur_bind_group: Res<'w, UniformBindGroup<Blur>>,
    chromatic_aberration_data: Res<'w, ChromaticAberrationData>,
    chromatic_aberration_bind_group: Res<'w, UniformBindGroup<ChromaticAberration>>,
    flip_data: Res<'w, FlipData>,
    flip_bind_group: Res<'w, UniformBindGroup<FlipUniform>>,
    lut_data: Res<'w, LutData>,
    mask_data: Res<'w, MaskData>,
    mask_bind_group: Res<'w, UniformBindGroup<MaskUniform>>,
    pixelate_data: Res<'w, PixelateData>,
    pixelate_bind_group: Res<'w, UniformBindGroup<Pixelate>>,
    raindrops_data: Res<'w, RaindropsData>,
    raindrops_bind_group: Res<'w, UniformBindGroup<Raindrops>>,
    wave_data: Res<'w, WaveData>,
    wave_bind_group: Res<'w, UniformBindGroup<Wave>>,
}

fn post_process_system(
    view: ViewQuery<PostProcessingView>,
    resources: PostProcessingResources,
    mut source_cache: Local<SourceBindGroupCache>,
    mut context: RenderContext,
) {
    let view = view.into_inner();
    let mut effects = Vec::with_capacity(8);

    macro_rules! push_uniform_effect {
        ($value:expr, $data:expr, $bind_group:expr) => {
            if let (Some((_settings, index, order)), Some(bind_group)) =
                ($value, $bind_group.inner.as_ref())
            {
                effects.push(RenderEffect {
                    order: order.order,
                    pipeline: $data.pipeline_id,
                    bind_group,
                    dynamic_offset: Some(index.index()),
                });
            }
        };
    }

    push_uniform_effect!(view.blur, resources.blur_data, resources.blur_bind_group);
    push_uniform_effect!(
        view.chromatic_aberration,
        resources.chromatic_aberration_data,
        resources.chromatic_aberration_bind_group
    );
    push_uniform_effect!(view.flip, resources.flip_data, resources.flip_bind_group);
    push_uniform_effect!(
        view.pixelate,
        resources.pixelate_data,
        resources.pixelate_bind_group
    );
    push_uniform_effect!(
        view.raindrops,
        resources.raindrops_data,
        resources.raindrops_bind_group
    );
    push_uniform_effect!(view.wave, resources.wave_data, resources.wave_bind_group);

    if let Some((_lut, bind_group, order)) = view.lut {
        effects.push(RenderEffect {
            order: order.order,
            pipeline: resources.lut_data.pipeline_id,
            bind_group: &bind_group.bind_group,
            dynamic_offset: None,
        });
    }

    if let (Some((_mask, index, variant, order)), Some(bind_group)) =
        (view.mask, resources.mask_bind_group.inner.as_ref())
    {
        effects.push(RenderEffect {
            order: order.order,
            pipeline: resources.mask_data.pipeline_id(*variant),
            bind_group,
            dynamic_offset: Some(index.index()),
        });
    }

    effects.sort_by(|a, b| a.order.total_cmp(&b.order));
    let Some(globals_binding) = resources.globals.buffer.binding() else {
        return;
    };

    for effect in effects {
        let Some(pipeline) = resources
            .pipeline_cache
            .get_render_pipeline(effect.pipeline)
        else {
            continue;
        };
        let post_process = view.view_target.post_process_write();
        let source_id = post_process.source.id();
        let source_bind_group = source_cache
            .bind_groups
            .entry(source_id)
            .or_insert_with(|| {
                context.render_device().create_bind_group(
                    "PostProcessing source bind group",
                    &resources
                        .pipeline_cache
                        .get_bind_group_layout(&resources.shared_layout.shared_layout),
                    &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(post_process.source),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::Sampler(
                                &context.render_device().create_sampler(&SamplerDescriptor {
                                    mag_filter: FilterMode::Linear,
                                    min_filter: FilterMode::Linear,
                                    ..default()
                                }),
                            ),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: globals_binding.clone(),
                        },
                    ],
                )
            });

        let mut pass = context
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("PostProcessing pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: post_process.destination,
                    depth_slice: None,
                    resolve_target: None,
                    ops: default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, source_bind_group, &[]);
        let offsets = effect.dynamic_offset.as_slice();
        pass.set_bind_group(1, effect.bind_group, offsets);
        pass.draw(0..3, 0..1);
    }
}

/// Decide on ordering for post processing effects. Lower numbers run earlier.
#[derive(Debug, Component)]
pub struct Order<C> {
    /// Priority.
    pub order: f32,
    marker: PhantomData<C>,
}

impl<C> Clone for Order<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C> Copy for Order<C> {}

impl<C> Order<C> {
    /// Create a new ordering.
    pub fn new(priority: f32) -> Self {
        Self {
            order: priority,
            marker: PhantomData,
        }
    }
}

pub(crate) fn render_app(app: &mut App) -> &mut SubApp {
    app.get_sub_app_mut(RenderApp)
        .expect("Need a render app for post processing")
}

#[derive(Debug, Default)]
pub(crate) struct PostProcessingPlugin;

impl Plugin for PostProcessingPlugin {
    fn build(&self, app: &mut App) {
        {
            let render_app = app
                .get_sub_app_mut(RenderApp)
                .expect("Need a render app for post processing");
            render_app
                .init_gpu_resource::<PostProcessingSharedLayout>()
                .add_systems(
                    Core3d,
                    post_process_system.in_set(Core3dSystems::PostProcess),
                )
                .add_systems(
                    Core2d,
                    post_process_system.in_set(Core2dSystems::PostProcess),
                );
        }

        app.add_plugins((
            blur::Plugin,
            chromatic_aberration::Plugin,
            flip::Plugin,
            lut::Plugin,
            masks::Plugin,
            raindrops::Plugin,
            pixelate::Plugin,
            wave::Plugin,
        ));
    }
}
