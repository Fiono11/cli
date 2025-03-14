use anyhow::Error;
use clap::Parser;
use cli::Cli;
use ed25519_dalek::SigningKey;
use olaf::SigningKeypair;
use rand_core::OsRng;

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

    /*let vec1= hex::decode("7F2FDB751E6D4DF05F5F4680A2AC0B219164A08532AE1222EDBE3FDBF733001A").unwrap();
    let mut pk1 = [0u8; 32];
    pk1.copy_from_slice(&vec1);

    let vec2= hex::decode("95AA66A09FDF202A57224F519A3EFC1FF4887FFA589E382FD6D2B27890E350D6").unwrap();
    let mut sk1 = [0u8; 32];
    sk1.copy_from_slice(&vec2);

    let vec3= hex::decode("9594E6E7AA3007F0F32CA3E12592E3A05CEC4B23BEDF9FD148283AEE2BE8D47C").unwrap();
    let mut pk2 = [0u8; 32];
    pk2.copy_from_slice(&vec3);

    let vec4= hex::decode("FD57753256CFF0B165E54D9EF476B8CDD37C33A732073761BE741FD69A180B63").unwrap();
    let mut sk2 = [0u8; 32];
    sk2.copy_from_slice(&vec4);

    println!("{:?}", pk1);
    println!("{:?}", sk1);
    println!("{:?}", pk2);
    println!("{:?}", sk2);*/

    let cli = Cli::parse();
    cli.run().await
}
