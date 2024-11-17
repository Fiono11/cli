use crate::{cli::errors::CliError, files::FilePaths};
use schnorrkel::{olaf::simplpedpop::AllMessage, MiniSecretKey, PublicKey};
use serde_json::from_str;
use subxt::utils::AccountId32;
use tokio::{
    fs::{read_to_string, File},
    io::AsyncWriteExt,
};
use hex;
use sp_core::crypto::Ss58Codec; 

pub async fn generate_threshold_public_key_round1(threshold: u16, files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let secret_key_file_content = read_to_string(file_paths.contributor_secret_key()).await?;
    let secret_key_string: String = serde_json::from_str(&secret_key_file_content)?;
    let secret_key_string = secret_key_string.trim();
    let secret_key_hex = secret_key_string.strip_prefix("0x").unwrap_or(secret_key_string);
    let secret_key_bytes = hex::decode(secret_key_hex)?;
    let keypair = MiniSecretKey::from_bytes(&secret_key_bytes)?.expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);

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

    let all_message: AllMessage = keypair.simplpedpop_contribute_all(threshold, recipients)?; 
    let all_message_bytes: Vec<u8> = all_message.to_bytes();
    let all_message_vec: Vec<Vec<u8>> = vec![all_message_bytes];
    let all_message_json = serde_json::to_string(&all_message_vec)?;

    let mut all_message_file = File::create(file_paths.all_messages()).await?;
    all_message_file
        .write_all(all_message_json.as_bytes())
        .await?;

    let account_id = AccountId32(keypair.public.to_bytes());

    println!("The owner of account {} completed round 1 of Threshold Public Key generation successfully!", account_id);
    println!("The message to all participants was written to: {:?}", file_paths.all_messages());    

    Ok(())
}

pub async fn generate_threshold_public_key_round2(files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let secret_key_file_content = read_to_string(file_paths.contributor_secret_key()).await?;
    let secret_key_string: String = serde_json::from_str(&secret_key_file_content)?;
    let secret_key_string = secret_key_string.trim();
    let secret_key_hex = secret_key_string.strip_prefix("0x").unwrap_or(secret_key_string);
    let secret_key_bytes = hex::decode(secret_key_hex)?;
    let keypair = MiniSecretKey::from_bytes(&secret_key_bytes)?.expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);

    let all_messages_string = read_to_string(file_paths.all_messages()).await?;
    let all_messages_bytes: Vec<Vec<u8>> = from_str(&all_messages_string)?;

    let all_messages: Vec<AllMessage> = all_messages_bytes
        .iter()
        .map(|all_message| AllMessage::from_bytes(all_message))
        .collect::<Result<_, _>>()?;

    let simplpedpop = keypair.simplpedpop_recipient_all(&all_messages)?;
    let spp_output = simplpedpop.0;
    let output_json = serde_json::to_string(&spp_output.to_bytes())?;

    let mut output_file = File::create(file_paths.spp_output()).await?;
    output_file.write_all(output_json.as_bytes()).await?;

    let signing_share = simplpedpop.1;
    let signing_share_json = serde_json::to_string(&signing_share.to_bytes().to_vec())?;

    let mut signing_share_file = File::create(file_paths.signing_share()).await?;
    signing_share_file
        .write_all(signing_share_json.as_bytes())
        .await?;

    let threshold_public_key =
        AccountId32(spp_output.spp_output().threshold_public_key().0.to_bytes());
    let threshold_public_key_json =
        serde_json::to_string(&threshold_public_key)?;

    let mut threshold_public_key_file = File::create(file_paths.threshold_public_key()).await?;
    threshold_public_key_file
        .write_all(threshold_public_key_json.as_bytes())
        .await?;

    println!("The owner of account {} completed round 2 of Threshold Public Key generation successfully!", threshold_public_key);
    println!("The output message was written to: {:?}", file_paths.spp_output()); 
    println!("The signing share was written to: {:?}", file_paths.signing_share()); 
    println!("The Threshold Public Key is {} and was written to: {:?}", threshold_public_key, file_paths.threshold_public_key());  

    Ok(())
}
