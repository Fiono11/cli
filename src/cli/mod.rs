pub mod commands;
pub mod errors;

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
    SubmitTransaction {
        #[arg(long)]
        files: String,
        #[arg(long, default_value = "ws://127.0.0.1:9944")]
        url: String,
        #[arg(long, default_value = "System")]
        pallet: String,
        #[arg(long, default_value = "remark")]
        function: String,
        #[arg(
            long,
            default_value = "[]",
            help = "Call arguments in JSON array format"
        )]
        call_args: String,
    },
}
