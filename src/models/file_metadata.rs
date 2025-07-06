use crate::errors::ServiceError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tokio::sync::oneshot;

/// Metadata for files stored in IPFS
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub cid: String,
    pub name: String,
    pub size: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: DateTime<Utc>,
    pub user_id: i32,
}

/// Upload status response
#[derive(Serialize, Deserialize, Clone)]
pub struct UploadStatus {
    pub task_id: String,
    // "pending", "completed", "failed"
    pub status: String,
    pub cid: Option<String>,
    pub error: Option<String>,
    // Percentage complete (0.0 to 100.0)
    pub progress: Option<f64>,
    pub started_at: DateTime<Utc>,
}

/// Task tracking information stored in memory and database
pub struct TaskInfo {
    pub status: UploadStatus,
    pub tx: Option<oneshot::Sender<Result<FileMetadata, ServiceError>>>,
}

/// Research paper metadata extracted from papers and linked to DIDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchPaperMetadata {
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
    pub doi: Option<String>,
    pub publication_date: Option<String>,
    pub journal: Option<String>,
    pub keywords: Vec<String>,
    pub cid: String,
    pub did: String,
    pub biological_entities: Vec<BiologicalEntityReference>,
    pub knowledge_graph_cid: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Reference to a biological entity identified in a research paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiologicalEntityReference {
    pub entity_type: String,
    pub name: String,
    pub identifier: Option<String>,
    pub source: Option<String>,
}
