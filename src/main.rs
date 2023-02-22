use sdw::cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::Args::run()?;

    Ok(())
}
