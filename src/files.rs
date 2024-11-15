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

    pub fn contributor_secret_key(&self) -> PathBuf {
        self.base.join("contributor_secret_key.json")
    }

    pub fn recipients(&self) -> PathBuf {
        self.base.join("recipients.json")
    }

    pub fn all_messages(&self) -> PathBuf {
        self.base.join("all_messages.json")
    }

    pub fn spp_output(&self) -> PathBuf {
        self.base.join("spp_output.json")
    }

    pub fn signing_share(&self) -> PathBuf {
        self.base.join("signing_share.json")
    }

    pub fn threshold_public_key(&self) -> PathBuf {
        self.base.join("threshold_public_key.json")
    }

    pub fn signing_nonces(&self) -> PathBuf {
        self.base.join("signing_nonces.json")
    }

    pub fn signing_commitments(&self) -> PathBuf {
        self.base.join("signing_commitments.json")
    }

    pub fn signing_packages(&self) -> PathBuf {
        self.base.join("signing_packages.json")
    }

    pub fn signature(&self) -> PathBuf {
        self.base.join("signature.json")
    }

    pub fn extrinsic_info(&self) -> PathBuf {
        self.base.join("extrinsic_info.json")
    }
}
