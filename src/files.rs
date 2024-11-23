use std::path::{Path, PathBuf};

pub struct FilePaths {
    base: PathBuf,
}

impl FilePaths {
    pub fn new(base: impl AsRef<Path>) -> Self {
        Self {
            base: base.as_ref().to_path_buf(),
        }
    }

    pub fn contributor_secret_key(&self, participant: u16) -> PathBuf {
        self.base.join(format!("contributor_secret_key{}.json", participant))
    }

    pub fn recipients(&self) -> PathBuf {
        self.base.join("recipients.json")
    }

    pub fn all_messages(&self) -> PathBuf {
        self.base.join("all_messages.json")
    }

    pub fn generation_output(&self, participant: u16) -> PathBuf {
        self.base.join(format!("generation_output{}.json", participant))
    }

    pub fn signing_share(&self, participant: u16) -> PathBuf {
        self.base.join(format!("signing_share{}.json", participant))
    }

    pub fn threshold_public_key(&self) -> PathBuf {
        self.base.join("threshold_public_key.json")
    }

    pub fn signing_nonce(&self, participant: u16) -> PathBuf {
        self.base.join(format!("signing_nonce{}.json", participant))
    }

    pub fn signing_commitments(&self) -> PathBuf {
        self.base.join("signing_commitments.json")
    }

    pub fn signing_packages(&self) -> PathBuf {
        self.base.join("signing_packages.json")
    }

    pub fn threshold_signature(&self) -> PathBuf {
        self.base.join("threshold_signature.json")
    }

    pub fn extrinsic_info(&self) -> PathBuf {
        self.base.join("extrinsic_info.json")
    }
}
