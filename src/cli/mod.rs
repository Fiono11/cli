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
        participant: u16,
        #[arg(long, default_value = ".")]
        files: String,
    },
    GenerateThresholdPublicKeyRound2 {
        #[arg(long)]
        participant: u16,
        #[arg(long, default_value = ".")]
        files: String,
    },
    ThresholdSignRound1 {
        #[arg(long)]
        participant: u16,
        #[arg(long, default_value = ".")]
        files: String,
    },
    ThresholdSignRound2 {
        #[arg(long)]
        participant: u16,
        #[arg(long, default_value = ".")]
        files: String,
        #[arg(long, default_value = "wss://westend-rpc.polkadot.io")]
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
        #[arg(long, default_value = ".")]
        files: String,
    },
    SubmitThresholdExtrinsic {
        #[arg(long, default_value = ".")]
        files: String,
    },
}
