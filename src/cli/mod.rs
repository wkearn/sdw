use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Action,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
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

pub fn run(args: Args) -> std::io::Result<()> {
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
