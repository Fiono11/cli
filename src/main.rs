use anyhow::Error;
use clap::Parser;
use cli::Cli;

mod cli;
mod rpc;
mod structs;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Error> {
    /*let mut rng = OsRng;
    let keypair1 = SigningKeypair::generate(&mut rng);
    let keypair2 = SigningKeypair::generate(&mut rng);

    println!("keypair1 sk: {:?}", keypair1.secret_key);
    println!("keypair1 pk: {:?}", keypair1.verifying_key.as_bytes());
    println!("keypair2 sk: {:?}", keypair2.secret_key);
    println!("keypair2 pk: {:?}", keypair2.verifying_key.as_bytes());*/

    let cli = Cli::parse();
    cli.run().await
}
