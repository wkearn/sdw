//! Command line sonar data management tools
use clap::Parser;

pub mod avro;
pub mod count;
pub mod info;
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
        #[arg(short = 'z', long)]
        compress: bool,
    },
    Info {
        path: std::path::PathBuf,
    },
}

/// Command line arguments
///
/// This wraps an Action, which is where everything happens
#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    cmd: Action,
}

impl Args {
    fn dispatch(&self) -> std::io::Result<()> {
        match &self.cmd {
            Action::Count { path, output } => {
                count::count(path, output)?;
            }
            Action::List { path, output } => {
                list::list(path, output)?;
            }
            Action::Avro {
                path,
                output,
                compress,
            } => {
                avro::avro(path, output, compress)?;
            }
            Action::Info { path } => info::info(path)?,
        };
        Ok(())
    }

    /// Run the CLI
    pub fn run() -> std::io::Result<()> {
        Args::parse().dispatch()
    }
}
