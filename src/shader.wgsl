struct CameraUniform {
    view_proj: mat4x4<f32>
}

struct ModelUniform {
    model: mat4x4<f32>
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> model: ModelUniform;

@group(0) @binding(2)
var<uniform> is_instanced: i32;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
};

@vertex
fn vs_main(in: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    let m = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    if is_instanced == 1 {
        out.clip_position = camera.view_proj * m * model.model * vec4<f32>(in.position, 1.0);
    } else if is_instanced == 0 {
        out.clip_position = camera.view_proj * model.model * vec4<f32>(in.position, 1.0);
    }

    out.tex_coords = in.tex_coords;
    return out;
}

@group(0) @binding(3)
var tex_diffuse: texture_2d<f32>;
@group(0) @binding(4)
var tex_sampler: sampler; 

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex_diffuse, tex_sampler, in.tex_coords);
}