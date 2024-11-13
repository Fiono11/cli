use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "app", about = "An application.", version = "1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    SimplpedpopRound1 {
        #[arg(long)]
        files: String,
    },
    SimplpedpopRound2 {
        #[arg(long)]
        files: String,
    },
    MultisigRound1 {
        #[arg(long)]
        files: String,
    },
    MultisigRound2 {
        #[arg(long)]
        files: String,
    },
    MultisigAggregate {
        #[arg(long)]
        files: String,
    },
}
