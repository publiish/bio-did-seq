use crate::models::{auth::AuthResponse, requests::*};
use crate::errors::AppError;
use crate::models::auth::AuthUser;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use log::info;
use crate::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct UcanIssueRequest {
    pub audience: String,
    pub capabilities: Vec<UcanCapability>,
    pub expiration: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UcanCapability {
    pub with: String,
    pub can: String,
}

#[derive(Debug, Serialize)]
pub struct UcanResponse {
    pub token: String,
    pub expires_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct UcanValidateRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct UcanValidationResponse {
    pub valid: bool,
    pub issuer: Option<String>,
    pub audience: Option<String>,
    pub capabilities: Option<Vec<UcanCapability>>,
    pub expires_at: Option<i64>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UcanRevokeRequest {
    pub token: String,
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/signup", web::post().to(signup))
        .route("/signin", web::post().to(signin))
        .route("/ucan/issue", web::post().to(issue_ucan))
        .route("/ucan/validate", web::post().to(validate_ucan))
        .route("/ucan/revoke", web::post().to(revoke_ucan));
}

/// Handles user signup requests
/// POST /api/signup
async fn signup(
    app_state: web::Data<AppState>,
    req: web::Json<SignupRequest>,
) -> Result<HttpResponse, actix_web::error::Error> {
    let token = app_state.ipfs_service.signup(req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(AuthResponse { token }))
}

/// Handles user signin requests
/// POST /api/signin
async fn signin(
    app_state: web::Data<AppState>,
    req: web::Json<SigninRequest>,
) -> Result<HttpResponse, actix_web::error::Error> {
    let token = app_state.ipfs_service.signin(req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(AuthResponse { token }))
}

/// Issue a new UCAN token
/// POST /api/ucan/issue
async fn issue_ucan(
    app_state: web::Data<AppState>,
    user: web::ReqData<AuthUser>,
    req: web::Json<UcanIssueRequest>,
) -> Result<impl Responder, AppError> {
    info!("User {} is issuing a UCAN token for {}", user.id, req.audience);
    
    let capabilities = req.capabilities.iter()
        .map(|cap| (cap.with.clone(), cap.can.clone()))
        .collect::<Vec<_>>();
    
    let (token, expires_at) = app_state.ucan_service.issue_token(
        user.id,
        &req.audience,
        &capabilities,
        req.expiration,
    ).await?;
    
    Ok(HttpResponse::Created().json(UcanResponse {
        token,
        expires_at,
    }))
}

/// Validate a UCAN token
/// POST /api/ucan/validate
async fn validate_ucan(
    app_state: web::Data<AppState>,
    req: web::Json<UcanValidateRequest>,
) -> Result<impl Responder, AppError> {
    info!("Validating UCAN token");
    
    let validation = app_state.ucan_service.validate_token(&req.token).await?;
    
    let response = match validation {
        Ok(data) => {
            let capabilities = data.capabilities.into_iter()
                .map(|(with, can)| UcanCapability { with, can })
                .collect();
                
            UcanValidationResponse {
                valid: true,
                issuer: Some(data.issuer),
                audience: Some(data.audience),
                capabilities: Some(capabilities),
                expires_at: Some(data.expires_at),
                reason: None,
            }
        },
        Err(e) => UcanValidationResponse {
            valid: false,
            issuer: None,
            audience: None,
            capabilities: None,
            expires_at: None,
            reason: Some(e),
        }
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Revoke a UCAN token
/// POST /api/ucan/revoke
async fn revoke_ucan(
    app_state: web::Data<AppState>,
    user: web::ReqData<AuthUser>,
    req: web::Json<UcanRevokeRequest>,
) -> Result<impl Responder, AppError> {
    info!("User {} is revoking a UCAN token", user.id);
    
    app_state.ucan_service.revoke_token(user.id, &req.token).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Token revoked successfully"
    })))
}
