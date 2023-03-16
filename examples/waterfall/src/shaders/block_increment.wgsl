// Compute block increments by the Hillis-Steele algorithm.
// Since we have ~O(10) blocks per line, this might be more
// efficient and done in a single compute shader dispatch

@group(0)
@binding(0)
var<storage,read_write> block_sums: array<f32>;

var<workgroup> temp: array<f32,128>;

@compute
@workgroup_size(64,1,1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>,
	@builtin(local_invocation_index) tid: u32,
	@builtin(num_workgroups) num_groups: vec3<u32>,
	@builtin(workgroup_id) group_id: vec3<u32>) {

  // Is there a more robust way to obtain this?
  let n: u32 = arrayLength(&block_sums)/ num_groups.y; 
  let row = group_id.y;
  
  // Load data into shared memory
  var in: u32 = 1u;
  var out: u32 = 0u;

  if tid < n {
      temp[out * n + tid] = select(0.0,block_sums[row * n + tid - 1u],tid > 0u);
    }

  workgroupBarrier();

  for (var offset: u32 = 1u; offset < n; offset *= 2u) {
    // Swap buffers
    in = 1u - in;
    out = 1u - out;

    if tid < n {
	temp[out * n + tid] = temp[in * n + tid];
	if tid >= offset {
	    temp[out * n + tid] += temp[in * n + tid - offset];
	  }
      }
    workgroupBarrier();
  }

  // Write shared memory back to block_sums
  if tid < n {
      block_sums[row * n + tid] = temp[out * n + tid];
    }
}

