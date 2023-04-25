// This shader is copied from vello's BlitPipeline in vello/src/lib.rs
// It is licensed under the MIT/Apache 2.0 license as given below
//
// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Also licensed under MIT license, at your choice.

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4<f32> {  
  var vertex = vec2(-1.0, 1.0);
  switch idx {
      case 1u: {
	vertex = vec2(-1.0, -1.0);
      }
      case 2u, 4u: {
	vertex = vec2(1.0, -1.0);
      }
      case 5u: {
	vertex = vec2(1.0, 1.0);
      }
	default: {}
    }
  return vec4(vertex,0.0,1.0);
}

@group(0) @binding(0)
var fine_output: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let rgba_sep = textureLoad(fine_output, vec2<i32>(pos.xy), 0);
  return vec4(rgba_sep.rgb * rgba_sep.a, rgba_sep.a);
}
