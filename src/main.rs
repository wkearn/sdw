use clap::Parser;

use sdw::cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Args::parse();

    cli::run(args)?;

    Ok(())
}
