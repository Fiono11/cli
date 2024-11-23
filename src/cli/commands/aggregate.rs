use schnorrkel::{olaf::multisig::{aggregate, SigningPackage}, Signature};
use tokio::{fs::{read_to_string, File}, io::AsyncWriteExt};
use crate::{cli::CliError, files::FilePaths};

/// Aggregates at least t partial signatures into one threshold signature
pub async fn aggregate_threshold_signature(files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let signing_packages_string = read_to_string(file_paths.signing_packages())
        .await
        .map_err(|e| CliError(format!("Failed to read signing packages file: {}", e)))?;
    
    let signing_packages_bytes: Vec<Vec<u8>> = serde_json::from_str(&signing_packages_string)
        .map_err(|e| CliError(format!("Failed to deserialize signing packages: {}", e)))?;
    
    let signing_packages: Vec<SigningPackage> = signing_packages_bytes
        .iter()
        .map(|sp| SigningPackage::from_bytes(sp)
            .map_err(|e| CliError(format!("Failed to parse SigningPackage: {:?}", e))))
        .collect::<Result<_, _>>()?;
    
    let group_signature: Signature = aggregate(&signing_packages)
        .map_err(|e| CliError(format!("Failed to aggregate threshold signature: {:?}", e)))?;
    
    let signature_json = serde_json::to_string(&group_signature.to_bytes().to_vec())
        .map_err(|e| CliError(format!("Failed to serialize threshold signature: {}", e)))?;
    
    let mut signature_file = File::create(file_paths.threshold_signature())
        .await
        .map_err(|e| CliError(format!("Failed to create threshold signature file: {}", e)))?;
    
    signature_file
        .write_all(signature_json.as_bytes())
        .await
        .map_err(|e| CliError(format!("Failed to write threshold signature to file: {}", e)))?;

    println!("Aggregation of threshold signature was completed successfully!");
    println!(
        "The threshold signature was written to: {:?}",
        file_paths.threshold_signature()
    );

    Ok(())
}
