use crate::errors::AppError;
use crate::models::file_metadata::{BiologicalEntityReference, ResearchPaperMetadata};
use crate::services::bioagents_service::{BioAgentsService, ExtractedMetadata};
use crate::services::did_service::DIDService;
use crate::services::ipfs_service::IPFSService;
use chrono::{TimeZone, Utc};
use log::{error, info};
use mysql_async::{params, prelude::*, Row};
use serde::Deserialize;
use std::sync::Arc;

/// Database row representation for research paper metadata
#[derive(Debug, Deserialize)]
struct PaperDbRow {
    title: String,
    authors: String,
    abstract_text: String,
    doi: Option<String>,
    publication_date: Option<String>,
    journal: Option<String>,
    keywords: String,
    cid: String,
    did: String,
    biological_entities: String,
    knowledge_graph_cid: Option<String>,
    created_at: String,
    updated_at: String,
}

impl FromRow for PaperDbRow {
    fn from_row(row: Row) -> Self {
        Self {
            title: row.get(0).unwrap_or_default(),
            authors: row.get(1).unwrap_or_default(),
            abstract_text: row.get(2).unwrap_or_default(),
            doi: row.get(3),
            publication_date: row.get(4),
            journal: row.get(5),
            keywords: row.get(6).unwrap_or_default(),
            cid: row.get(7).unwrap_or_default(),
            did: row.get(8).unwrap_or_default(),
            biological_entities: row.get(9).unwrap_or_default(),
            knowledge_graph_cid: row.get(10),
            created_at: row.get(11).unwrap_or_default(),
            updated_at: row.get(12).unwrap_or_default(),
        }
    }

    fn from_row_opt(row: Row) -> Result<Self, mysql_async::FromRowError> {
        Ok(Self {
            title: row
                .get(0)
                .ok_or_else(|| mysql_async::FromRowError(row.clone()))?,
            authors: row
                .get(1)
                .ok_or_else(|| mysql_async::FromRowError(row.clone()))?,
            abstract_text: row
                .get(2)
                .ok_or_else(|| mysql_async::FromRowError(row.clone()))?,
            doi: row.get(3),
            publication_date: row.get(4),
            journal: row.get(5),
            keywords: row
                .get(6)
                .ok_or_else(|| mysql_async::FromRowError(row.clone()))?,
            cid: row
                .get(7)
                .ok_or_else(|| mysql_async::FromRowError(row.clone()))?,
            did: row
                .get(8)
                .ok_or_else(|| mysql_async::FromRowError(row.clone()))?,
            biological_entities: row
                .get(9)
                .ok_or_else(|| mysql_async::FromRowError(row.clone()))?,
            knowledge_graph_cid: row.get(10),
            created_at: row
                .get(11)
                .ok_or_else(|| mysql_async::FromRowError(row.clone()))?,
            updated_at: row
                .get(12)
                .ok_or_else(|| mysql_async::FromRowError(row.clone()))?,
        })
    }
}

/// Service for managing research paper metadata
pub struct ResearchPaperService {
    db_pool: Arc<mysql_async::Pool>,
    #[allow(dead_code)]
    ipfs_service: Arc<IPFSService>,
    did_service: Arc<DIDService>,
    bioagents_service: Arc<BioAgentsService>,
}

impl ResearchPaperService {
    /// Create a new ResearchPaperService
    pub fn new(
        db_pool: Arc<mysql_async::Pool>,
        ipfs_service: Arc<IPFSService>,
        did_service: Arc<DIDService>,
        bioagents_service: Arc<BioAgentsService>,
    ) -> Self {
        Self {
            db_pool,
            ipfs_service,
            did_service,
            bioagents_service,
        }
    }

    /// Create a new research paper metadata entry
    pub async fn create_paper_metadata(
        &self,
        metadata: ExtractedMetadata,
        file_cid: &str,
        did: &str,
        user_id: i64,
        knowledge_graph_cid: Option<&str>,
    ) -> Result<ResearchPaperMetadata, AppError> {
        let now = Utc::now();
        let created_at = now.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
        let updated_at = created_at.clone();

        // Convert BioAgents entities to our internal format
        let biological_entities: Vec<BiologicalEntityReference> = metadata
            .biological_entities
            .into_iter()
            .map(|entity| BiologicalEntityReference {
                entity_type: entity.entity_type,
                name: entity.name,
                identifier: entity.identifier,
                source: entity.source,
            })
            .collect();

        // Create the research paper metadata object
        let paper_metadata = ResearchPaperMetadata {
            title: metadata.title,
            authors: metadata.authors,
            abstract_text: metadata.abstract_text,
            doi: metadata.doi,
            publication_date: metadata.publication_date,
            journal: metadata.journal,
            keywords: metadata.keywords,
            cid: file_cid.to_string(),
            did: did.to_string(),
            biological_entities: biological_entities.clone(),
            knowledge_graph_cid: knowledge_graph_cid.map(|cid| cid.to_string()),
            created_at: now,
            updated_at: now,
        };

        // Serialize the JSON fields
        let authors_json = serde_json::to_string(&paper_metadata.authors)
            .map_err(|_| AppError::SerializationError)?;
        let keywords_json = serde_json::to_string(&paper_metadata.keywords)
            .map_err(|_| AppError::SerializationError)?;
        let biological_entities_json = serde_json::to_string(&biological_entities)
            .map_err(|_| AppError::SerializationError)?;

        // Store the metadata in the database
        let mut conn = self.db_pool.get_conn().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

        "INSERT INTO research_papers (title, authors, abstract_text, doi, publication_date, journal, keywords, cid, did, biological_entities, knowledge_graph_cid, created_at, updated_at, user_id) VALUES (:title, :authors, :abstract_text, :doi, :publication_date, :journal, :keywords, :cid, :did, :biological_entities, :knowledge_graph_cid, :created_at, :updated_at, :user_id)"
            .with(params! {
                "title" => &paper_metadata.title,
                "authors" => &authors_json,
                "abstract_text" => &paper_metadata.abstract_text,
                "doi" => &paper_metadata.doi,
                "publication_date" => &paper_metadata.publication_date,
                "journal" => &paper_metadata.journal,
                "keywords" => &keywords_json,
                "cid" => &paper_metadata.cid,
                "did" => &paper_metadata.did,
                "biological_entities" => &biological_entities_json,
                "knowledge_graph_cid" => &paper_metadata.knowledge_graph_cid,
                "created_at" => &created_at,
                "updated_at" => &updated_at,
                "user_id" => user_id,
            })
            .run(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when storing research paper metadata: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;

        info!("Created research paper metadata for DID: {}", did);

        Ok(paper_metadata)
    }

    /// Get research paper metadata by DID
    pub async fn get_paper_metadata_by_did(
        &self,
        did: &str,
    ) -> Result<ResearchPaperMetadata, AppError> {
        let mut conn = self.db_pool.get_conn().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

        // Query the database for the paper metadata
        let row = "SELECT title, authors, abstract_text, doi, publication_date, journal, keywords, cid, did, biological_entities, knowledge_graph_cid, created_at, updated_at FROM research_papers WHERE did = :did"
            .with(params! { "did" => did })
            .first::<PaperDbRow, _>(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when retrieving research paper metadata: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;

        let row = row.ok_or_else(|| {
            AppError::NotFound(format!(
                "Research paper metadata not found for DID: {}",
                did
            ))
        })?;

        // Parse the JSON fields
        let authors: Vec<String> =
            serde_json::from_str(&row.authors).map_err(|_| AppError::DeserializationError)?;
        let keywords: Vec<String> =
            serde_json::from_str(&row.keywords).map_err(|_| AppError::DeserializationError)?;
        let biological_entities: Vec<BiologicalEntityReference> =
            serde_json::from_str(&row.biological_entities)
                .map_err(|_| AppError::DeserializationError)?;

        // Parse the timestamps
        let created_at =
            chrono::NaiveDateTime::parse_from_str(&row.created_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|_| AppError::DeserializationError)?;
        let updated_at =
            chrono::NaiveDateTime::parse_from_str(&row.updated_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|_| AppError::DeserializationError)?;

        // Create the research paper metadata object
        let paper_metadata = ResearchPaperMetadata {
            title: row.title,
            authors,
            abstract_text: row.abstract_text,
            doi: row.doi,
            publication_date: row.publication_date,
            journal: row.journal,
            keywords,
            cid: row.cid,
            did: row.did,
            biological_entities,
            knowledge_graph_cid: row.knowledge_graph_cid,
            created_at: Utc.from_utc_datetime(&created_at),
            updated_at: Utc.from_utc_datetime(&updated_at),
        };

        Ok(paper_metadata)
    }

    /// Get research paper metadata by CID
    pub async fn get_paper_metadata_by_cid(
        &self,
        cid: &str,
    ) -> Result<ResearchPaperMetadata, AppError> {
        let mut conn = self.db_pool.get_conn().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

        // Query the database for the paper metadata
        let row = "SELECT title, authors, abstract_text, doi, publication_date, journal, keywords, cid, did, biological_entities, knowledge_graph_cid, created_at, updated_at FROM research_papers WHERE cid = :cid"
            .with(params! { "cid" => cid })
            .first::<PaperDbRow, _>(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when retrieving research paper metadata: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;

        let row = row.ok_or_else(|| {
            AppError::NotFound(format!(
                "Research paper metadata not found for CID: {}",
                cid
            ))
        })?;

        // Parse the JSON fields
        let authors: Vec<String> =
            serde_json::from_str(&row.authors).map_err(|_| AppError::DeserializationError)?;
        let keywords: Vec<String> =
            serde_json::from_str(&row.keywords).map_err(|_| AppError::DeserializationError)?;
        let biological_entities: Vec<BiologicalEntityReference> =
            serde_json::from_str(&row.biological_entities)
                .map_err(|_| AppError::DeserializationError)?;

        // Parse the timestamps
        let created_at =
            chrono::NaiveDateTime::parse_from_str(&row.created_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|_| AppError::DeserializationError)?;
        let updated_at =
            chrono::NaiveDateTime::parse_from_str(&row.updated_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|_| AppError::DeserializationError)?;

        // Create the research paper metadata object
        let paper_metadata = ResearchPaperMetadata {
            title: row.title,
            authors,
            abstract_text: row.abstract_text,
            doi: row.doi,
            publication_date: row.publication_date,
            journal: row.journal,
            keywords,
            cid: row.cid,
            did: row.did,
            biological_entities,
            knowledge_graph_cid: row.knowledge_graph_cid,
            created_at: Utc.from_utc_datetime(&created_at),
            updated_at: Utc.from_utc_datetime(&updated_at),
        };

        Ok(paper_metadata)
    }

    /// Search for research papers by keywords
    pub async fn search_papers(&self, query: &str) -> Result<Vec<ResearchPaperMetadata>, AppError> {
        let mut conn = self.db_pool.get_conn().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

        // Query the database for papers matching the search term
        let rows = "SELECT title, authors, abstract_text, doi, publication_date, journal, keywords, cid, did, biological_entities, knowledge_graph_cid, created_at, updated_at FROM research_papers WHERE title LIKE :query OR abstract_text LIKE :query"
            .with(params! { "query" => format!("%{}%", query) })
            .fetch::<PaperDbRow, _>(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error when searching research papers: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;

        // Convert the rows to ResearchPaperMetadata objects
        let mut results = Vec::new();
        for row in rows {
            // Parse the JSON fields
            let authors: Vec<String> =
                serde_json::from_str(&row.authors).map_err(|_| AppError::DeserializationError)?;
            let keywords: Vec<String> =
                serde_json::from_str(&row.keywords).map_err(|_| AppError::DeserializationError)?;
            let biological_entities: Vec<BiologicalEntityReference> =
                serde_json::from_str(&row.biological_entities)
                    .map_err(|_| AppError::DeserializationError)?;

            // Parse the timestamps
            let created_at =
                chrono::NaiveDateTime::parse_from_str(&row.created_at, "%Y-%m-%d %H:%M:%S")
                    .map_err(|_| AppError::DeserializationError)?;
            let updated_at =
                chrono::NaiveDateTime::parse_from_str(&row.updated_at, "%Y-%m-%d %H:%M:%S")
                    .map_err(|_| AppError::DeserializationError)?;

            // Create the research paper metadata object
            let paper_metadata = ResearchPaperMetadata {
                title: row.title,
                authors,
                abstract_text: row.abstract_text,
                doi: row.doi,
                publication_date: row.publication_date,
                journal: row.journal,
                keywords,
                cid: row.cid,
                did: row.did,
                biological_entities,
                knowledge_graph_cid: row.knowledge_graph_cid,
                created_at: Utc.from_utc_datetime(&created_at),
                updated_at: Utc.from_utc_datetime(&updated_at),
            };

            results.push(paper_metadata);
        }

        Ok(results)
    }

    /// Process a research paper with BioAgents and create metadata
    pub async fn process_paper_and_create_metadata(
        &self,
        file_cid: &str,
        title: &str,
        authors: &[String],
        doi: Option<&str>,
        user_id: i64,
    ) -> Result<String, AppError> {
        // First, create a DID for the paper
        let did_metadata = crate::models::did::BiometadataExtension {
            title: title.to_string(),
            description: Some(format!("Research paper: {}", title)),
            researchers: authors
                .iter()
                .map(|author| crate::models::did::Researcher {
                    name: author.clone(),
                    orcid: None,
                    role: "Author".to_string(),
                    affiliation: None,
                    email: None,
                })
                .collect(),
            // Will be updated after processing
            keywords: Vec::new(),
            data_type: "Research Paper".to_string(),
            license: "CC-BY-4.0".to_string(),
            doi: doi.map(|d| d.to_string()),
            handle: None,
            dataverse_link: None,
            related_identifiers: None,
            dataset_size: None,
            funding_info: None,
            creation_date: Utc::now(),
            last_modified: Utc::now(),
            custom_fields: None,
        };

        // Create a DID for the paper
        let did_request = crate::models::did::DIDCreationRequest {
            // This should be the user's actual DID
            controller: format!("did:key:user{}", user_id),
            // This should be generated
            public_key: "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
            service_endpoints: Vec::new(),
            metadata: did_metadata,
        };

        let did_doc = self.did_service.create_did(did_request, user_id).await?;
        let did = did_doc.id.clone();

        info!("Created DID for paper: {}", did);

        // Process the paper with BioAgents
        let process_request = crate::services::bioagents_service::ProcessPaperRequest {
            file_cid: file_cid.to_string(),
            title: title.to_string(),
            authors: authors.to_vec(),
            doi: doi.map(|d| d.to_string()),
            extract_metadata: true,
            generate_knowledge_graph: true,
        };

        let process_response = self
            .bioagents_service
            .process_paper(process_request)
            .await?;
        let task_id = process_response.task_id;

        info!("Started BioAgents processing with task ID: {}", task_id);

        // Wait for the task to complete (in a Production system, this would be handled asynchronously)
        let mut status = self.bioagents_service.check_task_status(&task_id).await?;

        // Simple polling mechanism - in production, this should be replaced with a proper async workflow
        let mut attempts = 0;
        while status.status != "completed" && status.status != "failed" && attempts < 10 {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            status = self.bioagents_service.check_task_status(&task_id).await?;
            attempts += 1;
        }

        if status.status == "failed" {
            return Err(AppError::ExternalServiceError(format!(
                "BioAgents processing failed: {:?}",
                status.error
            )));
        }

        if status.status != "completed" {
            return Err(AppError::ExternalServiceError(
                "BioAgents processing timed out".to_string(),
            ));
        }

        // Get the extracted metadata
        let metadata = self
            .bioagents_service
            .get_extracted_metadata(&task_id)
            .await?;

        // Get the knowledge graph CID if available
        let knowledge_graph_cid = if let Some(result) = &status.result {
            result
                .get("knowledge_graph_cid")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        };

        // Create the paper metadata
        let paper_metadata = self
            .create_paper_metadata(
                metadata,
                file_cid,
                &did,
                user_id,
                knowledge_graph_cid.as_deref(),
            )
            .await?;

        // Update the DID document with the keywords from the metadata
        if !paper_metadata.keywords.is_empty() {
            let update_request = crate::models::did::DIDUpdateRequest {
                controller: None,
                add_verification_method: None,
                remove_verification_method: None,
                add_service: None,
                remove_service: None,
                update_metadata: Some(crate::models::did::BiometadataExtension {
                    title: paper_metadata.title.clone(),
                    description: Some(paper_metadata.abstract_text.clone()),
                    researchers: did_doc.metadata.unwrap().researchers,
                    keywords: paper_metadata.keywords.clone(),
                    data_type: "Research Paper".to_string(),
                    license: "CC-BY-4.0".to_string(),
                    doi: paper_metadata.doi.clone(),
                    handle: None,
                    dataverse_link: None,
                    related_identifiers: None,
                    dataset_size: None,
                    funding_info: None,
                    creation_date: Utc::now(),
                    last_modified: Utc::now(),
                    custom_fields: None,
                }),
            };

            self.did_service
                .update_did(&did, update_request, user_id)
                .await?;
        }

        Ok(did)
    }
}
