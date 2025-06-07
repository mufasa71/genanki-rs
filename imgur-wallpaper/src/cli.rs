use clap::{Parser, ValueEnum};
use strum_macros::Display;

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, ValueEnum, Display)]
pub enum Time {
    #[strum(serialize = "hour")]
    Hour,
    #[strum(serialize = "day")]
    Day,
    #[strum(serialize = "week")]
    Week,
    #[strum(serialize = "month")]
    Month,
    #[strum(serialize = "year")]
    Year,
    #[strum(serialize = "all")]
    All,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub output: Option<String>,
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
    #[arg(short, long, long_help = "the maximum number of items desired (default: 25, maximum: 100)", value_parser = clap::value_parser!(u8).range(1..100))]
    pub limit: Option<u8>,
    #[arg(short, value_enum)]
    pub t: Option<Time>,
}
