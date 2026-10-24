fn hex_coords(p: vec2<f32>) -> vec2<f32> {
    let s = vec2(1.0, 1.7320508);
    let h = s * 0.5;
    let a = ( p % s ) - h;
    let b = ((p - h) % s) - h;
    return select(a, b, dot(a, a) < dot(b, b));
}

pub fn hex_grid(p: vec2<f32>, scale: f32) -> f32 {
    let p_scaled = p * scale;
    let gv = hex_coords(p_scaled);
    let edge = max(abs(gv.x), abs(gv.y) * 0.866025 + abs(gv.x) * 0.5);
    return smoothstep(0.45, 0.5, edge);
}
