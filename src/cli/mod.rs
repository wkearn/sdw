//! Command line sonar data management tools
use clap::Parser;

pub mod avro;
pub mod count;
pub mod list;

#[derive(clap::Subcommand, Debug)]
enum Action {
    Count {
        path: std::path::PathBuf,
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },
    List {
        path: std::path::PathBuf,
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },
    Avro {
        path: std::path::PathBuf,
        output: std::path::PathBuf,
    },
}

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    cmd: Action,
}

impl Args {
    pub fn run(&self) -> std::io::Result<()> {
        match &self.cmd {
            Action::Count { path, output } => {
                count::count(path, output)?;
            }
            Action::List { path, output } => {
                list::list(path, output)?;
            }
            Action::Avro { path, output } => {
                avro::avro(path, output)?;
            }
        };
        Ok(())
    }
}
