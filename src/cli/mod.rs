use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Action,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    Count { path: std::path::PathBuf },
    List { path: std::path::PathBuf },
    Avro { path: std::path::PathBuf,
	   output: std::path::PathBuf}
}
