struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) weight: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = vec2<f32>((model.position.x + 1) / 2 , ((1 - model.position.y)) / 2);
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@group(0) @binding(0)
var colormap_tex: texture_1d<f32>;
@group(0) @binding(1)
var colormap_samp: sampler;

@group(1) @binding(0)
var blended_tex: texture_2d<f32>;
@group(1) @binding(1)
var blended_samp: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let weight = textureSample(blended_tex, blended_samp, in.tex_coords).x;

    if weight == 0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    let tex_dim = textureDimensions(colormap_tex);
    let map_coord = clamp(weight, 0.0, f32(tex_dim));

    let color = textureLoad(colormap_tex, u32(map_coord * 10), 0);

    return vec4<f32>(color);
}
