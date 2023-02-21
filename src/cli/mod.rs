//! Command line sonar data management tools
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    pub cmd: Action,
}

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

pub fn run() -> std::io::Result<()> {
    let args = Args::parse();

    match args.cmd {
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

pub mod avro;
pub mod count;
pub mod list;
