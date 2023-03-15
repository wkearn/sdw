use waterfall;

use sdw::{
    model::{Ping, SonarDataRecord},
    parser::jsf,
};

use itertools::Itertools;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let jsf =
        jsf::File::open("/home/wkearn/Documents/projects/SDW/assets/HE501/HE501_Hydro1_007.jsf")?;

    let (port_data, starboard_data): (Vec<_>, Vec<_>) = jsf
        .filter_map(|msg| match SonarDataRecord::from(msg.unwrap()) {
            SonarDataRecord::Ping(Ping { data, .. }) => Some(data),
            _ => None,
        })
        .tuples::<(_, _)>()
        .unzip();

    let port_data: Vec<f32> = port_data.into_iter().flatten().collect();
    let starboard_data: Vec<f32> = starboard_data.into_iter().flatten().collect();

    pollster::block_on(waterfall::run(port_data,starboard_data));
    Ok(())
}
