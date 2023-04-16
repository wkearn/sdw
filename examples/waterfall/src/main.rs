use sdw::{
    model::{Ping, SonarDataRecord},
    parser::jsf,
};

use waterfall::run;

use itertools::Itertools;

use vello::util::RenderContext;

use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let jsf =
        jsf::File::open("/home/wkearn/Documents/projects/SDW/assets/HE501/HE501_Hydro1_007.jsf")?;

    let (port_data, starboard_data): (Vec<_>, Vec<_>) = jsf
        .filter_map(|msg| match SonarDataRecord::from(msg.unwrap()) {
            SonarDataRecord::Ping(Ping { data, .. }) => Some(data),
            _ => None,
        })
        .tuples::<(_, _)>()
        .unzip();

    let data_len = port_data[0].len();
    let padding = vec![0.0f32; 256 - (data_len % 256)];

    let row_max = port_data.len();

    let padded_len = data_len + padding.len();

    let port_data: Vec<f32> = port_data
        .into_iter()
        .flat_map(|x| x.into_iter().chain(padding.clone()))
        .collect();
    let starboard_data: Vec<f32> = starboard_data
        .into_iter()
        .flat_map(|x| x.into_iter().chain(padding.clone()))
        .collect();
   
    // The event loop comes from winit
    let event_loop = EventLoop::new();

    // The render context comes from vello::util::RenderContext
    // It basically wraps all the necessary wgpu rendering state
    let render_cx = RenderContext::new().unwrap();

    // This is our run function (see above)
    run(
        event_loop,
        render_cx,
        port_data,
        starboard_data,
        padded_len,
        row_max,
    );

    Ok(())
}
