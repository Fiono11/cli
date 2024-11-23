use std::str::FromStr;
use crate::{cli::{commands::value_into_composite, CliError}, files::FilePaths};
use schnorrkel::olaf::{
    multisig::{SigningCommitments, SigningNonces},
    simplpedpop::SPPOutputMessage,
    SigningKeypair,
};
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient}, config::polkadot::PolkadotExtrinsicParamsBuilder, tx, utils::AccountId32, OnlineClient, PolkadotConfig
};
use tokio::{
    fs::{read_to_string, File},
    io::AsyncWriteExt,
};

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}

/// Generates the secret signing nonce and the corresponding public signing commitment of a participant
pub async fn threshold_sign_round1(participant: u16, files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let signing_share_string = read_to_string(file_paths.signing_share(participant))
        .await
        .map_err(|e| CliError(format!("Failed to read signing share file for participant {}: {}", participant, e)))?;
    
    let signing_share_bytes: Vec<u8> = serde_json::from_str(&signing_share_string)
        .map_err(|e| CliError(format!("Failed to deserialize signing share for participant {}: {}", participant, e)))?;
    
    let signing_share: SigningKeypair = SigningKeypair::from_bytes(&signing_share_bytes)
        .map_err(|e| CliError(format!("Failed to parse signing share for participant {}: {}", participant, e)))?;

    let (signing_nonces, signing_commitments) = signing_share.commit();

    let signing_nonces_json = serde_json::to_string(&signing_nonces.to_bytes().to_vec())
        .map_err(|e| CliError(format!("Failed to serialize signing nonces for participant {}: {}", participant, e)))?;
    
    let mut signing_nonces_file = File::create(file_paths.signing_nonce(participant))
        .await
        .map_err(|e| CliError(format!("Failed to create signing nonce file for participant {}: {}", participant, e)))?;
    
    signing_nonces_file
        .write_all(signing_nonces_json.as_bytes())
        .await
        .map_err(|e| CliError(format!("Failed to write signing nonces to file for participant {}: {}", participant, e)))?;

    let mut signing_commitments_vec: Vec<Vec<u8>> = if tokio::fs::metadata(file_paths.signing_commitments()).await.is_ok() {
        let existing_content = read_to_string(file_paths.signing_commitments())
            .await
            .map_err(|e| CliError(format!("Failed to read existing signing commitments file: {}", e)))?;
        serde_json::from_str(&existing_content)
            .map_err(|e| CliError(format!("Failed to deserialize existing signing commitments: {}", e)))?
    } else {
        Vec::new()
    };
    signing_commitments_vec.push(signing_commitments.to_bytes().to_vec());

    let signing_commitments_json = serde_json::to_string(&signing_commitments_vec)
        .map_err(|e| CliError(format!("Failed to serialize signing commitments: {}", e)))?;

    let mut signing_commitments_file = File::create(file_paths.signing_commitments())
        .await
        .map_err(|e| CliError(format!("Failed to create signing commitments file: {}", e)))?;
    
    signing_commitments_file
        .write_all(signing_commitments_json.as_bytes())
        .await
        .map_err(|e| CliError(format!("Failed to write signing commitments to file: {}", e)))?;

    println!("Round 1 of threshold signing was completed successfully!");
    println!("Signing nonce was written to: {:?}", file_paths.signing_nonce(participant));
    println!(
        "Signing commitment was written to: {:?}",
        file_paths.signing_commitments()
    );

    Ok(())
}

/// Generates the signing package of a participant from: 
/// - its secret signing nonce
/// - the public signing commitments of all participants (including itself)
/// - the public output of round 2 of the generation of the threshold public key
/// - the threshold public key
pub async fn threshold_sign_round2(
    participant: u16,
    files: String,
    url: String,
    pallet: String,
    call_name: String,
    call_data: String,
    context: String,
) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let signing_commitments_string = read_to_string(file_paths.signing_commitments())
        .await
        .map_err(|e| CliError(format!("Failed to read signing commitments file: {}", e)))?;
    
    let signing_commitments_bytes: Vec<Vec<u8>> = serde_json::from_str(&signing_commitments_string)
        .map_err(|e| CliError(format!("Failed to deserialize signing commitments: {}", e)))?;
    
    let signing_commitments: Vec<SigningCommitments> = signing_commitments_bytes
        .iter()
        .map(|sc| SigningCommitments::from_bytes(sc)
            .map_err(|e| CliError(format!("Failed to parse SigningCommitments: {:?}", e))))
        .collect::<Result<_, _>>()?;

    let signing_nonces_string = read_to_string(file_paths.signing_nonce(participant))
        .await
        .map_err(|e| CliError(format!("Failed to read signing nonce file for participant {}: {}", participant, e)))?;
    
    let signing_nonces_bytes: Vec<u8> = serde_json::from_str(&signing_nonces_string)
        .map_err(|e| CliError(format!("Failed to deserialize signing nonces for participant {}: {}", participant, e)))?;
    
    let signing_nonces = SigningNonces::from_bytes(&signing_nonces_bytes)
        .map_err(|e| CliError(format!("Failed to parse signing nonces for participant {}: {:?}", participant, e)))?;

    let signing_share_string = read_to_string(file_paths.signing_share(participant))
        .await
        .map_err(|e| CliError(format!("Failed to read signing share file for participant {}: {}", participant, e)))?;
    
    let signing_share_bytes: Vec<u8> = serde_json::from_str(&signing_share_string)
        .map_err(|e| CliError(format!("Failed to deserialize signing share for participant {}: {}", participant, e)))?;
    
    let signing_share = SigningKeypair::from_bytes(&signing_share_bytes)
        .map_err(|e| CliError(format!("Failed to parse signing share for participant {}: {}", participant, e)))?;

    let output_string = read_to_string(file_paths.generation_output(participant))
        .await
        .map_err(|e| CliError(format!("Failed to read generation output file for participant {}: {}", participant, e)))?;
    
    let output_bytes: Vec<u8> = serde_json::from_str(&output_string)
        .map_err(|e| CliError(format!("Failed to deserialize generation output for participant {}: {}", participant, e)))?;
    
    let generation_output = SPPOutputMessage::from_bytes(&output_bytes)
        .map_err(|e| CliError(format!("Failed to parse generation output for participant {}: {:?}", participant, e)))?;

    let threshold_public_key_string = read_to_string(file_paths.threshold_public_key())
        .await
        .map_err(|e| CliError(format!("Failed to read threshold public key file: {}", e)))?;
    
    let account_id = AccountId32::from_str(&threshold_public_key_string)
        .map_err(|e| CliError(format!("Failed to parse threshold public key: {}", e)))?;

    let client = OnlineClient::<PolkadotConfig>::from_url(&url)
        .await
        .map_err(|e| CliError(format!("Failed to connect to URL {}: {}", url, e)))?;
    
    let rpc_client = RpcClient::from_url(&url)
        .await
        .map_err(|e| CliError(format!("Failed to create RPC client from URL {}: {}", url, e)))?;
    
    let legacy_rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);

    let value = scale_value::stringify::from_str(&call_data).0.unwrap();
    let value_as_composite = value_into_composite(value);

    let call = tx::dynamic(&pallet, &call_name, value_as_composite);

    let nonce = legacy_rpc.system_account_next_index(&account_id)
        .await
        .map_err(|e| CliError(format!("Failed to fetch nonce for account {}: {}", account_id, e)))?;
    
    let params = PolkadotExtrinsicParamsBuilder::new().nonce(nonce).build();
    let partial_tx = client.tx().create_partial_signed_offline(&call, params)
        .map_err(|e| CliError(format!("Failed to create partial signed transaction: {}", e)))?;
    
    let payload = partial_tx.signer_payload().to_vec();

    let signing_package = signing_share.sign(
        context.as_bytes().to_vec(),
        payload,
        generation_output.spp_output(),
        signing_commitments,
        &signing_nonces,
    )
    .map_err(|e| CliError(format!("Failed to create signing package: {:?}", e)))?;

    let mut signing_packages_vec: Vec<Vec<u8>> = if tokio::fs::metadata(file_paths.signing_packages()).await.is_ok() {
        let existing_content = read_to_string(file_paths.signing_packages())
            .await
            .map_err(|e| CliError(format!("Failed to read existing signing packages file: {}", e)))?;
        serde_json::from_str(&existing_content)
            .map_err(|e| CliError(format!("Failed to deserialize existing signing packages: {}", e)))?
    } else {
        Vec::new()
    };
    signing_packages_vec.push(signing_package.to_bytes());

    let signing_package_json = serde_json::to_string(&signing_packages_vec)
        .map_err(|e| CliError(format!("Failed to serialize signing packages: {}", e)))?;

    let mut signing_package_file = File::create(file_paths.signing_packages())
        .await
        .map_err(|e| CliError(format!("Failed to create signing packages file: {}", e)))?;
    
    signing_package_file
        .write_all(signing_package_json.as_bytes())
        .await
        .map_err(|e| CliError(format!("Failed to write signing packages to file: {}", e)))?;

    let extrinsic_info = serde_json::json!({
        "url": url,
        "pallet": pallet,
        "call_name": call_name,
        "call_data": call_data,
    });

    let extrinsic_args_string = serde_json::to_string(&extrinsic_info)
        .map_err(|e| CliError(format!("Failed to serialize extrinsic info: {}", e)))?;
    
    let mut extrinsic_args_file = File::create(file_paths.extrinsic_info())
        .await
        .map_err(|e| CliError(format!("Failed to create extrinsic info file: {}", e)))?;
    
    extrinsic_args_file
        .write_all(extrinsic_args_string.as_bytes())
        .await
        .map_err(|e| CliError(format!("Failed to write extrinsic info to file: {}", e)))?;

    println!("Round 2 of threshold signing was completed successfully!");
    println!("Signing package was written to: {:?}", file_paths.signing_packages());
    println!(
        "Extrinsic info was written to: {:?}",
        file_paths.extrinsic_info()
    );

    Ok(())
}


