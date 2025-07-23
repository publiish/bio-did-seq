use actix_web::{web, HttpResponse, Responder};
use log::info;
use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::models::auth::AuthUser;
use crate::routes::AppState;
use crate::services::bioagents_service::ProcessPaperRequest;

/// Request to process a paper
#[derive(Deserialize)]
pub struct ProcessPaperApiRequest {
    pub file_cid: String,
    pub title: String,
    pub authors: Vec<String>,
    pub doi: Option<String>,
}

/// Request to check task status
#[derive(Deserialize)]
pub struct TaskStatusRequest {
    pub task_id: String,
}

/// Request to search for biological entities
#[derive(Deserialize)]
pub struct EntitySearchRequest {
    pub query: String,
}

/// Request to extract metadata
#[derive(Deserialize)]
pub struct ExtractMetadataRequest {
    pub task_id: String,
}

/// Request to generate a knowledge graph
#[derive(Deserialize)]
pub struct GenerateKnowledgeGraphRequest {
    pub cid: String,
}

/// Process a paper through BioAgents
pub async fn process_paper(
    user: web::ReqData<AuthUser>,
    app_state: web::Data<AppState>,
    request: web::Json<ProcessPaperApiRequest>,
) -> Result<impl Responder, AppError> {
    info!("Processing paper: {} for user: {}", request.title, user.id);

    let service_request = ProcessPaperRequest {
        file_cid: request.file_cid.clone(),
        title: request.title.clone(),
        authors: request.authors.clone(),
        doi: request.doi.clone(),
        extract_metadata: true,
        generate_knowledge_graph: true,
    };

    let response = app_state
        .bioagents_service
        .process_paper(service_request)
        .await?;

    Ok(HttpResponse::Accepted().json(response))
}

/// Check the status of a paper processing task
pub async fn check_task_status(
    user: web::ReqData<AuthUser>,
    app_state: web::Data<AppState>,
    request: web::Json<TaskStatusRequest>,
) -> Result<impl Responder, AppError> {
    info!(
        "Checking task status: {} for user: {}",
        request.task_id, user.id
    );

    let status = app_state
        .bioagents_service
        .check_task_status(&request.task_id)
        .await?;

    Ok(HttpResponse::Ok().json(status))
}

/// Get extracted metadata for a completed task
pub async fn get_extracted_metadata(
    user: web::ReqData<AuthUser>,
    app_state: web::Data<AppState>,
    request: web::Json<ExtractMetadataRequest>,
) -> Result<impl Responder, AppError> {
    info!(
        "Getting extracted metadata for task: {} for user: {}",
        request.task_id, user.id
    );

    let metadata = app_state
        .bioagents_service
        .get_extracted_metadata(&request.task_id)
        .await?;

    Ok(HttpResponse::Ok().json(metadata))
}

/// Search for related biological entities
pub async fn search_entities(
    user: web::ReqData<AuthUser>,
    app_state: web::Data<AppState>,
    request: web::Json<EntitySearchRequest>,
) -> Result<impl Responder, AppError> {
    info!(
        "Searching for entities with query: {} for user: {}",
        request.query, user.id
    );

    let entities = app_state
        .bioagents_service
        .search_related_entities(&request.query)
        .await?;

    Ok(HttpResponse::Ok().json(entities))
}

/// Generate a knowledge graph for a paper
pub async fn generate_knowledge_graph(
    user: web::ReqData<AuthUser>,
    app_state: web::Data<AppState>,
    request: web::Json<GenerateKnowledgeGraphRequest>,
) -> Result<impl Responder, AppError> {
    info!(
        "Generating knowledge graph for paper with CID: {} for user: {}",
        request.cid, user.id
    );

    let knowledge_graph_cid = app_state
        .bioagents_service
        .generate_knowledge_graph(&request.cid)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "knowledge_graph_cid": knowledge_graph_cid
    })))
}

#[derive(Debug, Deserialize)]
pub struct AgentQueryRequest {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct AgentQueryResponse {
    pub answer: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct KnowledgeAddRequest {
    pub title: String,
    pub content: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct KnowledgeAddResponse {
    pub id: String,
    pub status: String,
}

/// Query bioagents with a natural language question
pub async fn query_agents(
    req: web::Json<AgentQueryRequest>,
    app_state: web::Data<AppState>,
    user: web::ReqData<AuthUser>,
) -> Result<impl Responder, AppError> {
    info!("User {} is querying bioagents with: {}", user.id, req.query);

    let (answer, sources) = app_state.bioagents_service.query_agents(&req.query).await?;

    Ok(HttpResponse::Ok().json(AgentQueryResponse { answer, sources }))
}

/// Add knowledge to the bioagent system
pub async fn add_knowledge(
    req: web::Json<KnowledgeAddRequest>,
    app_state: web::Data<AppState>,
    user: web::ReqData<AuthUser>,
) -> Result<impl Responder, AppError> {
    info!("User {} is adding knowledge: {}", user.id, req.title);

    let id = app_state
        .bioagents_service
        .add_knowledge(&req.title, &req.content, &req.keywords)
        .await?;

    Ok(HttpResponse::Ok().json(KnowledgeAddResponse {
        id,
        status: "success".to_string(),
    }))
}

/// Get the health status of connected bioagents
pub async fn health_check(app_state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    let status = app_state.bioagents_service.check_health().await?;

    #[derive(Serialize)]
    struct HealthStatus {
        status: String,
        agents_online: i32,
    }

    Ok(HttpResponse::Ok().json(HealthStatus {
        status: if status.agents_online > 0 {
            "ok".to_string()
        } else {
            "degraded".to_string()
        },
        agents_online: status.agents_online,
    }))
}

/// Initialize BioAgents routes
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/bioagents")
            .route("/process", web::post().to(process_paper))
            .route("/status", web::post().to(check_task_status))
            .route("/metadata", web::post().to(get_extracted_metadata))
            .route("/search", web::post().to(search_entities))
            .route("/knowledge-graph", web::post().to(generate_knowledge_graph))
            .route("/query", web::post().to(query_agents))
            .route("/knowledge", web::post().to(add_knowledge))
            .route("/health", web::get().to(health_check)),
    );
}
