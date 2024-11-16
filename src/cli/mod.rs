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
    GenerateThresholdPublicKeyRound1 {
        #[arg(long)]
        threshold: u16,
        #[arg(long)]
        files: String,
    },
    GenerateThresholdPublicKeyRound2 {
        #[arg(long)]
        files: String,
    },
    ThresholdSignRound1 {
        #[arg(long)]
        files: String,
    },
    ThresholdSignRound2 {
        #[arg(long)]
        files: String,
        #[arg(long, default_value = "ws://127.0.0.1:9944")]
        url: String,
        #[arg(long, default_value = "System")]
        pallet: String,
        #[arg(long, default_value = "remark")]
        call_name: String,
        #[arg(long, default_value = "((197, 38))")]
        call_data: String,
        #[arg(long, default_value = "substrate")]
        context: String,
    },
    AggregateThresholdSignature {
        #[arg(long)]
        files: String,
    },
    SubmitThresholdTransaction {
        #[arg(long)]
        files: String,
    },
}
