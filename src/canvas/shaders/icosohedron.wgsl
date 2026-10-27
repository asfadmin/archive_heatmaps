// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var colormap_tex: texture_1d<f32>;

@group(2) @binding(0)
var blended_tex: texture_2d<f32>;
@group(2) @binding(1)
var blended_samp: sampler;

@group(3)@binding(0)
var<uniform> max_weight: vec4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let PI = 3.141592653589793;
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.color = model.color;
    out.uv = vec2((atan2(model.position.z, model.position.x)/(2*PI)), (asin(model.position.y)/PI)+0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let weight = textureSample(blended_tex, blended_samp, in.uv).x;

    if weight == 0 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let tex_dim = textureDimensions(colormap_tex);

    // This equation originated entirley from messing around and finding what felt good
    let ratio = (weight * 1.32)/(max_weight.x) * f32(tex_dim);

    let map_coord = clamp(ratio, 0.0, f32(tex_dim - 1 ));

    var color = textureLoad(colormap_tex, u32(map_coord), 0);

    color.a = clamp(pow(f32(weight), 2.0)/(max_weight.x*4), 0.3, 1.0);

    return vec4<f32>(color);
}

