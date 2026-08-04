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
    shader::{Shader, ShaderDefVal},
};
use std::fmt::Display;

use super::{Order, UniformBindGroup};
pub(crate) const MASK_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("00000000-0000-0000-0eb3-bec4e7b6fe36");

#[derive(Resource)]
pub(crate) struct MaskData {
    pub uniform_layout: BindGroupLayoutDescriptor,
    pipelines: [CachedRenderPipelineId; 3],
}

impl FromWorld for MaskData {
    fn from_world(world: &mut World) -> Self {
        let uniform_layout = super::create_layout(
            world,
            "Mask",
            &[BindGroupLayoutEntry {
                binding: 0,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(MaskUniform::min_size()),
                },
                visibility: ShaderStages::FRAGMENT,
                count: None,
            }],
        );

        let pipelines = [
            (MaskVariant::Square, "SQUARE"),
            (MaskVariant::Crt, "CRT"),
            (MaskVariant::Vignette, "VIGNETTE"),
        ]
        .map(|(variant, shader_def)| {
            let _ = variant;
            super::create_pipeline(
                world,
                "Masks",
                &uniform_layout,
                MASK_SHADER_HANDLE.clone(),
                vec![ShaderDefVal::Bool(shader_def.into(), true)],
            )
        });
        MaskData {
            uniform_layout,
            pipelines,
        }
    }
}

impl MaskData {
    pub(crate) fn pipeline_id(&self, variant: MaskVariant) -> CachedRenderPipelineId {
        self.pipelines[match variant {
            MaskVariant::Square => 0,
            MaskVariant::Crt => 1,
            MaskVariant::Vignette => 2,
        }]
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            MASK_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/", "masks.wgsl"),
            Shader::from_wgsl
        );

        // This puts the uniform into the render world.
        app.add_plugins((
            ExtractComponentPlugin::<Mask>::default(),
            UniformComponentPlugin::<MaskUniform>::default(),
        ));

        super::render_app(app)
            .init_gpu_resource::<MaskData>()
            .init_resource::<UniformBindGroup<MaskUniform>>()
            .add_systems(Render, queue.in_set(RenderSystems::PrepareBindGroups));
    }
}

fn queue(
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    data: Res<MaskData>,
    mut bind_group: ResMut<UniformBindGroup<MaskUniform>>,
    uniforms: Res<ComponentUniforms<MaskUniform>>,
    views: Query<Entity, With<MaskUniform>>,
) {
    bind_group.inner = None;

    if let Some(uniforms) = uniforms.uniforms().binding() {
        if !views.is_empty() {
            bind_group.inner = Some(render_device.create_bind_group(
                "Mask Uniform Bind Group",
                &pipeline_cache.get_bind_group_layout(&data.uniform_layout),
                &[BindGroupEntry {
                    binding: 0,
                    resource: uniforms.clone(),
                }],
            ));
        }
    }
}

/// This controls the parameters of the effect.
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Component)]
pub enum MaskVariant {
    /// Rounded square type mask.
    ///
    /// One use of this mask is to post-process _other_ effects which might
    /// have artifacts around the edges.
    /// This mask can then attenuate that effect and thus remove the effects of the
    /// artifacts.
    ///
    /// Strength value guidelines for use in [`Mask`]:
    ///
    /// Low end:    3.0 almost loses the square shape.
    /// High end:   100.0 has almost sharp, thin edges.
    Square,

    /// Rounded square type mask, but more oval like a CRT television.
    ///
    /// This effect can be used as a part of a retry-style effect.
    ///
    /// Strength value guidelines for use in [`Mask`]:
    ///
    /// Low end:    3000.0 almost loses the CRT shape.
    /// High end:   500000.0 "inflates" the effect a bit.
    Crt,

    /// Vignette mask.
    ///
    /// This effect can be used to replicate the classic photography
    /// light attenuation seen at the edges of photos.
    ///
    /// Strength value guidelines for use in [`Mask`]:
    ///
    /// Low end:    0.10 gives a very subtle effect.
    /// High end:   1.50 is almost a spotlight in the middle of the screen.
    Vignette,
}

/// A darkening mask on the outer edges of the image.
#[derive(Debug, Copy, Clone, Component)]
pub struct Mask {
    /// The strength parameter of the mask in use.
    ///
    /// See [`MaskVariant`] for guidelines on which range of values make sense
    /// for the variant in use.
    ///
    /// Run the masks example to experiment with these values interactively.
    pub strength: f32,

    /// How much the mask is faded: 1.0 - mask has no effect, 0.0 - mask is in full effect
    pub fade: f32,

    /// Which [`MaskVariant`] to produce.
    pub variant: MaskVariant,
}

impl Display for Mask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Mask {:?}, strength: {} fade: {}",
            self.variant, self.strength, self.fade
        )
    }
}

impl Mask {
    /// Create a new square mask with a reasonable strength value.
    pub fn square() -> Self {
        Self {
            strength: 20.,
            fade: 0.,
            variant: MaskVariant::Square,
        }
    }

    /// Create a new CRT mask with a reasonable strength value.
    pub fn crt() -> Self {
        Self {
            strength: 80000.,
            fade: 0.,
            variant: MaskVariant::Crt,
        }
    }

    /// Create a new vignette mask with a reasonable strength value.
    pub fn vignette() -> Self {
        Self {
            strength: 0.66,
            fade: 0.,
            variant: MaskVariant::Vignette,
        }
    }
}

impl Default for Mask {
    fn default() -> Self {
        Self::vignette()
    }
}

#[doc(hidden)]
/// [`Mask`] as a uniform.
#[derive(Debug, ShaderType, Clone, Component, Copy)]
pub struct MaskUniform {
    pub(crate) strength: f32,
    pub(crate) fade: f32,
}

impl From<Mask> for MaskUniform {
    fn from(mask: Mask) -> Self {
        Self {
            strength: mask.strength,
            fade: mask.fade,
        }
    }
}

impl ExtractComponent for Mask {
    type QueryData = (&'static Self, Option<&'static Order<Self>>);
    type QueryFilter = ();
    type Out = (MaskUniform, MaskVariant, Order<Self>);

    fn extract_component(
        (settings, order): QueryItem<'_, '_, Self::QueryData>,
    ) -> Option<Self::Out> {
        Some((
            (*settings).into(),
            settings.variant,
            order.copied().unwrap_or_else(|| Order::new(0.0)),
        ))
    }
}

impl SyncComponent for Mask {
    type Target = (MaskUniform, MaskVariant, Order<Self>);
}
