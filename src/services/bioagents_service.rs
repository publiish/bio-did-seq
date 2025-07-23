use crate::errors::AppError;
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Health status of the BioAgents system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub agents_online: i32,
    pub service_status: String,
    pub last_updated: String,
}

/// BioAgents service for interacting with BioAgents API
pub struct BioAgentsService {
    client: Client,
    api_url: String,
}

/// Request body for processing a paper through BioAgents
#[derive(Debug, Serialize)]
pub struct ProcessPaperRequest {
    pub file_cid: String,
    pub title: String,
    pub authors: Vec<String>,
    pub doi: Option<String>,
    pub extract_metadata: bool,
    pub generate_knowledge_graph: bool,
}

/// Response from BioAgents paper processing
#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessPaperResponse {
    pub task_id: String,
    pub status: String,
}

/// Metadata extracted by BioAgents
#[derive(Debug, Deserialize, Serialize)]
pub struct ExtractedMetadata {
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
    pub keywords: Vec<String>,
    pub publication_date: Option<String>,
    pub journal: Option<String>,
    pub doi: Option<String>,
    pub biological_entities: Vec<BiologicalEntity>,
}

/// Biological entity identified in the paper
#[derive(Debug, Deserialize, Serialize)]
pub struct BiologicalEntity {
    // e.g., "gene", "protein", "disease", etc.
    pub entity_type: String,
    pub name: String,
    // e.g., gene ID, protein ID
    pub identifier: Option<String>,
    // e.g., "UniProt", "NCBI", etc.
    pub source: Option<String>,
    pub mentions: Vec<EntityMention>,
}

/// Mention of a biological entity in the paper
#[derive(Debug, Deserialize, Serialize)]
pub struct EntityMention {
    pub text: String,
    pub start_pos: Option<i32>,
    pub end_pos: Option<i32>,
    // e.g., "abstract", "introduction", etc.
    pub section: Option<String>,
}

/// Status of a BioAgents task
#[derive(Debug, Deserialize, Serialize)]
pub struct TaskStatus {
    pub task_id: String,
    // "pending", "processing", "completed", "failed"
    pub status: String,
    // 0.0 to 1.0
    pub progress: f32,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl BioAgentsService {
    /// Create a new BioAgents service
    pub fn new(api_url: &str) -> Self {
        // Create HTTP client with appropriate timeouts
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_url: api_url.to_string(),
        }
    }

    /// Process a paper through BioAgents for metadata extraction and knowledge graph generation
    pub async fn process_paper(
        &self,
        request: ProcessPaperRequest,
    ) -> Result<ProcessPaperResponse, AppError> {
        let url = format!("{}/api/process-paper", self.api_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to BioAgents: {}", e);
                AppError::ExternalServiceError("BioAgents service unavailable".to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("BioAgents API error ({}): {}", status, error_text);
            return Err(AppError::ExternalServiceError(format!(
                "BioAgents API error: {}",
                error_text
            )));
        }

        let process_response: ProcessPaperResponse = response.json().await.map_err(|e| {
            error!("Failed to parse BioAgents response: {}", e);
            AppError::DeserializationError
        })?;

        info!(
            "Paper processing task started with ID: {}",
            process_response.task_id
        );

        Ok(process_response)
    }

    /// Check the status of a paper processing task
    pub async fn check_task_status(&self, task_id: &str) -> Result<TaskStatus, AppError> {
        let url = format!("{}/api/task-status/{}", self.api_url, task_id);

        let response = self.client.get(&url).send().await.map_err(|e| {
            error!("Failed to check task status: {}", e);
            AppError::ExternalServiceError("BioAgents service unavailable".to_string())
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("BioAgents API error ({}): {}", status, error_text);
            return Err(AppError::ExternalServiceError(format!(
                "BioAgents API error: {}",
                error_text
            )));
        }

        let task_status: TaskStatus = response.json().await.map_err(|e| {
            error!("Failed to parse task status response: {}", e);
            AppError::DeserializationError
        })?;

        Ok(task_status)
    }

    /// Get extracted metadata for a completed task
    pub async fn get_extracted_metadata(
        &self,
        task_id: &str,
    ) -> Result<ExtractedMetadata, AppError> {
        let url = format!("{}/api/metadata/{}", self.api_url, task_id);

        let response = self.client.get(&url).send().await.map_err(|e| {
            error!("Failed to get extracted metadata: {}", e);
            AppError::ExternalServiceError("BioAgents service unavailable".to_string())
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("BioAgents API error ({}): {}", status, error_text);
            return Err(AppError::ExternalServiceError(format!(
                "BioAgents API error: {}",
                error_text
            )));
        }

        let metadata: ExtractedMetadata = response.json().await.map_err(|e| {
            error!("Failed to parse metadata response: {}", e);
            AppError::DeserializationError
        })?;

        Ok(metadata)
    }

    /// Search for related biological entities
    pub async fn search_related_entities(
        &self,
        query: &str,
    ) -> Result<Vec<BiologicalEntity>, AppError> {
        let url = format!("{}/api/search", self.api_url);

        let response = self
            .client
            .get(&url)
            .query(&[("q", query)])
            .send()
            .await
            .map_err(|e| {
                error!("Failed to search related entities: {}", e);
                AppError::ExternalServiceError("BioAgents service unavailable".to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("BioAgents API error ({}): {}", status, error_text);
            return Err(AppError::ExternalServiceError(format!(
                "BioAgents API error: {}",
                error_text
            )));
        }

        let entities: Vec<BiologicalEntity> = response.json().await.map_err(|e| {
            error!("Failed to parse search response: {}", e);
            AppError::DeserializationError
        })?;

        Ok(entities)
    }

    /// Generate a knowledge graph from a research paper
    pub async fn generate_knowledge_graph(&self, cid: &str) -> Result<String, AppError> {
        let url = format!("{}/api/knowledge-graph", self.api_url);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "cid": cid }))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to generate knowledge graph: {}", e);
                AppError::ExternalServiceError("BioAgents service unavailable".to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("BioAgents API error ({}): {}", status, error_text);
            return Err(AppError::ExternalServiceError(format!(
                "BioAgents API error: {}",
                error_text
            )));
        }

        // The response contains a knowledge graph in RDF format
        let knowledge_graph = response.text().await.map_err(|e| {
            error!("Failed to read knowledge graph response: {}", e);
            AppError::DeserializationError
        })?;

        Ok(knowledge_graph)
    }

    /// Query the BioAgents with a natural language question
    pub async fn query_agents(&self, query: &str) -> Result<(String, Vec<String>), AppError> {
        info!("Querying BioAgents: {}", query);

        // Create the request body
        let body = serde_json::json!({
            "query": query,
        });

        // Send the request to BioAgents
        let response = self
            .client
            .post(&format!("{}/query", self.api_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to query BioAgents: {}", e);
                AppError::ExternalServiceError("BioAgents service unavailable".to_string())
            })?;

        // Check if the request was successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("BioAgents API error ({}): {}", status, error_text);
            return Err(AppError::ExternalServiceError(format!(
                "BioAgents API error: {}",
                error_text
            )));
        }

        // Parse the response
        let response_data: serde_json::Value = response.json().await.map_err(|e| {
            error!("Failed to parse BioAgents response: {}", e);
            AppError::DeserializationError
        })?;

        // Extract the answer and sources
        let answer = response_data["answer"]
            .as_str()
            .ok_or_else(|| AppError::DeserializationError)?
            .to_string();

        let sources = response_data["sources"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        info!(
            "BioAgents query successful, answer length: {}",
            answer.len()
        );

        Ok((answer, sources))
    }

    /// Add knowledge to the BioAgents system
    pub async fn add_knowledge(
        &self,
        title: &str,
        content: &str,
        keywords: &[String],
    ) -> Result<String, AppError> {
        info!("Adding knowledge to BioAgents: {}", title);

        // Create the request body
        let body = serde_json::json!({
            "title": title,
            "content": content,
            "keywords": keywords,
        });

        // Send the request to BioAgents
        let response = self
            .client
            .post(&format!("{}/knowledge", self.api_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to add knowledge to BioAgents: {}", e);
                AppError::ExternalServiceError("BioAgents service unavailable".to_string())
            })?;

        // Check if the request was successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("BioAgents API error ({}): {}", status, error_text);
            return Err(AppError::ExternalServiceError(format!(
                "BioAgents API error: {}",
                error_text
            )));
        }

        // Parse the response
        let response_data: serde_json::Value = response.json().await.map_err(|e| {
            error!("Failed to parse BioAgents response: {}", e);
            AppError::DeserializationError
        })?;

        // Extract the knowledge ID
        let id = response_data["id"]
            .as_str()
            .ok_or_else(|| AppError::DeserializationError)?
            .to_string();

        info!("Knowledge added to BioAgents successfully, ID: {}", id);

        Ok(id)
    }

    /// Check the health of the BioAgents system
    pub async fn check_health(&self) -> Result<HealthStatus, AppError> {
        info!("Checking BioAgents health status");

        // Send a health check request to BioAgents
        let response = self
            .client
            .get(&format!("{}/health", self.api_url))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to check BioAgents health: {}", e);
                AppError::ExternalServiceError("BioAgents service unavailable".to_string())
            })?;

        // Check if the request was successful
        if !response.status().is_success() {
            // Return a degraded status rather than an error
            return Ok(HealthStatus {
                agents_online: 0,
                service_status: "offline".to_string(),
                last_updated: chrono::Utc::now().to_rfc3339(),
            });
        }

        // Parse the response
        let response_data: serde_json::Value = response.json().await.map_err(|e| {
            error!("Failed to parse BioAgents health response: {}", e);
            AppError::DeserializationError
        })?;

        // Extract the health status
        let agents_online = response_data["agents_online"].as_i64().unwrap_or(0) as i32;

        let service_status = response_data["status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        // Get last updated timestamp or use current time
        let last_updated = match response_data["last_updated"].as_str() {
            Some(timestamp) => timestamp.to_string(),
            None => chrono::Utc::now().to_rfc3339(),
        };

        info!(
            "BioAgents health check: {} agents online, status: {}",
            agents_online, service_status
        );

        Ok(HealthStatus {
            agents_online,
            service_status,
            last_updated,
        })
    }
}
