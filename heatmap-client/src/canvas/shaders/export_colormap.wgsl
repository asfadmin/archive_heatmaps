// This is shader is identical to colormap.wgsl except
// for the removal of the transperancy adjustments

struct VertexInput {
    @location(0) position: vec3<f32>,
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

@group(1) @binding(0)
var blended_tex: texture_2d<f32>;
@group(1) @binding(1)
var blended_samp: sampler;

@group(2)@binding(0)
var<uniform> max_weight: vec4<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let weight = textureSample(blended_tex, blended_samp, in.tex_coords).x;

    if weight == 0 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let tex_dim = textureDimensions(colormap_tex);
    
    // This equation originated entirley from messing around and finding what felt good
    let ratio = (weight * 1.1)/(max_weight.x) * f32(tex_dim);

    let map_coord = clamp(ratio * 1.2, 0.0, f32(tex_dim - 1 ));

    var color = textureLoad(colormap_tex, u32(map_coord), 0).rgb;

    return vec4<f32>(color, 1.0);
}
