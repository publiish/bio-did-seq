use actix_web::{web, HttpResponse, Responder};
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use std::io::Write;
use tempfile::NamedTempFile;
use log::{info, error};
use serde::{Deserialize, Serialize};
use crate::errors::AppError;
use crate::models::auth::AuthUser;
use crate::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct DatasetCreateRequest {
    pub title: String,
    pub description: String,
    pub authors: Vec<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DatasetCreateResponse {
    pub id: String,
    pub persistent_id: String,
}

#[derive(Debug, Deserialize)]
pub struct MetadataUpdateRequest {
    pub persistent_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
}

/// Create a new dataset in Dataverse
pub async fn create_dataset(
    req: web::Json<DatasetCreateRequest>,
    app_state: web::Data<AppState>,
    user: web::ReqData<AuthUser>,
) -> Result<impl Responder, AppError> {
    info!("Creating new dataset: {} for user {}", req.title, user.id);
    
    let dataset = app_state.dataverse_service.create_dataset(
        &req.title, 
        &req.description, 
        &req.authors, 
        &req.keywords
    ).await?;
    
    Ok(HttpResponse::Created().json(DatasetCreateResponse {
        id: dataset.id,
        persistent_id: dataset.persistent_id,
    }))
}

/// Upload a file to a dataset
pub async fn upload_file(
    path: web::Path<String>,
    mut payload: Multipart,
    app_state: web::Data<AppState>,
    user: web::ReqData<AuthUser>,
) -> Result<impl Responder, AppError> {
    let persistent_id = path.into_inner();
    info!("Uploading file to dataset: {} for user {}", persistent_id, user.id);
    
    let mut description = String::new();
    let mut temp_file = None;
    
    // Process multipart form
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let name = content_disposition
            .and_then(|cd| cd.get_name())
            .unwrap_or("");
        
        if name == "file" {
            // Create temp file
            let mut tmp = NamedTempFile::new().map_err(|e| {
                error!("Failed to create temp file: {}", e);
                AppError::FileError(format!("Failed to create temp file: {}", e))
            })?;
            
            // Write file content
            while let Ok(Some(chunk)) = field.try_next().await {
                tmp.write_all(&chunk).map_err(|e| {
                    error!("Failed to write to temp file: {}", e);
                    AppError::FileError(format!("Failed to write file: {}", e))
                })?;
            }
            
            temp_file = Some(tmp);
        } else if name == "description" {
            // Read description
            while let Ok(Some(chunk)) = field.try_next().await {
                description = String::from_utf8_lossy(&chunk).to_string();
            }
        }
    }
    
    // Check if we have a file
    let tmp = match temp_file {
        Some(f) => f,
        None => return Err(AppError::ValidationError("No file provided".to_string())),
    };
    
    // Upload the file to Dataverse
    let file_id = app_state.dataverse_service.upload_file(&persistent_id, tmp.path(), &description).await?;
    
    #[derive(Serialize)]
    struct FileResponse {
        file_id: String,
        message: String,
    }
    
    Ok(HttpResponse::Ok().json(FileResponse {
        file_id,
        message: "File uploaded successfully".to_string(),
    }))
}

/// Update dataset metadata
pub async fn update_metadata(
    req: web::Json<MetadataUpdateRequest>,
    app_state: web::Data<AppState>,
    user: web::ReqData<AuthUser>,
) -> Result<impl Responder, AppError> {
    info!("Updating metadata for dataset: {} for user {}", req.persistent_id, user.id);
    
    app_state.dataverse_service.update_metadata(
        &req.persistent_id,
        req.title.as_deref(),
        req.description.as_deref(),
        req.authors.as_ref().map(|v| &v[..]),
        req.keywords.as_ref().map(|v| &v[..]),
    ).await?;
    
    #[derive(Serialize)]
    struct UpdateResponse {
        persistent_id: String,
        status: String,
    }
    
    Ok(HttpResponse::Ok().json(UpdateResponse {
        persistent_id: req.persistent_id.clone(),
        status: "Metadata updated".to_string(),
    }))
}

/// Request to publish a dataset in Dataverse
#[derive(Deserialize)]
pub struct PublishDatasetRequest {
    pub persistent_id: String,
}

/// Response for Dataverse operations
#[derive(Serialize)]
pub struct DataverseResponse {
    pub persistent_id: String,
    pub message: String,
}

/// Publish a dataset in Dataverse
pub async fn publish_dataset(
    user: web::ReqData<AuthUser>,
    app_state: web::Data<AppState>,
    request: web::Json<PublishDatasetRequest>,
) -> Result<impl Responder, AppError> {
    info!("Publishing dataset in Dataverse: {} for user {}", request.persistent_id, user.id);
    
    // Publish the dataset
    app_state.dataverse_service.publish_dataset(&request.persistent_id).await?;
    
    info!("Dataset published in Dataverse: {}", request.persistent_id);
    
    Ok(HttpResponse::Ok().json(DataverseResponse {
        persistent_id: request.persistent_id.clone(),
        message: "Dataset published successfully".to_string(),
    }))
}

/// Get metadata for a dataset in Dataverse
pub async fn get_dataset_metadata(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let persistent_id = path.into_inner();
    
    info!("Getting metadata for dataset: {}", persistent_id);
    
    // Get the dataset metadata
    let metadata = app_state.dataverse_service.get_dataset_metadata(&persistent_id).await?;
    
    Ok(HttpResponse::Ok().json(metadata))
}

/// Initialize Dataverse routes
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dataverse")
            .route("/dataset", web::post().to(create_dataset))
            .route("/dataset/file/{persistent_id}", web::post().to(upload_file))
            .route("/dataset/metadata", web::put().to(update_metadata))
            .route("/dataset/publish", web::post().to(publish_dataset))
            .route("/dataset/{persistent_id}", web::get().to(get_dataset_metadata))
    );
} 