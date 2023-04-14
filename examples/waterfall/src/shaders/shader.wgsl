struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) tex_coords: vec2<f32>,
};

struct Viewport {
 viewport: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> viewport: Viewport;  
  
@vertex
fn vs_main(@builtin(vertex_index) idx: u32,
	   @builtin(instance_index) instance: u32) -> VertexOutput {
  var out: VertexOutput;

  let a = viewport.viewport.y;
  
  var vertex = vec2(0.0, 1.0);
  var tex_coords = vec2(0.0,4.0 + a);
  switch idx {
      case 1u: {
	vertex = vec2(0.0, -1.0);
	tex_coords = vec2(0.0,0.0 + a);
      }
      case 2u, 4u: {
	vertex = vec2(1.0, -1.0);
	tex_coords = vec2(1.0,0.0 + a);
      }
      case 5u: {
	vertex = vec2(1.0, 1.0);
	tex_coords = vec2(1.0,4.0 + a);
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

fn lab_to_xyz(c: vec3<f32>) -> vec3<f32> {
  let xyz = vec3<f32>((c.x + 16.0) / 116.0 + c.y / 500.0, (c.x + 16.0) / 116.0, (c.x + 16.0) / 116.0 - c.z / 200.0);
  let delta = vec3<f32>(6.0 / 29.0);
  let d65 = vec3<f32>(0.950489, 1.0, 1.08840);
  return d65 * select(pow(xyz,vec3<f32>(3.0)), 3.0 * pow(delta,vec3<f32>(2.0)) * (xyz - 4.0 / 29.0),xyz <= delta);
}

fn rgb_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let thresh = vec3<f32>(0.0031308);
  return select(1.055*pow(rgb,vec3<f32>(1.0 / 2.4)) - 0.055,12.92 * rgb, rgb <= thresh);
}

fn xyz_to_rgb(c: vec3<f32>) -> vec3<f32> {
  let conversion =  mat3x3<f32>(3.2406, -0.9689,0.0557, -1.5372, 1.8758, -0.2040, -0.4986, 0.0415, 1.0570);
  return conversion * c;
}

fn lab_to_srgb(c: vec3<f32>) -> vec3<f32> {  
  return rgb_to_srgb(xyz_to_rgb(lab_to_xyz(c)));
}

@group(0) @binding(0)
var t_diffuse: texture_2d_array<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let tile_index = i32(floor(in.tex_coords.y)); // Index of the tile in the complete data set.
  let visible_tile = (tile_index + 2) % 8; // Index of the tile in the buffer
  let s = textureSample(t_diffuse,s_diffuse,vec2(in.tex_coords.x,fract(in.tex_coords.y)),visible_tile);
  let v = sqrt(clamp(s.x / 10000.0, 0.0,1.0));
  return vec4<f32>(lab_to_srgb(vec3<f32>(100.0*v,18.0*v,77.0*v)),1.0);
  
}
