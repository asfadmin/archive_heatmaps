struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput
    ) -> VertexOutput {
    var out: VertexOutput;

    out.tex_coords = vec2<f32>((model.position.x + 1) / 2 , ((1 - model.position.y)) / 2);
    out.clip_position = vec4<f32>(model.position, 1.0);

    return out;
}


@group(0) @binding(0)
var blended_tex: texture_2d<f32>;
@group(0) @binding(1)
var blended_samp: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0)vec4<f32> {
    
    var color = textureSample(blended_tex, blended_samp, in.tex_coords);
    
    // Replace any black pixels with white
    if (dot(color, vec4<f32>(1.0, 1.0, 1.0, 0.0)) < 0.1) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }

    return color;
}