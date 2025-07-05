use crate::errors::AppError;
use crate::models::did::{DIDDocument, DIDCreationRequest, DIDUpdateRequest, generate_did, create_default_did_document};
use crate::services::ipfs_service::IPFSService;
use crate::services::ucan_service::UcanService;
use std::sync::Arc;
use mysql_async::{Pool, prelude::*};
use chrono::Utc;
use log::{info, error};

/// Service for handling DID document operations
pub struct DIDService {
    db_pool: Arc<Pool>,
    ipfs_service: Arc<IPFSService>,
}

impl DIDService {
    pub fn new(db_pool: Arc<Pool>, ipfs_service: Arc<IPFSService>) -> Self {
        Self {
            db_pool,
            ipfs_service,
        }
    }

    /// Create a new DID document and store it in IPFS
    pub async fn create_did(&self, request: DIDCreationRequest, user_id: i64) -> Result<DIDDocument, AppError> {
        let did = generate_did();
        
        // Create the DID document
        let did_document = create_default_did_document(
            &did,
            &request.controller,
            &request.public_key,
            request.metadata
        );
        
        // Serialize the DID document to JSON
        let did_json = serde_json::to_string(&did_document).map_err(|e| {
            error!("Failed to serialize DID document: {}", e);
            AppError::SerializationError
        })?;
        
        // Store the DID document in IPFS
        let cid = self.ipfs_service.add_content(&did_json).await.map_err(|e| {
            error!("Failed to store DID document in IPFS: {:?}", e);
            e
        })?;
        
        // Store the DID reference in the database
        let now = Utc::now().naive_utc();
        let created_at = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let updated_at = created_at.clone();
        
        let mut conn = self.db_pool.get_conn().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;
        
        "INSERT INTO did_documents (did, cid, user_id, created_at, updated_at) VALUES (:did, :cid, :user_id, :created_at, :updated_at)"
            .with(params! {
                "did" => &did,
                "cid" => &cid,
                "user_id" => user_id,
                "created_at" => created_at,
                "updated_at" => updated_at,
            })
            .run(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when storing DID reference: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;
        
        info!("Created new DID: {} with CID: {}", did, cid);
        
        Ok(did_document)
    }
    
    /// Retrieve a DID document by its DID identifier
    pub async fn get_did(&self, did_id: &str) -> Result<DIDDocument, AppError> {
        // Query the database to get the CID for the DID
        let mut conn = self.db_pool.get_conn().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;
        
        let cid: Option<String> = "SELECT cid FROM did_documents WHERE did = :did"
            .with(params! { "did" => did_id })
            .first(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when retrieving DID reference: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;
        
        let cid = cid.ok_or_else(|| AppError::NotFound("DID not found".to_string()))?;
        
        // Retrieve the DID document from IPFS
        let did_json = self.ipfs_service.get_content(&cid).await.map_err(|e| {
            error!("Failed to retrieve DID document from IPFS: {:?}", e);
            e
        })?;
        
        // Parse the DID document
        let did_document: DIDDocument = serde_json::from_str(&did_json).map_err(|e| {
            error!("Failed to parse DID document: {}", e);
            AppError::DeserializationError
        })?;
        
        Ok(did_document)
    }
    
    /// Update an existing DID document
    pub async fn update_did(&self, did_id: &str, request: DIDUpdateRequest, user_id: i64) -> Result<DIDDocument, AppError> {
        // Check if the user is authorized to update this DID
        let mut conn = self.db_pool.get_conn().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;
        
        let authorized: Option<i32> = "SELECT 1 FROM did_documents WHERE did = :did AND user_id = :user_id"
            .with(params! {
                "did" => did_id,
                "user_id" => user_id,
            })
            .first(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when checking DID authorization: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;
        
        if authorized.is_none() {
            return Err(AppError::AuthorizationError("Not authorized to update this DID".to_string()));
        }
        
        // Get the current DID document
        let mut did_document = self.get_did(did_id).await?;
        
        // Update the controller if specified
        if let Some(controller) = request.controller {
            did_document.controller = vec![controller];
        }
        
        // Add new verification methods if specified
        if let Some(methods) = request.add_verification_method {
            did_document.verification_method.extend(methods);
        }
        
        // Remove verification methods if specified
        if let Some(method_ids) = request.remove_verification_method {
            did_document.verification_method.retain(|method| !method_ids.contains(&method.id));
        }
        
        // Add new services if specified
        if let Some(services) = request.add_service {
            did_document.service.extend(services);
        }
        
        // Remove services if specified
        if let Some(service_ids) = request.remove_service {
            did_document.service.retain(|service| !service_ids.contains(&service.id));
        }
        
        // Update metadata if specified
        if let Some(metadata) = request.update_metadata {
            did_document.metadata = Some(metadata);
        }
        
        // Update the timestamp
        did_document.updated = Utc::now();
        
        // Serialize the updated DID document to JSON
        let did_json = serde_json::to_string(&did_document).map_err(|e| {
            error!("Failed to serialize updated DID document: {}", e);
            AppError::SerializationError
        })?;
        
        // Store the updated DID document in IPFS
        let cid = self.ipfs_service.add_content(&did_json).await.map_err(|e| {
            error!("Failed to store updated DID document in IPFS: {:?}", e);
            e
        })?;
        
        // Update the DID reference in the database
        let now = Utc::now().naive_utc();
        let updated_at = now.format("%Y-%m-%d %H:%M:%S").to_string();
        
        let mut conn = self.db_pool.get_conn().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;
        
        "UPDATE did_documents SET cid = :cid, updated_at = :updated_at WHERE did = :did"
            .with(params! {
                "cid" => &cid,
                "updated_at" => updated_at,
                "did" => did_id,
            })
            .run(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when updating DID reference: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;
        
        info!("Updated DID: {} with new CID: {}", did_id, cid);
        
        Ok(did_document)
    }
    
    /// Resolve a DID document and validate it
    pub async fn resolve_did(&self, did_id: &str) -> Result<DIDDocument, AppError> {
        // For now, we simply retrieve the DID document
        // In a production system, we would also perform validation here
        self.get_did(did_id).await
    }
    
    /// Create a link between a DID and a Dataverse dataset
    pub async fn link_to_dataverse(&self, did_id: &str, dataverse_doi: &str, user_id: i64) -> Result<(), AppError> {
        // Check if the user is authorized to update this DID
        let mut conn = self.db_pool.get_conn().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;
        
        let authorized: Option<i32> = "SELECT 1 FROM did_documents WHERE did = :did AND user_id = :user_id"
            .with(params! {
                "did" => did_id,
                "user_id" => user_id,
            })
            .first(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when checking DID authorization: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;
        
        if authorized.is_none() {
            return Err(AppError::AuthorizationError("Not authorized to link this DID".to_string()));
        }
        
        // Get the current DID document
        let mut did_document = self.get_did(did_id).await?;
        
        // Update the metadata to include the Dataverse link
        if let Some(ref mut metadata) = did_document.metadata {
            metadata.doi = Some(dataverse_doi.to_string());
            metadata.dataverse_link = Some(format!("https://dataverse.harvard.edu/dataset.xhtml?persistentId={}", dataverse_doi));
        }
        
        // Update the DID document in IPFS
        let did_json = serde_json::to_string(&did_document).map_err(|e| {
            error!("Failed to serialize updated DID document: {}", e);
            AppError::SerializationError
        })?;
        
        let cid = self.ipfs_service.add_content(&did_json).await.map_err(|e| {
            error!("Failed to store updated DID document in IPFS: {:?}", e);
            e
        })?;
        
        // Update the DID reference in the database
        let now = Utc::now().naive_utc();
        let updated_at = now.format("%Y-%m-%d %H:%M:%S").to_string();
        
        let mut conn = self.db_pool.get_conn().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;
        
        "UPDATE did_documents SET cid = :cid, updated_at = :updated_at, dataverse_doi = :dataverse_doi WHERE did = :did"
            .with(params! {
                "cid" => &cid,
                "updated_at" => updated_at,
                "dataverse_doi" => dataverse_doi,
                "did" => did_id,
            })
            .run(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when updating DID reference: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;
        
        info!("Linked DID: {} to Dataverse DOI: {}", did_id, dataverse_doi);
        
        Ok(())
    }
} 