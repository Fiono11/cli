use crate::{cli::errors::CliError, files::FilePaths};
use schnorrkel::{olaf::simplpedpop::AllMessage, Keypair, MiniSecretKey, PublicKey, SecretKey};
use serde_json::from_str;
use subxt::utils::AccountId32;
use tokio::{
    fs::{read_to_string, File},
    io::AsyncWriteExt,
};
use hex;
use sp_core::crypto::Ss58Codec; 

pub async fn simplpedpop_round1(files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    // Read and parse the secret key from hex string
    let secret_key_file_content = read_to_string(file_paths.contributor_secret_key()).await?;
    let secret_key_string: String = serde_json::from_str(&secret_key_file_content)?;
    let secret_key_string = secret_key_string.trim();
    let secret_key_hex = secret_key_string.strip_prefix("0x").unwrap_or(secret_key_string);
    let secret_key_bytes = hex::decode(secret_key_hex)?;
    let keypair = MiniSecretKey::from_bytes(&secret_key_bytes)?.expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);

    let account_id = AccountId32(keypair.public.to_bytes());
    println!("pk: {:?}", account_id.to_string());

    // Read and parse the recipients from JSON array of strings
    let recipients_file_content = read_to_string(file_paths.recipients()).await?;
    let recipients_strings: Vec<String> = serde_json::from_str(&recipients_file_content).map_err(CliError::Serde)?;
    let recipients: Vec<PublicKey> = recipients_strings
        .iter()
        .map(|recipient| {
            let account_id = sp_core::crypto::AccountId32::from_ss58check(recipient)
                .map_err(CliError::Account)?;
            let public_key_bytes = account_id.as_ref();
            PublicKey::from_bytes(public_key_bytes).map_err(CliError::Signature)
        })
        .collect::<Result<_, _>>()?;

    let all_message: AllMessage = keypair.simplpedpop_contribute_all(2, recipients)?;
    let all_message_bytes: Vec<u8> = all_message.to_bytes();
    let all_message_vec: Vec<Vec<u8>> = vec![all_message_bytes];
    let all_message_json = serde_json::to_string_pretty(&all_message_vec)?;

    let mut all_message_file = File::create(file_paths.all_messages()).await?;
    all_message_file
        .write_all(all_message_json.as_bytes())
        .await?;

    Ok(())
}

// Simplpedpop Round 2
pub async fn simplpedpop_round2(files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let secret_key_string = read_to_string(file_paths.contributor_secret_key()).await?;
    let secret_key_bytes: Vec<u8> = from_str(&secret_key_string)?;
    let keypair = Keypair::from(SecretKey::from_bytes(&secret_key_bytes)?);

    let all_messages_string = read_to_string(file_paths.all_messages()).await?;
    let all_messages_bytes: Vec<Vec<u8>> = from_str(&all_messages_string)?;

    let all_messages: Vec<AllMessage> = all_messages_bytes
        .iter()
        .map(|all_message| AllMessage::from_bytes(all_message))
        .collect::<Result<_, _>>()?;

    let simplpedpop = keypair.simplpedpop_recipient_all(&all_messages)?;
    let spp_output = simplpedpop.0;
    let output_json = serde_json::to_string_pretty(&spp_output.to_bytes())?;

    let mut output_file = File::create(file_paths.spp_output()).await?;
    output_file.write_all(output_json.as_bytes()).await?;

    let signing_share = simplpedpop.1;
    let signing_share_json = serde_json::to_string_pretty(&signing_share.to_bytes().to_vec())?;

    let mut signing_share_file = File::create(file_paths.signing_share()).await?;
    signing_share_file
        .write_all(signing_share_json.as_bytes())
        .await?;

    let threshold_public_key = spp_output.spp_output().threshold_public_key();
    let threshold_public_key_json =
        serde_json::to_string_pretty(&threshold_public_key.0.to_bytes())?;

    let mut threshold_public_key_file = File::create(file_paths.threshold_public_key()).await?;
    threshold_public_key_file
        .write_all(threshold_public_key_json.as_bytes())
        .await?;

    Ok(())
}
