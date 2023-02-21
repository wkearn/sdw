use clap::Parser;

use sdw::cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Args::parse();

    match args.cmd {
        cli::Action::Count { path, output } => {
            cli::count::count(path, output)?;
        }
        cli::Action::List { path, output } => {
            cli::list::list(path, output)?;
        }
        cli::Action::Avro { path, output } => {
            cli::avro::avro(path, output)?;
        }
    };

    Ok(())
}
