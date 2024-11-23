use std::str::FromStr;
use crate::{cli::{commands::value_into_composite, CliError}, files::FilePaths};
use schnorrkel::Signature;
use subxt::{
    tx,
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    config::polkadot::PolkadotExtrinsicParamsBuilder,
    utils::{AccountId32, MultiSignature},
    OnlineClient, PolkadotConfig,
};
use tokio::fs::read_to_string;

/// Submits the threshold extrinsic to the provided url
pub async fn submit_threshold_extrinsic(
    files: String,
) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let threshold_public_key_string = read_to_string(file_paths.threshold_public_key())
        .await
        .map_err(|e| CliError(format!("Failed to read threshold public key file: {}", e)))?;
    
    let account_id = AccountId32::from_str(&serde_json::from_str::<String>(&threshold_public_key_string)
        .map_err(|e| CliError(format!("Failed to parse threshold public key: {}", e)))?)
        .map_err(|e| CliError(format!("Invalid threshold public key format: {}", e)))?;

    let signature_string = read_to_string(file_paths.threshold_signature())
        .await
        .map_err(|e| CliError(format!("Failed to read threshold signature file: {}", e)))?;
    
    let signature_bytes: Vec<u8> = serde_json::from_str(&signature_string)
        .map_err(|e| CliError(format!("Failed to deserialize threshold signature: {}", e)))?;
    
    let group_signature = Signature::from_bytes(&signature_bytes)
        .map_err(|e| CliError(format!("Failed to parse threshold signature: {}", e)))?;

    let extrinsic_info_string = read_to_string(file_paths.extrinsic_info())
        .await
        .map_err(|e| CliError(format!("Failed to read extrinsic info file: {}", e)))?;
    
    let extrinsic_info: serde_json::Value = serde_json::from_str(&extrinsic_info_string)
        .map_err(|e| CliError(format!("Failed to parse extrinsic info: {}", e)))?;

    let url = extrinsic_info
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError("Missing 'url' in extrinsic info".to_string()))?;

    let pallet = extrinsic_info
        .get("pallet")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError("Missing 'pallet' in extrinsic info".to_string()))?;

    let call_name = extrinsic_info
        .get("call_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError("Missing 'call_name' in extrinsic info".to_string()))?;

    let call_data = extrinsic_info
        .get("call_data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError("Missing 'call_data' in extrinsic info".to_string()))?;

    let value = scale_value::stringify::from_str(&call_data).0
        .map_err(|_| CliError(format!("Failed to parse call data: {}", call_data)))?;
    let value_as_composite = value_into_composite(value);

    let call = tx::dynamic(pallet, call_name, value_as_composite);

    let client = OnlineClient::<PolkadotConfig>::from_url(&url)
        .await
        .map_err(|e| CliError(format!("Failed to connect to URL {}: {}", url, e)))?;

    let rpc_client = RpcClient::from_url(&url)
        .await
        .map_err(|e| CliError(format!("Failed to create RPC client from URL {}: {}", url, e)))?;
    
    let legacy_rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);

    let nonce = legacy_rpc.system_account_next_index(&account_id)
        .await
        .map_err(|e| CliError(format!("Failed to fetch nonce for account {}: {}", account_id, e)))?;

    let params = PolkadotExtrinsicParamsBuilder::new().nonce(nonce).build();

    let partial_extrinsic = client.tx().create_partial_signed_offline(&call, params)
        .map_err(|e| CliError(format!("Failed to create partial signed extrinsic: {}", e)))?;

    let signature = subxt_signer::sr25519::Signature(group_signature.to_bytes());

    let extrinsic = partial_extrinsic
        .sign_with_address_and_signature(
            &account_id.into(),
            &MultiSignature::Sr25519(signature.0),
        );

    extrinsic
        .submit()
        .await
        .map_err(|e| CliError(format!("Failed to submit extrinsic: {}", e)))?;

    println!("Submission of threshold extrinsic was completed successfully!");

    Ok(())
}
