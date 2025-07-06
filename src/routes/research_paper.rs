use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use log::info;
use std::sync::Arc;

use crate::errors::AppError;
use crate::models::auth::AuthUser;
use crate::services::research_paper_service::ResearchPaperService;

/// Request to process a research paper and create metadata
#[derive(Deserialize)]
pub struct ProcessPaperRequest {
    pub file_cid: String,
    pub title: String,
    pub authors: Vec<String>,
    pub doi: Option<String>,
}

/// Request to get research paper metadata by DID or CID
#[derive(Deserialize)]
pub struct GetPaperMetadataRequest {
    pub identifier: String,
    // "did" or "cid"
    pub identifier_type: String,
}

/// Request to search for research papers
#[derive(Deserialize)]
pub struct SearchPapersRequest {
    pub query: String,
}

/// Process a research paper and create metadata
pub async fn process_paper(
    user: web::ReqData<AuthUser>,
    service: web::Data<Arc<ResearchPaperService>>,
    request: web::Json<ProcessPaperRequest>,
) -> Result<impl Responder, AppError> {
    info!("Processing research paper for user {}: {}", user.id, request.title);
    
    let did = service.process_paper_and_create_metadata(
        &request.file_cid,
        &request.title,
        &request.authors,
        request.doi.as_deref(),
        user.id,
    ).await?;
    
    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "message": "Research paper processed successfully",
        "did": did
    })))
}

/// Get research paper metadata by DID
pub async fn get_paper_metadata_by_did(
    service: web::Data<Arc<ResearchPaperService>>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let did = path.into_inner();
    info!("Getting research paper metadata for DID: {}", did);
    
    let metadata = service.get_paper_metadata_by_did(&did).await?;
    
    Ok(HttpResponse::Ok().json(metadata))
}

/// Get research paper metadata by CID
pub async fn get_paper_metadata_by_cid(
    service: web::Data<Arc<ResearchPaperService>>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let cid = path.into_inner();
    info!("Getting research paper metadata for CID: {}", cid);
    
    let metadata = service.get_paper_metadata_by_cid(&cid).await?;
    
    Ok(HttpResponse::Ok().json(metadata))
}

/// Search for research papers
pub async fn search_papers(
    service: web::Data<Arc<ResearchPaperService>>,
    query: web::Query<SearchPapersRequest>,
) -> Result<impl Responder, AppError> {
    info!("Searching for research papers with query: {}", query.query);
    
    let papers = service.search_papers(&query.query).await?;
    
    Ok(HttpResponse::Ok().json(papers))
}

/// Initialize research paper routes
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/research-paper")
            .route("", web::post().to(process_paper))
            .route("/did/{did}", web::get().to(get_paper_metadata_by_did))
            .route("/cid/{cid}", web::get().to(get_paper_metadata_by_cid))
            .route("/search", web::get().to(search_papers))
    );
} 