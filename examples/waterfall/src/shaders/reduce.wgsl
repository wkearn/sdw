// Reduce over sonar data according to Blelloch's algorithm

@group(0)
@binding(0)
var<storage,read> in_data: array<f32>;

@group(0)
@binding(1)
var<storage,read_write> out_data: array<f32>;

@group(0)
@binding(2)
var<storage,read_write> block_sums: array<f32>;

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

  // Reduce the temporary array

  var offset: u32 = 1u;

  // There are always 256 elements in the temp array,
  // so we start the loop at 256 >> 1 = 128
  for (var d: u32 = 128u; d > 0u; d >>= 1u) {
    if tid < d {
	let ai = offset * (2u * tid + 2u) - 1u;
	let bi = offset * (2u * tid + 1u) - 1u;
	temp[ai] += temp[bi];
      }
    offset <<= 1u;
    workgroupBarrier();
  }

  // Store the block sums
  if tid == 0u {
      block_sums[block] = temp[255];
    }

  // Store the output
  if gid < (total_length >> 1u) {
      out_data[2u * gid] = temp[2u * tid];
      out_data[2u * gid + 1u] = temp[2u * tid + 1u];
    }
}
