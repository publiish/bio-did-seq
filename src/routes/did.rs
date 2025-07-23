use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use log::info;

use crate::errors::AppError;
use crate::models::auth::AuthUser;
use crate::models::did::{DIDCreationRequest, DIDUpdateRequest};
use crate::routes::AppState;

/// Request to link a DID to a Dataverse dataset
#[derive(Deserialize)]
pub struct LinkToDataverseRequest {
    pub dataverse_doi: String,
}

/// Create a new DID
pub async fn create_did(
    app_state: web::Data<AppState>,
    user: web::ReqData<AuthUser>,
    req: web::Json<DIDCreationRequest>,
) -> Result<impl Responder, AppError> {
    info!("Creating new DID for user {}", user.id);
    
    let did_doc = app_state.did_service.create_did(req.into_inner(), user.id).await?;
    
    Ok(HttpResponse::Created().json(did_doc))
}

/// Get a DID document by its identifier
pub async fn get_did(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let did_id = path.into_inner();
    
    info!("Retrieving DID document: {}", did_id);
    
    let did_document = app_state.did_service.get_did(&did_id).await?;
    
    Ok(HttpResponse::Ok().json(did_document))
}

/// Update a DID Document
pub async fn update_did(
    app_state: web::Data<AppState>,
    user: web::ReqData<AuthUser>,
    path: web::Path<String>,
    req: web::Json<DIDUpdateRequest>,
) -> Result<impl Responder, AppError> {
    let did = path.into_inner();
    info!("User {} updating DID: {}", user.id, did);
    
    let did_doc = app_state.did_service.update_did(&did, req.into_inner(), user.id).await?;
    
    Ok(HttpResponse::Ok().json(did_doc))
}

/// Link a DID to a Dataverse dataset
pub async fn link_to_dataverse(
    user: web::ReqData<AuthUser>,
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    request: web::Json<LinkToDataverseRequest>,
) -> Result<impl Responder, AppError> {
    let did_id = path.into_inner();
    
    info!("Linking DID: {} to Dataverse DOI: {}", did_id, request.dataverse_doi);
    
    app_state.did_service.link_to_dataverse(&did_id, &request.dataverse_doi, user.id).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "DID successfully linked to Dataverse dataset",
        "did": did_id,
        "dataverse_doi": request.dataverse_doi
    })))
}

/// Resolve a DID to its DID Document
pub async fn resolve_did(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let did = path.into_inner();
    info!("Resolving DID: {}", did);
    
    let did_doc = app_state.did_service.resolve_did(&did).await?;
    
    Ok(HttpResponse::Ok().json(did_doc))
}

/// Initialize DID routes
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/did")
            .route("", web::post().to(create_did))
            .route("/{did}", web::get().to(get_did))
            .route("/{did}", web::put().to(update_did))
            .route("/{did}/dataverse", web::post().to(link_to_dataverse))
            .route("/resolve/{did}", web::get().to(resolve_did))
    );
} 