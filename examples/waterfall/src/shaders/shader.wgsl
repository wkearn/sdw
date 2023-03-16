struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) tex_coords: vec2<f32>
};

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(model: VertexInput, @builtin(instance_index) instance: u32) -> VertexOutput {
  var out: VertexOutput;
  
  if instance == 0u {
      // Starboard instance
      out.tex_coords = model.tex_coords;
      out.clip_position = vec4<f32>(model.position,1.0);
    }
  else {
    // Port instance
    out.tex_coords = vec2<f32>(1.0 - model.tex_coords.x,model.tex_coords.y);
    out.clip_position = vec4<f32>(model.position - vec3<f32>(1.0,0.0,0.0),1.0);
  }

  return out;
}

fn hsv_to_rgb(c: vec3<f32>) -> vec3<f32> {
  let K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0,0.0,0.0), vec3<f32>(1.0,1.0,1.0)), c.y);
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let s = textureSample(t_diffuse,s_diffuse,in.tex_coords);
  return vec4<f32>(hsv_to_rgb(vec3<f32>(0.109,0.9,clamp(s.x/10000.0,0.0,1.0))),1.0);
}
