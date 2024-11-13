use crate::{errors::CliError, files::FilePaths};
use schnorrkel::{olaf::simplpedpop::AllMessage, Keypair, PublicKey, SecretKey};
use serde_json::from_str;
use std::{
    fs::{self, File},
    io::Write,
};

// Simplpedpop Round 1
pub fn simplpedpop_round1(files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let secret_key_string = fs::read_to_string(file_paths.contributor_secret_key())?;
    let secret_key_bytes: Vec<u8> = from_str(&secret_key_string)?;
    let keypair = Keypair::from(SecretKey::from_bytes(&secret_key_bytes)?);

    let recipients_string = fs::read_to_string(file_paths.recipients())?;
    let recipients_bytes: Vec<Vec<u8>> = from_str(&recipients_string)?;

    let recipients: Vec<PublicKey> = recipients_bytes
        .iter()
        .map(|recipient| PublicKey::from_bytes(recipient))
        .collect::<Result<_, _>>()?;

    let all_message: AllMessage = keypair.simplpedpop_contribute_all(2, recipients)?;
    let all_message_bytes: Vec<u8> = all_message.to_bytes();
    let all_message_vec: Vec<Vec<u8>> = vec![all_message_bytes];
    let all_message_json = serde_json::to_string_pretty(&all_message_vec)?;

    let mut all_message_file = File::create(file_paths.all_messages())?;
    all_message_file.write_all(all_message_json.as_bytes())?;

    Ok(())
}

// Simplpedpop Round 2
pub fn simplpedpop_round2(files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let secret_key_string = fs::read_to_string(file_paths.recipient_secret_key())?;
    let secret_key_bytes: Vec<u8> = from_str(&secret_key_string)?;
    let keypair = Keypair::from(SecretKey::from_bytes(&secret_key_bytes)?);

    let all_messages_string = fs::read_to_string(file_paths.all_messages())?;
    let all_messages_bytes: Vec<Vec<u8>> = from_str(&all_messages_string)?;

    let all_messages: Vec<AllMessage> = all_messages_bytes
        .iter()
        .map(|all_message| AllMessage::from_bytes(all_message))
        .collect::<Result<_, _>>()?;

    let simplpedpop = keypair.simplpedpop_recipient_all(&all_messages)?;
    let spp_output = simplpedpop.0;
    let output_json = serde_json::to_string_pretty(&spp_output.to_bytes())?;

    let mut output_file = File::create(file_paths.spp_output())?;
    output_file.write_all(output_json.as_bytes())?;

    let signing_share = simplpedpop.1;
    let signing_share_json = serde_json::to_string_pretty(&signing_share.to_bytes().to_vec())?;

    let mut signing_share_file = File::create(file_paths.signing_share())?;
    signing_share_file.write_all(signing_share_json.as_bytes())?;

    let threshold_public_key = spp_output.spp_output().threshold_public_key();
    let threshold_public_key_json =
        serde_json::to_string_pretty(&threshold_public_key.0.to_bytes())?;

    let mut threshold_public_key_file = File::create(file_paths.threshold_public_key())?;
    threshold_public_key_file.write_all(threshold_public_key_json.as_bytes())?;

    Ok(())
}
