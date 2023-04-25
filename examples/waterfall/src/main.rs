use sdw::{
    model::{Ping, SonarDataRecord},
    parser::jsf,
};

use waterfall::run;

use itertools::Itertools;

use vello::util::RenderContext;

use winit::event_loop::EventLoop;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// The path to a JSF file to display
    path: std::path::PathBuf,
}

impl Args {
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let args = Args::parse();

        log::debug!("Opening {:?}", args.path);

        let jsf = jsf::File::open(args.path)?;

        let (port_data, starboard_data): (Vec<_>, Vec<_>) = jsf
            .filter_map(|msg| match SonarDataRecord::from(msg.unwrap()) {
                SonarDataRecord::Ping(Ping { data, .. }) => Some(data),
                _ => None,
            })
            .tuples()
            .unzip();

        let data_len = port_data[0].len();
        let padding = vec![0.0f32; 256 - (data_len % 256)];

        let row_max = port_data.len();

        let padded_len = std::cmp::min(8192, data_len + padding.len());

        let port_data: Vec<f32> = port_data
            .into_iter()
            .flat_map(|x| x.into_iter().chain(padding.clone()).take(8192))
            .collect();
        let starboard_data: Vec<f32> = starboard_data
            .into_iter()
            .flat_map(|x| x.into_iter().chain(padding.clone()).take(8192))
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    Args::run()?;

    Ok(())
}
