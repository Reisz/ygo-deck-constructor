use clap::{builder::ArgAction, Parser};

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long = "no-cache", action = ArgAction::SetFalse, help = "Always download data")]
    pub cache: bool,
}
