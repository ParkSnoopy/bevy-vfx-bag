use bevy::{
    asset::{RenderAssetUsages, load_internal_asset, uuid_handle},
    ecs::query::QueryItem,
    image::{CompressedImageFormats, ImageSampler, ImageType},
    prelude::*,
    render::{
        GpuResourceAppExt, Render, RenderSystems,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
            BindingResource, BindingType, CachedRenderPipelineId, Extent3d, PipelineCache,
            SamplerBindingType, ShaderStages, TextureDimension, TextureFormat, TextureSampleType,
            TextureViewDescriptor, TextureViewDimension,
        },
        renderer::RenderDevice,
        sync_component::SyncComponent,
        texture::GpuImage,
    },
    shader::Shader,
};

use super::Order;

pub(crate) const LUT_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("00000000-0000-0000-339f-a6bd5789a33c");

const LUT_ARCTIC_IMAGE_HANDLE: Handle<Image> = uuid_handle!("00000000-0000-0000-9fcc-ae05d02e3008");
const LUT_NEO_IMAGE_HANDLE: Handle<Image> = uuid_handle!("00000000-0000-0000-ff84-27feadea8403");
const LUT_SLATE_IMAGE_HANDLE: Handle<Image> = uuid_handle!("00000000-0000-0000-7a42-4bfde669d6fd");

#[derive(Debug, Component)]
pub(crate) struct LutBindGroup {
    pub(crate) bind_group: BindGroup,
}

#[derive(Resource)]
pub(crate) struct LutData {
    pub pipeline_id: CachedRenderPipelineId,
    pub layout: BindGroupLayoutDescriptor,
}

impl FromWorld for LutData {
    fn from_world(world: &mut World) -> Self {
        let (layout, pipeline_id) = super::create_layout_and_pipeline(
            world,
            "LUT",
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D3,
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
            ],
            LUT_SHADER_HANDLE.clone(),
        );

        LutData {
            pipeline_id,
            layout,
        }
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            LUT_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/", "lut.wgsl"),
            Shader::from_wgsl
        );

        let mut assets = app.world_mut().resource_mut::<Assets<_>>();

        let image = Image::from_buffer(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/luts/",
                "neo.png"
            )),
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            false,
            ImageSampler::Default,
            RenderAssetUsages::default(),
        )
        .expect("Should load LUT successfully");
        assets
            .insert(LUT_NEO_IMAGE_HANDLE.id(), image)
            .expect("UUID handles are valid");

        let image = Image::from_buffer(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/luts/",
                "slate.png"
            )),
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            false,
            ImageSampler::Default,
            RenderAssetUsages::default(),
        )
        .expect("Should load LUT successfully");
        assets
            .insert(LUT_SLATE_IMAGE_HANDLE.id(), image)
            .expect("UUID handles are valid");

        let image = Image::from_buffer(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/luts/",
                "arctic.png"
            )),
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            false,
            ImageSampler::Default,
            RenderAssetUsages::default(),
        )
        .expect("Should load LUT successfully");
        assets
            .insert(LUT_ARCTIC_IMAGE_HANDLE.id(), image)
            .expect("UUID handles are valid");

        // This puts the uniform into the render world.
        app.add_plugins(ExtractComponentPlugin::<Lut>::default())
            .add_systems(PostUpdate, adapt_image_for_lut_use);

        super::render_app(app)
            .init_gpu_resource::<LutData>()
            .add_systems(Render, queue.in_set(RenderSystems::PrepareBindGroups));
    }
}

fn adapt_image_for_lut_use(
    mut assets: ResMut<Assets<Image>>,
    mut luts: Query<&mut Lut, Changed<Lut>>,
) {
    for mut lut in luts.iter_mut() {
        if lut.prepared {
            continue;
        }

        let mut image = assets
            .get_mut(&lut.texture)
            .expect("Handle should point to asset");

        // The LUT is a 3d texture. It has 64 layers, each of which is a 64x64 image.
        image.texture_descriptor.size = Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 64,
        };
        image.texture_descriptor.dimension = TextureDimension::D3;
        image.texture_descriptor.format = TextureFormat::Rgba8Unorm;

        image.texture_view_descriptor = Some(TextureViewDescriptor {
            label: Some("LUT Texture View"),
            format: Some(TextureFormat::Rgba8Unorm),
            dimension: Some(TextureViewDimension::D3),
            ..default()
        });

        lut.prepared = true;
    }
}

fn queue(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    data: Res<LutData>,
    images: Res<RenderAssets<GpuImage>>,
    luts: Query<(Entity, &Lut)>,
) {
    for (entity, lut) in luts.iter() {
        if let Some(lut_image) = images.get(&lut.texture) {
            let bind_group = render_device.create_bind_group(
                "LUT Uniform Bind Group",
                &pipeline_cache.get_bind_group_layout(&data.layout),
                &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&lut_image.texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&lut_image.sampler),
                    },
                ],
            );

            commands.entity(entity).insert(LutBindGroup { bind_group });
        }
    }
}

/// A look-up texture. Maps colors to colors. Useful for colorschemes.
#[derive(Debug, Component, Clone)]
pub struct Lut {
    /// The 3D look-up texture
    texture: Handle<Image>,

    prepared: bool,
}

impl Lut {
    /// Creates a new LUT component.
    /// The image should be a 64x64x64 3D texture.
    /// See the `make-neutral-lut` example.
    pub fn new(texture: Handle<Image>) -> Self {
        Self {
            texture,
            prepared: false,
        }
    }

    /// The arctic color scheme LUT.
    pub fn arctic() -> Self {
        Self::new(LUT_ARCTIC_IMAGE_HANDLE.clone())
    }

    /// The neo color scheme LUT.
    pub fn neo() -> Self {
        Self::default()
    }

    /// The slate color scheme LUT.
    pub fn slate() -> Self {
        Self::new(LUT_SLATE_IMAGE_HANDLE.clone())
    }
}

impl Default for Lut {
    fn default() -> Self {
        Self::new(LUT_NEO_IMAGE_HANDLE.clone())
    }
}

impl ExtractComponent for Lut {
    type QueryData = (&'static Self, Option<&'static Order<Self>>);
    type QueryFilter = ();
    type Out = (Self, Order<Self>);

    fn extract_component((lut, order): QueryItem<'_, '_, Self::QueryData>) -> Option<Self::Out> {
        if !lut.prepared {
            return None;
        }

        Some((
            lut.clone(),
            order.copied().unwrap_or_else(|| Order::new(0.0)),
        ))
    }
}

impl SyncComponent for Lut {
    type Target = (Self, Order<Self>);
}
