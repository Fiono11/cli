use crate::{cli::errors::CliError, files::FilePaths};
use schnorrkel::{
    PublicKey, Signature,
};
use serde_json::from_str;
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    config::polkadot::PolkadotExtrinsicParamsBuilder,
    dynamic::Value,
    utils::{AccountId32, MultiAddress},
    OnlineClient, PolkadotConfig,
};
use tokio::fs::read_to_string;

pub async fn submit_transaction(
    files: String,
    url: String,
    pallet: String,
    function: String,
    call_args: String,
) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    // Read threshold_public_key
    let threshold_public_key_string = read_to_string(file_paths.threshold_public_key()).await?;
    let threshold_public_key_bytes: Vec<u8> = from_str(&threshold_public_key_string)?;
    let threshold_public_key = PublicKey::from_bytes(&threshold_public_key_bytes)?;

    let account_id = AccountId32(threshold_public_key.to_bytes());
    println!("pk: {:?}", account_id.to_string());

    // Read group_signature
    let signature_string = read_to_string(file_paths.signature()).await?;
    let signature_bytes: Vec<u8> = from_str(&signature_string)?;
    let group_signature = Signature::from_bytes(&signature_bytes)?;

    let client = OnlineClient::<PolkadotConfig>::new().await?;
    let rpc_client = RpcClient::from_url(url).await?;
    let legacy_rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);
    let nonce = legacy_rpc.system_account_next_index(&account_id).await?;

    // Parse call arguments from JSON string
    let args: Vec<Value> = serde_json::from_str(&call_args)?;

    // Create the call with the given pallet, function, and arguments
    let call = subxt::dynamic::tx(pallet, function, args);
    let params = PolkadotExtrinsicParamsBuilder::new().nonce(nonce).build();
    let partial_tx = client.tx().create_partial_signed_offline(&call, params)?;

    let signature = subxt_signer::sr25519::Signature(group_signature.to_bytes());
    let public_key: MultiAddress<AccountId32, _> =
        subxt_signer::sr25519::PublicKey(threshold_public_key.to_bytes()).into();

    let tx = partial_tx.sign_with_address_and_signature(&public_key, &signature.into());
    tx.submit().await?;

    Ok(())
}