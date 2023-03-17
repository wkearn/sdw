struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32,
	   @builtin(instance_index) instance: u32) -> VertexOutput {
  var out: VertexOutput;

  var vertex = vec2(0.0, 1.0);
  var tex_coords = vec2(0.0,6.0);
  switch idx {
      case 1u: {
	vertex = vec2(0.0, -1.0);
	tex_coords = vec2(0.0,2.0);
      }
      case 2u, 4u: {
	vertex = vec2(1.0, -1.0);
	tex_coords = vec2(1.0,2.0);
      }
      case 5u: {
	vertex = vec2(1.0, 1.0);
	tex_coords = vec2(1.0,6.0);
      }
	default: {}
    }
  
  if instance == 0u {
      // Starboard instance
      out.tex_coords = tex_coords;
      out.clip_position = vec4<f32>(vertex,0.0,1.0);
    }
  else {
    // Port instance
    out.tex_coords = vec2<f32>(1.0 - tex_coords.x,tex_coords.y);
    out.clip_position = vec4<f32>(vertex - vec2<f32>(1.0,0.0),0.0,1.0);
  }

  return out;
}

fn hsv_to_rgb(c: vec3<f32>) -> vec3<f32> {
  let K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0,0.0,0.0), vec3<f32>(1.0,1.0,1.0)), c.y);
}

@group(0) @binding(0)
var t_diffuse: texture_2d_array<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let visible_tile = i32(in.tex_coords.y / 1.0);
  let s = textureSample(t_diffuse,s_diffuse,vec2(in.tex_coords.x,in.tex_coords.y % 1.0),visible_tile);
  return vec4<f32>(hsv_to_rgb(vec3<f32>(0.109,0.9,clamp(s.x/10000.0,0.0,1.0))),1.0);
  
}
