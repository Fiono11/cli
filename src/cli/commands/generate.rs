use crate::{cli::CliError, files::FilePaths};
use schnorrkel::{olaf::simplpedpop::AllMessage, MiniSecretKey, PublicKey};
use subxt::utils::AccountId32;
use tokio::{fs::{read_to_string, File}, io::AsyncWriteExt};
use hex;
use sp_core::crypto::Ss58Codec; 

/// Generates the message of round 1 of a participant to send to all participants (including itself)
pub async fn generate_threshold_public_key_round1(threshold: u16, participant: u16, files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let secret_key_file_content = read_to_string(file_paths.contributor_secret_key(participant))
        .await
        .map_err(|e| CliError(format!("Failed to read contributor secret key file for participant {}: {}", participant, e)))?;
    
    let secret_key_string: String = serde_json::from_str(&secret_key_file_content)
        .map_err(|e| CliError(format!("Failed to deserialize secret key content for participant {}: {}", participant, e)))?;
    
    let secret_key_string = secret_key_string.trim();
    let secret_key_hex = secret_key_string.strip_prefix("0x").unwrap_or(secret_key_string);
    let secret_key_bytes = hex::decode(secret_key_hex)
        .map_err(|e| CliError(format!("Failed to decode hex secret key for participant {}: {}", participant, e)))?;
    
    let keypair = MiniSecretKey::from_bytes(&secret_key_bytes)
        .map_err(|e| CliError(format!("Failed to generate keypair from secret key bytes for participant {}: {}", participant, e)))?
        .expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);

    let recipients_file_content = read_to_string(file_paths.recipients())
        .await
        .map_err(|e| CliError(format!("Failed to read recipients file: {}", e)))?;
    
    let recipients_strings: Vec<String> = serde_json::from_str(&recipients_file_content)
        .map_err(|e| CliError(format!("Failed to deserialize recipients data: {}", e)))?;
    
    let recipients: Vec<PublicKey> = recipients_strings
        .iter()
        .map(|recipient| {
            let account_id = sp_core::crypto::AccountId32::from_ss58check(recipient)
                .map_err(|e| CliError(format!("Invalid SS58 recipient {}: {}", recipient, e)))?;
            let public_key_bytes = account_id.as_ref();
            PublicKey::from_bytes(public_key_bytes)
                .map_err(|e| CliError(format!("Failed to parse public key for recipient {}: {}", recipient, e)))
        })
        .collect::<Result<_, _>>()?;

    let all_message: AllMessage = keypair.simplpedpop_contribute_all(threshold, recipients)
        .map_err(|e| CliError(format!("Failed to generate AllMessage: {:?}", e)))?;
    let all_message_bytes: Vec<u8> = all_message.to_bytes();

    let mut all_message_vec: Vec<Vec<u8>> = if tokio::fs::metadata(file_paths.all_messages()).await.is_ok() {
        let existing_content = read_to_string(file_paths.all_messages())
            .await
            .map_err(|e| CliError(format!("Failed to read existing all_messages file: {}", e)))?;
        serde_json::from_str(&existing_content)
            .map_err(|e| CliError(format!("Failed to deserialize existing all_messages data: {}", e)))?
    } else {
        Vec::new()
    };
    all_message_vec.push(all_message_bytes);
    let all_message_json = serde_json::to_string(&all_message_vec)
        .map_err(|e| CliError(format!("Failed to serialize all_messages data: {}", e)))?;

    let mut all_message_file = File::create(file_paths.all_messages())
        .await
        .map_err(|e| CliError(format!("Failed to create all_messages file: {}", e)))?;
    
    all_message_file
        .write_all(all_message_json.as_bytes())
        .await
        .map_err(|e| CliError(format!("Failed to write all_messages data to file: {}", e)))?;

    let account_id = AccountId32(keypair.public.to_bytes());

    println!("The owner of account {} completed round 1 of Threshold Public Key generation successfully!", account_id);
    println!("The message to all participants was written to: {:?}", file_paths.all_messages());

    Ok(())
}

/// Generates the threshold public key and the corresponding secret secret share of the participant, from the messages of round 1 of all participants (including itself)
pub async fn generate_threshold_public_key_round2(participant: u16, files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let secret_key_file_content = read_to_string(file_paths.contributor_secret_key(participant))
        .await
        .map_err(|e| CliError(format!("Failed to read contributor secret key file for participant {}: {}", participant, e)))?;
    
    let secret_key_string: String = serde_json::from_str(&secret_key_file_content)
        .map_err(|e| CliError(format!("Failed to deserialize secret key content for participant {}: {}", participant, e)))?;
    
    let secret_key_string = secret_key_string.trim();
    let secret_key_hex = secret_key_string.strip_prefix("0x").unwrap_or(secret_key_string);
    let secret_key_bytes = hex::decode(secret_key_hex)
        .map_err(|e| CliError(format!("Failed to decode hex secret key for participant {}: {}", participant, e)))?;
    
    let keypair = MiniSecretKey::from_bytes(&secret_key_bytes)
        .map_err(|e| CliError(format!("Failed to generate keypair from secret key bytes for participant {}: {}", participant, e)))?
        .expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);

    let all_messages_string = read_to_string(file_paths.all_messages())
        .await
        .map_err(|e| CliError(format!("Failed to read all_messages file: {}", e)))?;
    
    let all_messages_bytes: Vec<Vec<u8>> = serde_json::from_str(&all_messages_string)
        .map_err(|e| CliError(format!("Failed to deserialize all_messages data: {}", e)))?;

    let all_messages: Vec<AllMessage> = all_messages_bytes
        .iter()
        .map(|all_message| AllMessage::from_bytes(all_message)
            .map_err(|e| CliError(format!("Failed to parse AllMessage: {:?}", e))))
        .collect::<Result<_, _>>()?;

    let simplpedpop = keypair.simplpedpop_recipient_all(&all_messages)
        .map_err(|e| CliError(format!("Failed to process AllMessages for participant {}: {:?}", participant, e)))?;
    
    let generation_output = simplpedpop.0;
    let output_json = serde_json::to_string(&generation_output.to_bytes())
        .map_err(|e| CliError(format!("Failed to serialize generation output: {}", e)))?;
    
    let mut output_file = File::create(file_paths.generation_output(participant))
        .await
        .map_err(|e| CliError(format!("Failed to create generation output file for participant {}: {:?}", participant, e)))?;
    
    output_file
        .write_all(output_json.as_bytes())
        .await
        .map_err(|e| CliError(format!("Failed to write generation output to file: {}", e)))?;

    let signing_share = simplpedpop.1;
    let signing_share_json = serde_json::to_string(&signing_share.to_bytes().to_vec())
        .map_err(|e| CliError(format!("Failed to serialize signing share: {}", e)))?;

    let mut signing_share_file = File::create(file_paths.signing_share(participant))
        .await
        .map_err(|e| CliError(format!("Failed to create signing share file for participant {}: {}", participant, e)))?;
    
    signing_share_file
        .write_all(signing_share_json.as_bytes())
        .await
        .map_err(|e| CliError(format!("Failed to write signing share to file: {}", e)))?;

    let threshold_public_key = AccountId32(generation_output.spp_output().threshold_public_key().0.to_bytes());
    let threshold_public_key_json = serde_json::to_string(&threshold_public_key)
        .map_err(|e| CliError(format!("Failed to serialize threshold public key: {}", e)))?;

    let mut threshold_public_key_file = File::create(file_paths.threshold_public_key())
        .await
        .map_err(|e| CliError(format!("Failed to create threshold public key file: {}", e)))?;
    
    threshold_public_key_file
        .write_all(threshold_public_key_json.as_bytes())
        .await
        .map_err(|e| CliError(format!("Failed to write threshold public key to file: {}", e)))?;

    println!("The owner of account {} completed round 2 of Threshold Public Key generation successfully!", threshold_public_key);
    println!("The output message was written to: {:?}", file_paths.generation_output(participant)); 
    println!("The signing share was written to: {:?}", file_paths.signing_share(participant)); 
    println!("The Threshold Public Key is {} and was written to: {:?}", threshold_public_key, file_paths.threshold_public_key());  

    Ok(())
}