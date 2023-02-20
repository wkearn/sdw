use clap::Parser;

#[derive(Parser,Debug)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Action,    
    pub path: std::path::PathBuf,
}

#[derive(clap::Subcommand,Debug)]
pub enum Action {
    Count    
}
