struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
};

@vertex fn vs(
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    let pos = array(
        vec2f(0.0, 0.5),    // top center
        vec2f(-0.5, -0.5),  // bottom left
        vec2f(0.5, -0.5),   // bottom right
    );

    let color = array(
        vec3f(1.0, 0.0, 0.0),   // Red
        vec3f(0.0, 1.0, 0.0),   // Green
        vec3f(0.0, 0.0, 1.0),   // Blue
    );

    var out: VertexOutput;
    out.position = vec4f(pos[vertex_index], 0.0, 1.0);
    out.color = vec4f(color[vertex_index], 1.0);
    return out;
}

struct FragementInput {
    @location(0) color: vec4f,
};

@fragment fn fs(input: FragementInput) -> @location(0) vec4f {
    return vec4f(input.color);
}