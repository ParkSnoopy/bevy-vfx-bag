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

struct Flip {
    x: f32,
    y: f32,
};
@group(1) @binding(0)
var<uniform> flip: Flip;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = abs(vec2<f32>(flip.x, flip.y) - in.uv);
    return textureSample(t, ts, uv);
}