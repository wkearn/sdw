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

    pollster::block_on(waterfall::run(
        port_data,
        starboard_data,
        padded_len,
        row_max,
    ));
    Ok(())
}
