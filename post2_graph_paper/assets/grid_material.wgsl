#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

@group(1) @binding(100) var<uniform> grid_color: vec4<f32>;
@group(1) @binding(101) var<uniform> grid_subdivisions: f32;
@group(1) @binding(102) var<uniform> grid_line_widths: vec2<f32>;

fn sample_grid(
    uv: vec2<f32>
) -> f32 {
    var multi_uv = uv * grid_subdivisions;
    let uv_ddxy = vec4<f32>(dpdx(multi_uv), dpdy(multi_uv));
    let uv_deriv = vec2<f32>(length(uv_ddxy.xz), length(uv_ddxy.yw));
    let invert_line = grid_line_widths > 0.5;
    let target_width = select(grid_line_widths, 1.0 - grid_line_widths, invert_line);
    let draw_width = clamp(target_width, uv_deriv, vec2<f32>(0.5, 0.5));
    let line_aa = uv_deriv * 1.5;
    var grid_uv = abs(fract(multi_uv) * 2.0 - 1.0);
    grid_uv = select(1.0 - grid_uv, grid_uv, invert_line);
    var grid2 = smoothstep(draw_width + line_aa, draw_width - line_aa, grid_uv);
    grid2 *= saturate(target_width / draw_width);
    grid2 = mix(grid2, target_width, saturate(uv_deriv * 2.0 - 1.0));
    grid2 = select(grid2, 1.0 - grid2, invert_line);
    let grid = mix(grid2.x, 1.0, grid2.y);

    return grid;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    let grid_mix = sample_grid(in.uv);
    if (grid_mix > 0.0) {
        // if the grid value is greater than 1, it means we should
        // overwrite the current fragment color with the grid color
        pbr_input.material.base_color = vec4<f32>(grid_color.xyz, grid_mix);
    }

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;

    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}