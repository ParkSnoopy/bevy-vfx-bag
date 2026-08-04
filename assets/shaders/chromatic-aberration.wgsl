struct FullscreenVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Globals {
    time: f32,
    delta_time: f32,
    frame_count: u32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    _webgl2_padding: f32
#endif
};

@group(0) @binding(0)
var t: texture_2d<f32>;
@group(0) @binding(1)
var ts: sampler;
@group(0) @binding(2)
var<uniform> globals: Globals;

struct ChromaticAberration {
    dir_r: vec2<f32>,
    magnitude_r: f32,

    dir_g: vec2<f32>,
    magnitude_g: f32,

    dir_b: vec2<f32>,
    magnitude_b: f32,
};

@group(1) @binding(0)
var<uniform> ca: ChromaticAberration;


@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let out = vec3<f32>(
        textureSample(t, ts, in.uv + (ca.dir_r * ca.magnitude_r)).r,
        textureSample(t, ts, in.uv + (ca.dir_g * ca.magnitude_g)).g,
        textureSample(t, ts, in.uv + (ca.dir_b * ca.magnitude_b)).b,
    );

    return vec4<f32>(out, 1.0);
}