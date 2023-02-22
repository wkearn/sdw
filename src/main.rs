use clap::Parser;
use sdw::cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::Args::parse().run()?;

    Ok(())
}
