use crate::errors::AppError;
use std::sync::Arc;
use log::{info, error};
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};
use mysql_async::{Pool, prelude::*};
use uuid;

/// Resource types for Bio-DID-Seq capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BioResource {
    // Dataset with optional ID
    Dataset(String),
    // DID with optional ID
    DID(String),
    // File with optional CID
    File(String),
    // Metadata with optional ID
    Metadata(String),
    // User profile with optional ID
    UserProfile(String),
}

/// Actions that can be performed on Bio-DID-Seq resources
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BioAction {
    Create,
    Read,
    Update,
    Delete,
    Upload,
    Download,
    Process,
    Publish,
}

/// Simple capability structure for Bio-DID-Seq
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioCapability {
    pub resource: BioResource,
    pub action: BioAction,
}

/// Token validation result
pub struct TokenValidationData {
    pub issuer: String,
    pub audience: String,
    pub capabilities: Vec<(String, String)>,
    pub expires_at: i64,
}

/// Service for handling UCAN based authorization
pub struct UcanService {
    db_pool: Arc<Pool>,
}

impl UcanService {
    /// Create a new UCAN service
    pub async fn new(db_pool: Arc<Pool>) -> Result<Self, AppError> {
        Ok(Self {
            db_pool,
        })
    }
    
    /// Issue a UCAN token for a user
    pub async fn issue_token(
        &self, 
        user_id: i64, 
        audience_did: &str, 
        capabilities: &[(String, String)], 
        expiration_opt: Option<i64>
    ) -> Result<(String, i64), AppError> {
        let now = Utc::now();
        
        // Default expiration is 24 hours if not specified
        let expiry = match expiration_opt {
            Some(exp_seconds) => now + Duration::seconds(exp_seconds),
            None => now + Duration::hours(24),
        };
        
        let expiry_timestamp = expiry.timestamp();
        
        // In a real implementation, you would use the actual DID of the service as issuer
        let service_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        
        // Format a simplified JWT-like token for demonstration
        let token_id = uuid::Uuid::new_v4().to_string();
        let capabilities_json = serde_json::to_string(&capabilities).unwrap_or_default();
        let token = format!("ucan:demo:{}:{}:{}:{}:{}",
            token_id, 
            service_did, 
            audience_did, 
            now.timestamp(),
            capabilities_json
        );
        
        // Store the token in the database
        let mut conn = self.db_pool.get_conn().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

        let issued_at = now.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        let expires_at = expiry.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        
        "INSERT INTO ucan_tokens (id, user_id, token, audience_did, issued_at, expires_at) VALUES (:id, :user_id, :token, :audience_did, :issued_at, :expires_at)"
            .with(params! {
                "id" => &token_id,
                "user_id" => user_id,
                "token" => &token,
                "audience_did" => audience_did,
                "issued_at" => issued_at,
                "expires_at" => expires_at,
            })
            .run(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when storing UCAN token: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;
        
        info!("Issued UCAN token for user {} to audience {}", user_id, audience_did);
        
        Ok((token, expiry_timestamp))
    }
    
    /// Validate a UCAN token
    pub async fn validate_token(&self, token: &str) -> Result<Result<TokenValidationData, String>, AppError> {
        // Parse token with simple format: ucan:demo:id:issuer:audience:timestamp:capabilities
        let parts: Vec<&str> = token.split(':').collect();
        if parts.len() < 7 || parts[0] != "ucan" || parts[1] != "demo" {
            return Ok(Err("Invalid UCAN token format".to_string()));
        }
        
        // Extract token components
        let token_id = parts[2];
        let issuer = parts[3];
        let audience = parts[4];
        
        // Parse timestamp safely
        let issued_timestamp = match parts[5].parse::<i64>() {
            Ok(ts) => ts,
            Err(_) => return Ok(Err("Invalid timestamp in token".to_string())),
        };
        
        // Log the token information
        info!("Validating token issued at timestamp {}", issued_timestamp);
        
        let capabilities_json = parts[6];
        
        // Check if token is revoked
        let is_revoked = self.is_token_revoked(token).await?;
        if is_revoked {
            return Ok(Err("Token has been revoked".to_string()));
        }
        
        // Check if token is expired
        let now = Utc::now().timestamp();
        
        // Get expiration from database
        let mut conn = self.db_pool.get_conn().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;
        
        // Use string format for the expires_at field instead of NaiveDateTime
        let expires_at: Option<String> = "SELECT DATE_FORMAT(expires_at, '%Y-%m-%d %H:%i:%s') FROM ucan_tokens WHERE id = :id"
            .with(params! {
                "id" => token_id,
            })
            .first(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when checking token expiration: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;
        
        // Parse the expires_at string to a timestamp
        let expires_timestamp = match expires_at {
            Some(dt_str) => {
                match chrono::NaiveDateTime::parse_from_str(&dt_str, "%Y-%m-%d %H:%M:%S") {
                    Ok(dt) => dt.and_utc().timestamp(),
                    Err(_) => return Ok(Err("Invalid expiration date format".to_string())),
                }
            },
            None => return Ok(Err("Token not found in database".to_string())),
        };
        
        if now > expires_timestamp {
            return Ok(Err("Token has expired".to_string()));
        }
        
        // Parse capabilities
        let capabilities: Vec<(String, String)> = match serde_json::from_str(capabilities_json) {
            Ok(caps) => caps,
            Err(_) => return Ok(Err("Invalid capabilities format in token".to_string())),
        };
        
        // Token is valid
        Ok(Ok(TokenValidationData {
            issuer: issuer.to_string(),
            audience: audience.to_string(),
            capabilities,
            expires_at: expires_timestamp,
        }))
    }
    
    /// Revoke a UCAN token
    pub async fn revoke_token(&self, user_id: i64, token: &str) -> Result<(), AppError> {
        // Parse token to extract ID
        let parts: Vec<&str> = token.split(':').collect();
        if parts.len() < 3 || parts[0] != "ucan" || parts[1] != "demo" {
            return Err(AppError::AuthError("Invalid UCAN token format".to_string()));
        }
        
        let token_id = parts[2];
        
        // Check if the user owns the token
        let mut conn = self.db_pool.get_conn().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;
        
        let exists: Option<i32> = "SELECT 1 FROM ucan_tokens WHERE id = :id AND user_id = :user_id"
            .with(params! {
                "id" => token_id,
                "user_id" => user_id,
            })
            .first(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when checking token ownership: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;
        
        if exists.is_none() {
            return Err(AppError::NotFound("Token not found or not owned by user".to_string()));
        }
        
        // Mark the token as revoked in the database
        let now = Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        
        "UPDATE ucan_tokens SET revoked = TRUE, revoked_at = :revoked_at WHERE id = :id"
            .with(params! {
                "revoked_at" => now,
                "id" => token_id,
            })
            .run(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when revoking token: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;
        
        info!("Revoked token {} for user {}", token_id, user_id);
        
        Ok(())
    }
    
    /// Check if a token is revoked
    async fn is_token_revoked(&self, token: &str) -> Result<bool, AppError> {
        // Extract token ID from our simple format
        let token_id = token.split(':').nth(2).ok_or_else(|| {
            error!("Invalid token format");
            AppError::AuthError("Invalid token format".to_string())
        })?;
        
        // Check the database to see if it's revoked
        let mut conn = self.db_pool.get_conn().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;
        
        let revoked: Option<i32> = "SELECT revoked FROM ucan_tokens WHERE id = :id"
            .with(params! {
                "id" => token_id,
            })
            .first(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when checking token revocation: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;
        
        Ok(revoked.unwrap_or(0) == 1)
    }
} 