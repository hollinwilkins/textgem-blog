#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    mesh_view_bindings::view,
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
    pbr_bindings,
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

@group(1) @binding(200) var<uniform> uv_scale: vec2<f32>;
@group(1) @binding(201) var<uniform> uv_translate: vec2<f32>;
@group(1) @binding(202) var<uniform> outline_thickness: f32;
@group(1) @binding(203) var<uniform> outline_color: vec4<f32>;

fn outline_alpha(uv: vec2<f32>) -> f32 {
    let outline_thickness_x = outline_thickness * uv_scale.x;
    let outline_thickness_y = outline_thickness * uv_scale.y;

    var alpha = 0.0;
    // fixed upAlpha = SampleSpriteTexture ( IN.texcoord + fixed2(0, _OutlineThickness)).a;
    alpha += textureSampleBias(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv + vec2<f32>(0.0, outline_thickness_y), view.mip_bias).a;

    // fixed downAlpha = SampleSpriteTexture ( IN.texcoord - fixed2(0, _OutlineThickness)).a;
    alpha += textureSampleBias(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv - vec2<f32>(0.0, outline_thickness_y), view.mip_bias).a;

    // fixed rightAlpha = SampleSpriteTexture ( IN.texcoord + fixed2(_OutlineThickness, 0)).a;
    alpha += textureSampleBias(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv + vec2<f32>(outline_thickness_x, 0.0), view.mip_bias).a;

    // fixed leftAlpha = SampleSpriteTexture ( IN.texcoord - fixed2(_OutlineThickness, 0)).a;
    alpha += textureSampleBias(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv - vec2<f32>(outline_thickness_x, 0.0), view.mip_bias).a;
    
    // fixed upRightAlpha = SampleSpriteTexture ( IN.texcoord - fixed2(_OutlineThickness, _OutlineThickness)).a;
    alpha += textureSampleBias(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv - vec2<f32>(outline_thickness_x, outline_thickness_y), view.mip_bias).a;

    // fixed upLeftAlpha = SampleSpriteTexture ( IN.texcoord - fixed2(_OutlineThickness, -_OutlineThickness)).a;
    alpha += textureSampleBias(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv - vec2<f32>(outline_thickness_x, -outline_thickness_y), view.mip_bias).a;

    // fixed downRightAlpha = SampleSpriteTexture ( IN.texcoord - fixed2(-_OutlineThickness, _OutlineThickness)).a;
    alpha += textureSampleBias(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv - vec2<f32>(-outline_thickness_x, outline_thickness_y), view.mip_bias).a;

    // fixed downLeftAlpha = SampleSpriteTexture ( IN.texcoord - fixed2(-_OutlineThickness, -_OutlineThickness)).a;
    alpha += textureSampleBias(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv - vec2<f32>(-outline_thickness_x, -outline_thickness_y), view.mip_bias).a;

    return saturate(alpha);
}

fn outline(uv: vec2<f32>) -> f32 {
    let c = textureSampleBias(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv, view.mip_bias);

    if (c.a == 0.0) {
        return outline_alpha(uv);
    } else {
        return 0.0;
    }
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    
    // translate and scale UVs
    let uv = (in.uv * uv_scale) + uv_translate;
    pbr_input.material.base_color = textureSampleBias(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv, view.mip_bias);

    // apply outline
    let outline_alpha = outline(uv);
    if (outline_alpha > 0.0) {
        pbr_input.material.base_color = outline_color;
    }

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    // write the gbuffer, lighting pass id, and optionally normal and motion_vector textures
    let out = deferred_output(in, pbr_input);
#else
    // in forward mode, we calculate the lit color immediately, and then apply some post-lighting effects here.
    // in deferred mode the lit color and these effects will be calculated in the deferred lighting shader
    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}