import bevy_pbr::render::mesh::VertexOutput;
import bevy_pbr::mesh_view_bindings::view;
import super::effect_utils::hex_grid;

struct PillarMaterial {
    // vec4 保证 16 字节对齐，time 放在 .x
    time: vec4<f32>,
    base_color: vec4<f32>,
};

@group(3) @binding(0) var<uniform> material: PillarMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let t = material.time.x;

    let view_dir = normalize(in.world_position.xyz - view.world_position.xyz);
    let normal = normalize(in.world_normal);
    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);

    let flow_uv = vec2(uv.x, uv.y * 2.0 - t * 1.5);
    let grid = hex_grid(flow_uv, 15.0);
    let scanline = sin(uv.y * 20.0 - t * 5.0) * 0.5 + 0.5;
    let pulse = sin(t * 3.0) * 0.2 + 0.8;

    var final_color = material.base_color.rgb * pulse;
    final_color += material.base_color.rgb * grid * 2.0;
    final_color += vec3(1.0, 0.8, 0.6) * fresnel * 3.0;
    final_color += vec3(0.5, 0.8, 1.0) * scanline * 0.3;

    let alpha = clamp(fresnel * 1.5 + grid * 0.5 + 0.1, 0.0, 1.0);
    return vec4(final_color * 4.0, alpha);
}