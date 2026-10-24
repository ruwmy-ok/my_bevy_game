import bevy_pbr::render::mesh::VertexOutput;
import super::effect_utils::hex_grid;

struct PillarMaterial {
    time: vec4<f32>,
    base_color: vec4<f32>,
};

@group(3) @binding(0) var<uniform> material: PillarMaterial;

// 伪随机：输入一个数，输出 0..1
fn hash(n: f32) -> f32 {
    return fract(sin(n) * 43758.5453);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let t = material.time.x;
    let normal = normalize(in.world_normal);
    let fresnel = 0.3;
    let flow_uv = vec2(uv.x, uv.y * 2.0 - t * 1.5);
    let grid = hex_grid(flow_uv, 30.0);
    let scanline = sin(uv.y * 20.0 - t * 5.0) * 0.5 + 0.5;

    // ---- 不定时闪烁 ----
    // 每 0.08 秒换一个随机种子；调大 12.5 会让跳变更快
    let seed = floor(t * 12.5);
    let r = hash(seed);

    // 随机值高于阈值才“亮”，否则暗。阈值 0.7 => 约 30% 时间亮
    // 亮时再叠加一个短促的衰减，让闪烁有“亮起-熄灭”的感觉
    let local = fract(t * 12.5);             // 0..1 在当前时间片内
    let flash = select(0.15, 1.0 - local * 0.8, r > 0.7);

    // 慢速副脉动保留，但幅度降低，避免盖过闪烁
    let slow_pulse = sin(t * 1.1) * 0.1 + 0.9;

    var final_color = material.base_color.rgb * flash * slow_pulse;
    final_color += material.base_color.rgb * grid * 2.0;
    final_color += vec3(1.0, 0.8, 0.6) * fresnel * 3.0 * slow_pulse;
    final_color += vec3(0.5, 0.8, 1.0) * scanline * 0.3 * slow_pulse;

    let alpha = clamp(fresnel * 1.5 + grid * 0.5 + 0.1, 0.0, 1.0);
    return vec4(final_color * 0.5, alpha);
}