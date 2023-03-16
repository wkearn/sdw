// Downsweep to compute the prefix scan

// Input data is the output from the reduction
@group(0)
@binding(0)
var<storage,read> in_data: array<f32>;

@group(0)
@binding(1)
var<storage,read_write> out_data: array<f32>;

@group(0)
@binding(2)
var<storage,read> block_incr: array<f32>;

var<workgroup> temp: array<f32,256>;

@compute
@workgroup_size(128,1,1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>,
	@builtin(local_invocation_index) tid: u32,
	@builtin(num_workgroups) num_workgroups: vec3<u32>,
	@builtin(workgroup_id) group_id: vec3<u32> ) {

  let block = group_id.x + num_workgroups.x * group_id.y;
  
  let gid = global_id.x + 128u * num_workgroups.x * global_id.y;
  
  let total_length: u32 = arrayLength(&in_data); // n * T

  // Load input data
  if gid < (total_length >> 1u) {
      temp[2u * tid] = in_data[2u * gid];
      temp[2u * tid + 1u] = in_data[2u * gid + 1u];
    }

  workgroupBarrier();

  // Replace top element with the block increment
  if tid == 0u {
      temp[255] = block_incr[block];
    }

  workgroupBarrier();

  var offset: u32 = 256u;
  for (var d: u32 = 1u; d < 256u; d *= 2u) {
    offset >>= 1u;
    if tid < d {
	var ai = offset * (2u * tid + 1u) - 1u;
	var bi = offset * (2u * tid + 2u) - 1u;
	let t = temp[ai];
	temp[ai] = temp[bi];
	temp[bi] += t;
      }
    workgroupBarrier();
  }

  // Store output data
  if gid < (total_length >> 1u) {
      out_data[2u * gid] = temp[2u * tid];
      out_data[2u * gid + 1u] = temp[2u * tid + 1u];
    }
}

