
                   use serde::{Serialize, Deserialize};
                   #[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct PaperAnnotationsItemAnnotationsSubject {
    #[serde(rename = "download_checksum_sha256")]
    pub download_checksum_sha_256: Option<String>,
    pub download_url: String,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct PaperAnnotationsItemAnnotations {
    pub subject: Option<PaperAnnotationsItemAnnotationsSubject>,
}
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct PaperAnnotations {
    pub annotations: Vec<PaperAnnotationsItemAnnotations>,
    pub version: String,
}
