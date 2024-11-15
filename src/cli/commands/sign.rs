use std::str::FromStr;
use crate::{cli::errors::CliError, files::FilePaths};
use schnorrkel::olaf::{
    multisig::{SigningCommitments, SigningNonces},
    simplpedpop::SPPOutputMessage,
    SigningKeypair,
};
use serde_json::from_str;
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient}, config::polkadot::PolkadotExtrinsicParamsBuilder, tx, utils::AccountId32, OnlineClient, PolkadotConfig
};
use tokio::{
    fs::{read_to_string, File},
    io::AsyncWriteExt,
};
use scale_value::{Composite, ValueDef};

// Generate an interface that we can use from the node's metadata.
#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}

pub async fn threshold_sign_round1(files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let signing_share_string = read_to_string(file_paths.signing_share()).await?;
    let signing_share_bytes: Vec<u8> = from_str(&signing_share_string)?;
    let signing_share: SigningKeypair = SigningKeypair::from_bytes(&signing_share_bytes)?;

    let (signing_nonces, signing_commitments) = signing_share.commit();

    let signing_nonces_json = serde_json::to_string_pretty(&signing_nonces.to_bytes().to_vec())?;
    let mut signing_nonces_file = File::create(file_paths.signing_nonces()).await?;
    signing_nonces_file
        .write_all(signing_nonces_json.as_bytes())
        .await?;

    let signing_commitments_vec = vec![signing_commitments.to_bytes().to_vec()];
    let signing_commitments_json = serde_json::to_string_pretty(&signing_commitments_vec)?;

    let mut signing_commitments_file = File::create(file_paths.signing_commitments()).await?;
    signing_commitments_file
        .write_all(signing_commitments_json.as_bytes())
        .await?;

    println!("Round 1 of threshold signing was completed successfully.");
    println!("Signing nonce was written to: {:?}", file_paths.signing_nonces());
    println!(
        "Signing commitment was written to: {:?}",
        file_paths.signing_commitments()
    );        

    Ok(())
}

pub async fn threshold_sign_round2(
    files: String,
    url: String,
    pallet: String,
    call_name: String,
    call_data: String,
    context: String,
) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let signing_commitments_string = read_to_string(file_paths.signing_commitments()).await?;
    let signing_commitments_bytes: Vec<Vec<u8>> = from_str(&signing_commitments_string)?;
    let signing_commitments: Vec<SigningCommitments> = signing_commitments_bytes
        .iter()
        .map(|sc| SigningCommitments::from_bytes(sc))
        .collect::<Result<_, _>>()?;

    let signing_nonces_string = read_to_string(file_paths.signing_nonces()).await?;
    let signing_nonces_bytes: Vec<u8> = from_str(&signing_nonces_string)?;
    let signing_nonces = SigningNonces::from_bytes(&signing_nonces_bytes)?;

    let signing_share_string = read_to_string(file_paths.signing_share()).await?;
    let signing_share_bytes: Vec<u8> = from_str(&signing_share_string)?;
    let signing_share = SigningKeypair::from_bytes(&signing_share_bytes)?;

    let output_string = read_to_string(file_paths.spp_output()).await?;
    let output_bytes: Vec<u8> = from_str(&output_string)?;
    let spp_output = SPPOutputMessage::from_bytes(&output_bytes)?;

    //let threshold_public_key_string = read_to_string(file_paths.threshold_public_key()).await?;
    //let threshold_public_key_bytes: Vec<u8> = from_str(&threshold_public_key_string)?;
    //let threshold_public_key = PublicKey::from_bytes(&threshold_public_key_bytes)?;

    let threshold_public_key_string = read_to_string(file_paths.threshold_public_key()).await?;
	let account_id = AccountId32::from_str(from_str(&threshold_public_key_string)?)
        .map_err(|e| CliError::Custom(e.to_string()))?;

    let client: OnlineClient<PolkadotConfig> = OnlineClient::<PolkadotConfig>::new().await?;
    let rpc_client = RpcClient::from_url(url.clone()).await?;
    let legacy_rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);

    //let encoded_call_data: Value = Value::from_bytes(call_data.clone().as_bytes());

    // parse scale_value from trailing arguments and try to create an unsigned extrinsic with it:
	let value = scale_value::stringify::from_str(&call_data).0.unwrap();
	let value_as_composite = value_into_composite(value);

    // Create the call with the given pallet, function, and arguments
    let call = tx::dynamic(&pallet, &call_name, value_as_composite);

    //let account_id = AccountId32(threshold_public_key.to_bytes());
    let nonce = legacy_rpc.system_account_next_index(&account_id).await?;
    let params = PolkadotExtrinsicParamsBuilder::new().nonce(nonce).build();
    let partial_tx = client.tx().create_partial_signed_offline(&call, params)?;
    let payload = partial_tx.signer_payload().to_vec();

    let signing_package = signing_share.sign(
        context.as_bytes().to_vec(),
        payload,
        spp_output.spp_output(),
        signing_commitments,
        &signing_nonces,
    )?;

    let signing_packages_vec = vec![signing_package.to_bytes()];
    let signing_package_json = serde_json::to_string_pretty(&signing_packages_vec)?;

    let mut signing_package_file = File::create(file_paths.signing_packages()).await?;
    signing_package_file
        .write_all(signing_package_json.as_bytes())
        .await?;

    // Save extrinsic arguments to a file
    let extrinsic_info = serde_json::json!({
        "url": url,
        "pallet": pallet,
        "call_name": call_name,
        "call_data": call_data,
    });
    let extrinsic_args_string = serde_json::to_string_pretty(&extrinsic_info)?;
    let mut extrinsic_args_file = File::create(file_paths.extrinsic_info()).await?;
    extrinsic_args_file
        .write_all(extrinsic_args_string.as_bytes())
        .await?;

    println!("Round 2 of threshold signing was completed successfully.");
    println!("Signing package was written to: {:?}", file_paths.signing_packages());
    println!(
        "Extrinsic info was written to: {:?}",
        file_paths.extrinsic_info()
    );

    Ok(())
}

/// composites stay composites, all other types are converted into a 1-fielded unnamed composite
pub(crate) fn value_into_composite(value: scale_value::Value) -> scale_value::Composite<()> {
	match value.value {
		ValueDef::Composite(composite) => composite,
		_ => Composite::Unnamed(vec![value]),
	}
}