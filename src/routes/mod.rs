use crate::services::ipfs_service::IPFSService;
use crate::services::did_service::DIDService;
use crate::services::bioagents_service::BioAgentsService;
use crate::services::dataverse_service::DataverseService;
use crate::services::ucan_service::UcanService;
use actix_web::web;
use std::sync::Arc;

pub mod auth;
pub mod bioagents;
pub mod dataverse;
pub mod did;
pub mod file;

#[derive(Clone)]
pub struct AppState {
    pub ipfs_service: Arc<IPFSService>,
    pub did_service: Arc<DIDService>,
    pub bioagents_service: Arc<BioAgentsService>,
    pub dataverse_service: Arc<DataverseService>,
    pub ucan_service: Arc<UcanService>,
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(auth::init_routes)
            .configure(file::init_routes)
            .configure(did::init_routes)
            .configure(bioagents::init_routes)
            .configure(dataverse::init_routes),
    );
}
