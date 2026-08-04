use std::fmt::Display;

use bevy::render::{
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    render_asset::RenderAssets,
    render_resource::{
        AddressMode, BindingResource, Sampler, SamplerBindingType, SamplerDescriptor,
        TextureSampleType, TextureViewDimension,
    },
    texture::GpuImage,
};
pub(crate) use bevy::{
    asset::{RenderAssetUsages, load_internal_asset, uuid_handle},
    ecs::query::QueryItem,
    image::{CompressedImageFormats, ImageSampler, ImageType},
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

pub(crate) const RAINDROPS_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("00000000-0000-0000-304f-b7ae02f5ce73");
const RAINDROPS_IMAGE_HANDLE: Handle<Image> = uuid_handle!("00000000-0000-0000-931d-ba3b46c1708f");

#[derive(Resource, ExtractResource, Deref, DerefMut, Clone)]
struct RaindropsTextureHandle(Handle<Image>);

#[derive(Resource)]
pub(crate) struct RaindropsData {
    pub pipeline_id: CachedRenderPipelineId,
    pub layout: BindGroupLayoutDescriptor,
    pub sampler: Sampler,
}

impl FromWorld for RaindropsData {
    fn from_world(world: &mut World) -> Self {
        let (raindrops_layout, pipeline_id) = super::create_layout_and_pipeline(
            world,
            "Raindrops",
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
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(Raindrops::min_size()),
                    },
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                },
            ],
            RAINDROPS_SHADER_HANDLE.clone(),
        );

        let raindrops_sampler = world
            .get_resource::<RenderDevice>()
            .expect("Should have render device")
            .create_sampler(&SamplerDescriptor {
                label: Some("Raindrops Sampler"),
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
                ..default()
            });

        RaindropsData {
            pipeline_id,
            layout: raindrops_layout,
            sampler: raindrops_sampler,
        }
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            RAINDROPS_SHADER_HANDLE,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/",
                "raindrops.wgsl"
            ),
            Shader::from_wgsl
        );

        let mut assets = app.world_mut().resource_mut::<Assets<_>>();

        let image = Image::from_buffer(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/textures/",
                "raindrops.tga"
            )),
            ImageType::Extension("tga"),
            CompressedImageFormats::NONE,
            false,
            ImageSampler::Default,
            RenderAssetUsages::default(),
        )
        .expect("Should load raindrops successfully");
        assets
            .insert(RAINDROPS_IMAGE_HANDLE.id(), image)
            .expect("UUID handles are valid");

        // This puts the uniform into the render world.
        app.add_plugins((
            ExtractComponentPlugin::<Raindrops>::default(),
            UniformComponentPlugin::<Raindrops>::default(),
            ExtractResourcePlugin::<RaindropsTextureHandle>::default(),
        ))
        .insert_resource(RaindropsTextureHandle(RAINDROPS_IMAGE_HANDLE.clone()));

        super::render_app(app)
            .init_gpu_resource::<RaindropsData>()
            .init_resource::<UniformBindGroup<Raindrops>>()
            .add_systems(Render, queue.in_set(RenderSystems::PrepareBindGroups));
    }
}

fn queue(
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    data: Res<RaindropsData>,
    texture_handle: Res<RaindropsTextureHandle>,
    mut bind_group: ResMut<UniformBindGroup<Raindrops>>,
    uniforms: Res<ComponentUniforms<Raindrops>>,
    images: Res<RenderAssets<GpuImage>>,
    views: Query<Entity, With<Raindrops>>,
) {
    bind_group.inner = None;

    if let (Some(uniforms), Some(raindrops_image)) =
        (uniforms.uniforms().binding(), images.get(&**texture_handle))
    {
        if !views.is_empty() {
            bind_group.inner = Some(render_device.create_bind_group(
                "Raindrops Uniform Bind Group",
                &pipeline_cache.get_bind_group_layout(&data.layout),
                &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&raindrops_image.texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&data.sampler),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: uniforms.clone(),
                    },
                ],
            ));
        }
    }
}

/// Raindrops settings.
#[derive(Debug, Component, Clone, Copy, ShaderType)]
pub struct Raindrops {
    /// How quickly the raindrops animate.
    pub speed: f32,

    /// How much the raindrops warp the image.
    pub warping: f32,

    /// How zoomed in the raindrops texture is.
    pub zoom: f32,
}

impl Default for Raindrops {
    fn default() -> Self {
        Self {
            speed: 0.8,
            warping: 0.03,
            zoom: 1.0,
        }
    }
}

impl Display for Raindrops {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Raindrops speed: {}, warping: {}, zoom: {}",
            self.speed, self.warping, self.zoom
        )
    }
}

impl ExtractComponent for Raindrops {
    type QueryData = (&'static Self, Option<&'static Order<Self>>);
    type QueryFilter = ();
    type Out = (Self, Order<Self>);

    fn extract_component(
        (settings, order): QueryItem<'_, '_, Self::QueryData>,
    ) -> Option<Self::Out> {
        Some((*settings, order.copied().unwrap_or_else(|| Order::new(0.0))))
    }
}

impl SyncComponent for Raindrops {
    type Target = (Self, Order<Self>);
}
