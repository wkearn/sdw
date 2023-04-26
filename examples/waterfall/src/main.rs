use sdw::{
    model::{Channel, Ping, SonarDataRecord},
    parser::jsf,
};

use waterfall::run;

use itertools::Itertools;
use std::collections::{BTreeSet,BTreeMap};

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

	// How can we make sure that port_data corresponds to starboard_data?
	// What happens if we have multiple subsystems?

	// This vector stores every ping
	let data: Vec<Ping<f32>> = jsf.filter_map(|msg| match SonarDataRecord::from(msg.unwrap()) {
	    SonarDataRecord::Ping(ping) => Some(ping),
	    _ => None,
	}).collect();

	// HashMap<Channel,Vec<Ping>>
	let mut data = data.into_iter().into_group_map_by(|ping| ping.channel);

	let port_pings: Vec<Ping<f32>> = data.remove(&Channel::Port).unwrap();
	let starboard_pings: Vec<Ping<f32>> = data.remove(&Channel::Starboard).unwrap();

	let mut port_map: BTreeMap<_,_> = port_pings.into_iter().map(|ping| (ping.timestamp, ping)).collect();
	let mut starboard_map: BTreeMap<_,_> = starboard_pings.into_iter().map(|ping| (ping.timestamp,ping)).collect();

	let port_ts: BTreeSet<_> = port_map.keys().cloned().collect(); // Port timestamps
	let starboard_ts: BTreeSet<_> = starboard_map.keys().cloned().collect(); // Starboard timestamps

	let ts: Vec<_> = port_ts.intersection(&starboard_ts).collect();

	let port_data: Vec<Vec<f32>> = ts.iter().map(|t| port_map.remove(t).unwrap().data).collect();
	let starboard_data: Vec<Vec<f32>> = ts.iter().map(|t| starboard_map.remove(t).unwrap().data).collect();

	let row_max = port_data.len();
	
	let data_len = port_data[0].len();
        let padding = vec![0.0f32; 256 - (data_len % 256)];

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
