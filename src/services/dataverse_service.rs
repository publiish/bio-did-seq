use crate::errors::AppError;
use serde::{Serialize, Deserialize};
use log::{info, error};
use std::path::Path;
use reqwest::multipart;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::time::Duration;
use serde_json::Value;

/// Dataset metadata structure
#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub title: String,
    pub description: String,
    pub authors: Vec<String>,
    pub keywords: Vec<String>,
}

/// Dataset response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetResponse {
    pub id: String,
    pub persistent_id: String,
    pub title: String,
    pub description: String,
}

/// Service for interacting with the Dataverse API
pub struct DataverseService {
    client: reqwest::Client,
    api_key: String,
    api_url: String,
}

impl DataverseService {
    /// Create a new DataverseService instance
    pub fn new(api_url: &str, api_key: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            client,
            api_key: api_key.to_string(),
            api_url: api_url.to_string(),
        }
    }
    
    /// Create a new dataset in Dataverse
    pub async fn create_dataset(
        &self,
        title: &str,
        description: &str,
        authors: &[String],
        keywords: &[String],
    ) -> Result<DatasetResponse, AppError> {
        info!("Creating dataset in Dataverse: {}", title);
        
        // Prepare dataset metadata in Dataverse format
        let metadata = self.build_dataset_metadata(title, description, authors, keywords);
        
        // Create the request
        let url = format!("{}/api/datasets", self.api_url);
        
        let response = self.client.post(&url)
            .header("X-Dataverse-key", &self.api_key)
            .json(&metadata)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to create dataset in Dataverse: {}", e);
                AppError::ExternalServiceError(format!("Dataverse request failed: {}", e))
            })?;
        
        // Check if the request was successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Dataverse API error ({}): {}", status, error_text);
            return Err(AppError::DataverseApiError(format!("API error ({}): {}", status, error_text)));
        }
        
        // Parse the response
        let response_json: Value = response.json().await.map_err(|e| {
            error!("Failed to parse Dataverse response: {}", e);
            AppError::DeserializationError
        })?;
        
        // Extract dataset ID and persistent ID from the response
        let dataset_json = &response_json["data"]["persistentId"];
        let persistent_id = dataset_json.as_str()
            .ok_or_else(|| AppError::DeserializationError)?
            .to_string();
            
        let dataset_id = response_json["data"]["id"]
            .as_i64()
            .map(|id| id.to_string())
            .ok_or_else(|| AppError::DeserializationError)?;
            
        info!("Dataset created in Dataverse with ID: {}, PID: {}", dataset_id, persistent_id);
        
        Ok(DatasetResponse {
            id: dataset_id,
            persistent_id,
            title: title.to_string(),
            description: description.to_string(),
        })
    }
    
    /// Update dataset metadata
    pub async fn update_metadata(
        &self,
        persistent_id: &str,
        title: Option<&str>,
        description: Option<&str>,
        authors: Option<&[String]>,
        keywords: Option<&[String]>,
    ) -> Result<(), AppError> {
        info!("Updating metadata for dataset: {}", persistent_id);
        
        // Get current metadata
        let current = self.get_dataset_metadata(persistent_id).await?;
        
        // Extract current values
        let current_title = current["datasetVersion"]["metadataBlocks"]["citation"]["fields"]
            .as_array()
            .and_then(|fields| fields.iter().find(|f| f["typeName"] == "title"))
            .and_then(|f| f["value"].as_str())
            .unwrap_or("");
            
        let current_description = current["datasetVersion"]["metadataBlocks"]["citation"]["fields"]
            .as_array()
            .and_then(|fields| fields.iter().find(|f| f["typeName"] == "dsDescription"))
            .and_then(|f| f["value"].as_array())
            .and_then(|arr| arr.first())
            .and_then(|desc| desc["dsDescriptionValue"]["value"].as_str())
            .unwrap_or("");
        
        // Build updated metadata
        let metadata = self.build_dataset_metadata(
            title.unwrap_or(current_title),
            description.unwrap_or(current_description),
            authors.unwrap_or(&[]),
            keywords.unwrap_or(&[]),
        );
        
        // Send the request
        let url = format!("{}/api/datasets/:persistentId/?persistentId={}", self.api_url, persistent_id);
        
        let response = self.client.put(&url)
            .header("X-Dataverse-key", &self.api_key)
            .json(&metadata)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to update dataset metadata: {}", e);
                AppError::ExternalServiceError(format!("Dataverse request failed: {}", e))
            })?;
        
        // Check if the request was successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Dataverse API error ({}): {}", status, error_text);
            return Err(AppError::DataverseApiError(format!("API error ({}): {}", status, error_text)));
        }
        
        info!("Dataset metadata updated successfully: {}", persistent_id);
        
        Ok(())
    }
    
    /// Upload a file to Dataverse
    pub async fn upload_file(&self, dataset_id: &str, file_path: &Path, description: &str) -> Result<String, AppError> {
        info!("Uploading file to Dataverse dataset {}: {}", dataset_id, file_path.display());
        
        // Open the file
        let mut file = File::open(file_path).await.map_err(|e| {
            error!("Failed to open file for upload: {}", e);
            AppError::FileError(format!("Failed to open file: {}", e))
        })?;
        
        // Read the file content
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await.map_err(|e| {
            error!("Failed to read file content: {}", e);
            AppError::FileError(format!("Failed to read file: {}", e))
        })?;
        
        // Create file part with the buffer
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file.dat");
            
        let file_part = multipart::Part::bytes(buffer)
            .file_name(file_name.to_string())
            .mime_str("application/octet-stream")
            .map_err(|e| {
                error!("Failed to set MIME type: {}", e);
                AppError::RequestError(format!("Failed to set MIME type: {}", e))
            })?;
        
        // Build the form
        let form = multipart::Form::new()
            .text("description", description.to_string())
            .part("file", file_part);
        
        // Construct the request
        let url = format!("{}/api/datasets/{}/add", self.api_url, dataset_id);
        
        let response = self.client.post(&url)
            .header("X-Dataverse-key", &self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to upload file to Dataverse: {}", e);
                AppError::RequestError(format!("Failed to upload file: {}", e))
            })?;
        
        // Check if the request was successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Dataverse API error ({}): {}", status, error_text);
            return Err(AppError::DataverseApiError(format!("API error ({}): {}", status, error_text)));
        }
        
        // Parse the response
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            error!("Failed to parse Dataverse response: {}", e);
            AppError::DeserializationError
        })?;
        
        // Extract the file ID from the response
        let file_id = response_json["data"]["files"][0]["dataFile"]["id"]
            .as_i64()
            .map(|id| id.to_string())
            .ok_or_else(|| {
                error!("Failed to extract file ID from Dataverse response");
                AppError::DeserializationError
            })?;
        
        info!("File uploaded successfully to dataset {}, file ID: {}", dataset_id, file_id);
        
        Ok(file_id)
    }
    
    /// Publish a dataset in Dataverse
    pub async fn publish_dataset(&self, persistent_id: &str) -> Result<(), AppError> {
        info!("Publishing dataset: {}", persistent_id);
        
        let url = format!("{}/api/datasets/:persistentId/actions/:publish?persistentId={}&type=major", 
            self.api_url, persistent_id);
            
        let response = self.client.post(&url)
            .header("X-Dataverse-key", &self.api_key)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to publish dataset: {}", e);
                AppError::ExternalServiceError(format!("Dataverse request failed: {}", e))
            })?;
            
        // Check if the request was successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Dataverse API error ({}): {}", status, error_text);
            return Err(AppError::DataverseApiError(format!("API error ({}): {}", status, error_text)));
        }
        
        info!("Dataset published successfully: {}", persistent_id);
        
        Ok(())
    }
    
    /// Get dataset metadata
    pub async fn get_dataset_metadata(&self, persistent_id: &str) -> Result<Value, AppError> {
        info!("Getting metadata for dataset: {}", persistent_id);
        
        let url = format!("{}/api/datasets/:persistentId?persistentId={}", self.api_url, persistent_id);
        
        let response = self.client.get(&url)
            .header("X-Dataverse-key", &self.api_key)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to get dataset metadata: {}", e);
                AppError::ExternalServiceError(format!("Dataverse request failed: {}", e))
            })?;
            
        // Check if the request was successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Dataverse API error ({}): {}", status, error_text);
            return Err(AppError::DataverseApiError(format!("API error ({}): {}", status, error_text)));
        }
        
        // Parse the response
        let metadata: Value = response.json().await.map_err(|e| {
            error!("Failed to parse Dataverse response: {}", e);
            AppError::DeserializationError
        })?;
        
        Ok(metadata["data"].clone())
    }
    
    /// Build dataset metadata in Dataverse format
    fn build_dataset_metadata(
        &self,
        title: &str,
        description: &str,
        authors: &[String],
        keywords: &[String],
    ) -> Value {
        // Create author entries
        let author_values = authors.iter().map(|author| {
            serde_json::json!({
                "authorName": { "value": author },
                "authorAffiliation": { "value": "Unknown" }
            })
        }).collect::<Vec<_>>();
        
        // Create keyword entries
        let keyword_values = keywords.iter().map(|keyword| {
            serde_json::json!({
                "keywordValue": { "value": keyword }
            })
        }).collect::<Vec<_>>();
        
        // Create description entry
        let description_value = serde_json::json!({
            "dsDescriptionValue": { "value": description }
        });
        
        // Build the complete metadata object
        serde_json::json!({
            "datasetVersion": {
                "license": { "name": "CC0", "uri": "http://creativecommons.org/publicdomain/zero/1.0" },
                "metadataBlocks": {
                    "citation": {
                        "fields": [
                            {
                                "typeName": "title",
                                "multiple": false,
                                "value": title
                            },
                            {
                                "typeName": "author",
                                "multiple": true,
                                "value": author_values
                            },
                            {
                                "typeName": "dsDescription",
                                "multiple": true,
                                "value": [description_value]
                            },
                            {
                                "typeName": "keyword",
                                "multiple": true,
                                "value": keyword_values
                            },
                            {
                                "typeName": "subject",
                                "multiple": true,
                                "value": ["Medicine, Health and Life Sciences"]
                            }
                        ]
                    }
                }
            }
        })
    }
} 