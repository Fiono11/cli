use clap::{Parser, Subcommand};
use schnorrkel::{
    olaf::{
        frost::{aggregate, SigningCommitments, SigningNonces, SigningPackage},
        simplpedpop::{AllMessage, SPPOutputMessage},
        SigningKeypair,
    },
    Keypair, PublicKey, SecretKey, Signature,
};
use serde_json::from_str;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    config::PolkadotExtrinsicParamsBuilder,
    dynamic::Value,
    utils::{AccountId32, MultiAddress},
    OnlineClient, PolkadotConfig,
};

const CONTEXT: &[u8] = b"substrate";

// Generate an interface that we can use from the node's metadata.
#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*let keypair1 = Keypair::generate();
    let keypair2 = Keypair::generate();

    println!("keypair1 sk: {:?}", keypair1.secret.to_bytes());
    println!("keypair1 pk: {:?}", keypair1.public.to_bytes());
    println!("keypair2 sk: {:?}", keypair2.secret.to_bytes());
    println!("keypair2 pk: {:?}", keypair2.public.to_bytes());*/

    let cli = Cli::parse();

    match cli.command {
        Commands::SimplpedpopRound1 { files } => {
            let secret_key_string =
                fs::read_to_string(Path::new(&files).join("contributor_secret_key.json")).unwrap();

            let secret_key_bytes: Vec<u8> = from_str(&secret_key_string).unwrap();

            let keypair = Keypair::from(SecretKey::from_bytes(&secret_key_bytes).unwrap());

            let recipients_string =
                fs::read_to_string(Path::new(&files).join("recipients.json")).unwrap();

            let recipients_bytes: Vec<Vec<u8>> = from_str(&recipients_string).unwrap();

            let recipients: Vec<PublicKey> = recipients_bytes
                .iter()
                .map(|recipient| PublicKey::from_bytes(recipient).unwrap())
                .collect();

            let all_message: AllMessage =
                keypair.simplpedpop_contribute_all(2, recipients).unwrap();

            let all_message_bytes: Vec<u8> = all_message.to_bytes();

            let all_message_vec: Vec<Vec<u8>> = vec![all_message_bytes];

            let all_message_json = serde_json::to_string_pretty(&all_message_vec).unwrap();

            let mut all_message_file =
                File::create(Path::new(&files).join("all_messages.json")).unwrap();

            all_message_file
                .write_all(&all_message_json.as_bytes())
                .unwrap();
        }
        Commands::SimplpedpopRound2 { files } => {
            let secret_key_string =
                fs::read_to_string(Path::new(&files).join("recipient_secret_key.json")).unwrap();

            let secret_key_bytes: Vec<u8> = from_str(&secret_key_string).unwrap();

            let keypair = Keypair::from(SecretKey::from_bytes(&secret_key_bytes).unwrap());

            let all_messages_string =
                fs::read_to_string(Path::new(&files).join("all_messages.json")).unwrap();

            let all_messages_bytes: Vec<Vec<u8>> = from_str(&all_messages_string).unwrap();

            let all_messages: Vec<AllMessage> = all_messages_bytes
                .iter()
                .map(|all_message| AllMessage::from_bytes(all_message).unwrap())
                .collect();

            let simplpedpop = keypair.simplpedpop_recipient_all(&all_messages).unwrap();

            let spp_output = simplpedpop.0.clone();

            let output_json = serde_json::to_string_pretty(&spp_output.to_bytes()).unwrap();

            let mut output_file = File::create(Path::new(&files).join("spp_output.json")).unwrap();

            output_file.write_all(&output_json.as_bytes()).unwrap();

            let signing_share = simplpedpop.1;

            let signing_share_json =
                serde_json::to_string_pretty(&signing_share.to_bytes().to_vec()).unwrap();

            let mut signing_share_file =
                File::create(Path::new(&files).join("signing_share.json")).unwrap();

            signing_share_file
                .write_all(&signing_share_json.as_bytes())
                .unwrap();

            let threshold_public_key = simplpedpop.0.spp_output().threshold_public_key();

            let threshold_public_key_json =
                serde_json::to_string_pretty(&threshold_public_key.0.to_bytes()).unwrap();

            let mut threshold_public_key_file =
                File::create(Path::new(&files).join("threshold_public_key.json")).unwrap();

            threshold_public_key_file
                .write_all(threshold_public_key_json.as_bytes())
                .unwrap();
        }
        Commands::FrostRound1 { files } => {
            let signing_share_string =
                fs::read_to_string(Path::new(&files).join("signing_share.json")).unwrap();

            let signing_share_bytes: Vec<u8> = from_str(&signing_share_string).unwrap();

            let signing_share = SigningKeypair::from_bytes(&signing_share_bytes).unwrap();

            let (signing_nonces, signing_commitments) = signing_share.commit();

            let signing_nonces_json =
                serde_json::to_string_pretty(&signing_nonces.to_bytes().to_vec()).unwrap();

            let mut signing_nonces_file =
                File::create(Path::new(&files).join("signing_nonces.json")).unwrap();

            signing_nonces_file
                .write_all(&signing_nonces_json.as_bytes())
                .unwrap();

            let signing_commitments_vec = vec![signing_commitments.to_bytes().to_vec()];

            let signing_commitments_json =
                serde_json::to_string_pretty(&signing_commitments_vec).unwrap();

            let mut signing_commitments_file =
                File::create(Path::new(&files).join("signing_commitments.json")).unwrap();

            signing_commitments_file
                .write_all(&signing_commitments_json.as_bytes())
                .unwrap();
        }
        Commands::FrostRound2 { files } => {
            let signing_commitments_string =
                fs::read_to_string(Path::new(&files).join("signing_commitments.json")).unwrap();

            let signing_commitments_bytes: Vec<Vec<u8>> =
                from_str(&signing_commitments_string).unwrap();

            let signing_commitments: Vec<SigningCommitments> = signing_commitments_bytes
                .iter()
                .map(|signing_commitments| {
                    SigningCommitments::from_bytes(signing_commitments).unwrap()
                })
                .collect();

            let signing_nonces_string =
                fs::read_to_string(Path::new(&files).join("signing_nonces.json")).unwrap();

            let signing_nonces_bytes: Vec<u8> = from_str(&signing_nonces_string).unwrap();

            let signing_nonces = SigningNonces::from_bytes(&signing_nonces_bytes).unwrap();

            let signing_share_string =
                fs::read_to_string(Path::new(&files).join("signing_share.json")).unwrap();

            let signing_share_bytes: Vec<u8> = from_str(&signing_share_string).unwrap();

            let signing_share = SigningKeypair::from_bytes(&signing_share_bytes).unwrap();

            let output_string =
                fs::read_to_string(Path::new(&files).join("spp_output.json")).unwrap();

            let output_bytes: Vec<u8> = from_str(&output_string).unwrap();

            let spp_output = SPPOutputMessage::from_bytes(&output_bytes).unwrap();

            let threshold_public_key_string =
                fs::read_to_string(Path::new(&files).join("threshold_public_key.json")).unwrap();

            let threshold_public_key_bytes: Vec<u8> =
                from_str(&threshold_public_key_string).unwrap();

            let threshold_public_key = PublicKey::from_bytes(&threshold_public_key_bytes).unwrap();

            let client = OnlineClient::<PolkadotConfig>::new().await?;

            let rpc_client = RpcClient::from_url("ws://127.0.0.1:9944").await?;

            let legacy_rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);

            let call =
                subxt::dynamic::tx("System", "remark", vec![Value::from_bytes("Hello there")]);

            let account_id = AccountId32(threshold_public_key.to_bytes());

            let nonce = legacy_rpc.system_account_next_index(&account_id).await?;

            let params = PolkadotExtrinsicParamsBuilder::new().nonce(nonce).build();

            let partial_tx = client.tx().create_partial_signed_offline(&call, params)?;

            let payload = partial_tx.signer_payload();

            let signing_package = signing_share
                .sign(
                    CONTEXT.to_vec(),
                    payload,
                    spp_output.spp_output(),
                    signing_commitments,
                    &signing_nonces,
                )
                .unwrap();

            let signing_packages_vec = vec![signing_package.to_bytes()];

            let signing_package_json = serde_json::to_string_pretty(&signing_packages_vec).unwrap();

            let mut signing_package_file =
                File::create(Path::new(&files).join("signing_packages.json")).unwrap();

            signing_package_file
                .write_all(&signing_package_json.as_bytes())
                .unwrap();
        }
        Commands::FrostAggregate { files } => {
            let threshold_public_key_string =
                fs::read_to_string(Path::new(&files).join("threshold_public_key.json")).unwrap();

            let threshold_public_key_bytes: Vec<u8> =
                from_str(&threshold_public_key_string).unwrap();

            let threshold_public_key = PublicKey::from_bytes(&threshold_public_key_bytes).unwrap();

            let account_id = AccountId32(threshold_public_key.to_bytes());

            println!("pk: {:?}", account_id.to_string());

            let signing_packages_string =
                fs::read_to_string(Path::new(&files).join("signing_packages.json")).unwrap();

            let signing_packages_bytes: Vec<Vec<u8>> = from_str(&signing_packages_string).unwrap();

            let signing_packages: Vec<SigningPackage> = signing_packages_bytes
                .iter()
                .map(|signing_commitments| SigningPackage::from_bytes(signing_commitments).unwrap())
                .collect();

            let group_signature: Signature = aggregate(&signing_packages).unwrap();

            let signature_json =
                serde_json::to_string_pretty(&group_signature.to_bytes().to_vec()).unwrap();

            let mut signature_file =
                File::create(Path::new(&files).join("signature.json")).unwrap();

            signature_file
                .write_all(&signature_json.as_bytes())
                .unwrap();

            let client = OnlineClient::<PolkadotConfig>::new().await?;

            let rpc_client = RpcClient::from_url("ws://127.0.0.1:9944").await?;

            let legacy_rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);

            let nonce = legacy_rpc.system_account_next_index(&account_id).await?;

            // Create a dummy tx payload to sign:
            let call =
                subxt::dynamic::tx("System", "remark", vec![Value::from_bytes("Hello there")]);

            let params = PolkadotExtrinsicParamsBuilder::new().nonce(nonce).build();

            let partial_tx = client.tx().create_partial_signed_offline(&call, params)?;

            //let payload = partial_tx.signer_payload();

            //let signing_package = SigningPackage::new(commitments_map, &payload[..], CONTEXT);

            let signature = subxt_signer::sr25519::Signature(group_signature.to_bytes());

            //let public_key = MultiAddress::Address32(container.group_public_key.to_bytes());
            let public_key: MultiAddress<AccountId32, _> =
                subxt_signer::sr25519::PublicKey(threshold_public_key.to_bytes()).into();

            // Now we can build an tx, which one can call `submit` or `submit_and_watch`
            // on to submit to a node and optionally watch the status.
            let tx = partial_tx.sign_with_address_and_signature(&public_key, &signature.into());

            tx.submit().await?;

            /*let output_json = serde_json::to_string_pretty(&group_signature)?;

            let mut output_file =
                File::create(Path::new(&output_dir).join("group_signature.json"))?;

            output_file.write_all(output_json.as_bytes())?;

            println!("FROST Aggregate data saved to {}", output_dir);*/
        }
    }
    Ok(())
}

#[derive(Parser)]
#[command(name = "app", about = "An application.", version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    SimplpedpopRound1 {
        #[arg(long)]
        files: String,
    },
    SimplpedpopRound2 {
        #[arg(long)]
        files: String,
    },
    FrostRound1 {
        #[arg(long)]
        files: String,
    },
    FrostRound2 {
        #[arg(long)]
        files: String,
    },
    FrostAggregate {
        #[arg(long)]
        files: String,
    },
}
