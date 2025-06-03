#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    mesh_view_bindings,
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

@group(2) @binding(100)
var<uniform> progress: vec4f;
@group(2) @binding(101)
var<uniform> center: vec4f;
@group(2) @binding(102)
var<uniform> radius: vec4f;


const STEP_LENGTH: f32 = 4.0;
const LIGHT_STEP_LENGTH: f32 = 8.0;



fn density_at_point(point: vec3f) -> f32 {
    let progress = progress.x;
    let center = center.xyz;
    let radius = radius.x;

    let dist = distance(point, center) / radius;

    let noise_scale = 2. + progress * 8.;
    let noise = snoise(point / noise_scale) * 0.5 + 0.5;
    let progress_radius_mask = smoothstep(progress, progress - 0.1, dist);
    var density = noise * progress_radius_mask;

    let sparseness = smoothstep(0.3, 0.99, progress);
    density =  smoothstep(sparseness, sparseness + 0.1, density);

    return density;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    let progress = progress.x;
    let center = center.xyz;
    let radius = radius.x;
  
    let pbr_input = pbr_input_from_standard_material(in, is_front);
    let light_direction = mesh_view_bindings::lights.directional_lights[0].direction_to_light;
    
    let ray_length = radius * 2.;
    var position = pbr_input.world_position.xyz;
    let steps = u32(ray_length / STEP_LENGTH);
    let light_steps = u32(ray_length / LIGHT_STEP_LENGTH);
    // cumulative density and brightness
    var max_density: f32 = 0.0;
    var cum_brightness: f32 = 0.0;

    for (var i = 0u; i < steps; i++) {
        position -= pbr_input.V * STEP_LENGTH;
        let d = density_at_point(position);

        // march toward the light for current sample point
        var transmittance = 1.0;
        var light_pos = position;
        var j = 0u;
        for (; j < light_steps; j++) {
            let ld = density_at_point(light_pos);
            // transmittance *= exp(-ld * LIGHT_STEP_LENGTH);
            transmittance *= 1. / (2.0 * ld + 1.0);
            light_pos += light_direction * LIGHT_STEP_LENGTH;
        }

        // accumulate scattering: more density, more brightness, but modulated by how much light makes it in
        cum_brightness += d * transmittance;
        // cum_density += d;
        max_density = max(max_density, d);
        if (max_density > 0.7) {
            break;
        } 
    }
    
    var out: FragmentOutput;


    var color = vec3f(cum_brightness);
    if (cum_brightness > 0.33) {
        color = vec3f(
            0. / 255., 
            70. / 255.,
            247. / 255.
        );
    } else if (cum_brightness > 0.25) {
        color = vec3f(
            138. / 255., 
            165. / 255.,
            234. / 255.
        );
    } else {
        color = vec3f(
            15. / 255., 
            15. / 255.,
            15. / 255.
        );
    }

    let alpha = 1.0 - exp(-max_density);
    var alpha_mod = alpha;
    if (alpha_mod > 0.7) {
        alpha_mod = 0.9;
    } else if (alpha_mod > 0.5) {
        alpha_mod = 0.5;
    } else {
        alpha_mod = 0.0;
    }

    out.color = vec4f(color, alpha_mod);

    return out;
    

    // pbr_input.material.base_color.b = pbr_input.material.base_color.r;
    // pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
    // var out: FragmentOutput;
    // out.color = apply_pbr_lighting(pbr_input);
    // out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    // out.color = out.color * 2.0;
    // return out;
}




//
// Description : Array and textureless WGSL 2D/3D/4D simplex
//               noise functions.
//      Author : Ian McEwan, Ashima Arts.
//  Maintainer : ijm
//     Lastmod : 20110822 (ijm)
//     License : Copyright (C) 2011 Ashima Arts. All rights reserved.
//               Distributed under the MIT License. See LICENSE file.
//               https://github.com/ashima/webgl-noise
//     Translated to WGSL by Claude

fn mod289_vec3(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn mod289_vec4(x: vec4<f32>) -> vec4<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn permute(x: vec4<f32>) -> vec4<f32> {
    return mod289_vec4(((x * 34.0) + 1.0) * x);
}

fn taylorInvSqrt(r: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(1.79284291400159) - 0.85373472095314 * r;
}

fn snoise(v: vec3<f32>) -> f32 {
    let C = vec2<f32>(1.0 / 6.0, 1.0 / 3.0);
    let D = vec4<f32>(0.0, 0.5, 1.0, 2.0);
    
    // First corner
    var i = floor(v + dot(v, C.yyy));
    let x0 = v - i + dot(i, C.xxx);
    
    // Other corners
    let g = step(x0.yzx, x0.xyz);
    let l = vec3<f32>(1.0) - g;
    let i1 = min(g.xyz, l.zxy);
    let i2 = max(g.xyz, l.zxy);
    
    //   x0 = x0 - 0.0 + 0.0 * C.xxx;
    //   x1 = x0 - i1  + 1.0 * C.xxx;
    //   x2 = x0 - i2  + 2.0 * C.xxx;
    //   x3 = x0 - 1.0 + 3.0 * C.xxx;
    let x1 = x0 - i1 + C.xxx;
    let x2 = x0 - i2 + C.yyy; // 2.0*C.x = 1/3 = C.y
    let x3 = x0 - D.yyy;      // -1.0+3.0*C.x = -0.5 = -D.y
    
    // Permutations
    i = mod289_vec3(i);
    let p = permute(permute(permute(
                i.z + vec4<f32>(0.0, i1.z, i2.z, 1.0))
              + i.y + vec4<f32>(0.0, i1.y, i2.y, 1.0))
              + i.x + vec4<f32>(0.0, i1.x, i2.x, 1.0));
    
    // Gradients: 7x7 points over a square, mapped onto an octahedron.
    // The ring size 17*17 = 289 is close to a multiple of 49 (49*6 = 294)
    let n_ = 0.142857142857; // 1.0/7.0
    let ns = n_ * D.wyz - D.xzx;
    
    let j = p - 49.0 * floor(p * ns.z * ns.z);  //  mod(p,7*7)
    let x_ = floor(j * ns.z);
    let y_ = floor(j - 7.0 * x_);    // mod(j,N)
    
    let x = x_ * ns.x + ns.yyyy;
    let y = y_ * ns.x + ns.yyyy;
    let h = vec4<f32>(1.0) - abs(x) - abs(y);
    
    let b0 = vec4<f32>(x.xy, y.xy);
    let b1 = vec4<f32>(x.zw, y.zw);
    
    //let s0 = vec4<f32>(lessThan(b0,0.0))*2.0 - 1.0;
    //let s1 = vec4<f32>(lessThan(b1,0.0))*2.0 - 1.0;
    let s0 = floor(b0) * 2.0 + vec4<f32>(1.0);
    let s1 = floor(b1) * 2.0 + vec4<f32>(1.0);
    let sh = -step(h, vec4<f32>(0.0));
    
    let a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    let a1 = b1.xzyw + s1.xzyw * sh.zzww;
    
    let p0 = vec3<f32>(a0.xy, h.x);
    let p1 = vec3<f32>(a0.zw, h.y);
    let p2 = vec3<f32>(a1.xy, h.z);
    let p3 = vec3<f32>(a1.zw, h.w);
    
    // Normalise gradients
    let norm = taylorInvSqrt(vec4<f32>(dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)));
    let p0_norm = p0 * norm.x;
    let p1_norm = p1 * norm.y;
    let p2_norm = p2 * norm.z;
    let p3_norm = p3 * norm.w;
    
    // Mix final noise value
    var m = max(vec4<f32>(0.6) - vec4<f32>(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3)), vec4<f32>(0.0));
    m = m * m;
    return 42.0 * dot(m * m, vec4<f32>(dot(p0_norm, x0), dot(p1_norm, x1),
                                       dot(p2_norm, x2), dot(p3_norm, x3)));
}
