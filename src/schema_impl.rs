use std::collections::HashMap;
use std::path::PathBuf;

use crate::schema::*;

pub type PapersByHash = HashMap<String, PaperAnnotationsItemAnnotations>;

impl PaperAnnotationsItemAnnotationsSubject {
    /// Returns the download target path for the subject file.
    pub fn download_target_path(
        &self,
        data_dir: &str,
        override_hash: Option<&String>,
    ) -> Option<PathBuf> {
        let data_dir_path: PathBuf = data_dir.into();
        let hash = override_hash.or(self.download_checksum_sha_3_256.as_ref());
        if hash.is_none() {
            return None;
        }

        let hash = hash.unwrap();

        Some(
            data_dir_path
                .join("./papers")
                .join(&format!("./{}.pdf", hash)),
        )
    }

    /// Checks if subject file is present at download target path
    pub fn is_downloaded(&self, data_dir: &str, override_hash: Option<&String>) -> bool {
        let target_path = self.download_target_path(data_dir, override_hash);
        if target_path.is_none() {
            return false;
        }

        target_path.unwrap().exists()
    }

    /// Checks if hash matches the expected hash for the download
    ///
    /// Is always true if no `download_checksum_sha_3_256` is set.
    pub fn verify_download_checksum(&self, hash: &str) -> bool {
        match &self.download_checksum_sha_3_256 {
            None => true,
            Some(expected_hash) => expected_hash == hash,
        }
    }
}

impl PaperAnnotations {
    /// Create a lookup map hash->papers
    pub fn papers_by_hash(&self) -> HashMap<String, PaperAnnotationsItemAnnotations> {
        self.annotations
            .iter()
            .filter_map(|item| match &item.subject.download_checksum_sha_3_256 {
                None => None,
                Some(hash) => Some((hash.to_owned(), item.clone())),
            })
            .collect()
    }
}
