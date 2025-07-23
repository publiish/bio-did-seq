use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// W3C-compliant DID Document for biological research data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDDocument {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "alsoKnownAs")]
    pub also_known_as: Option<Vec<String>>,
    pub controller: Vec<String>,
    #[serde(rename = "verificationMethod")]
    pub verification_method: Vec<VerificationMethod>,
    pub authentication: Vec<String>,
    #[serde(rename = "assertionMethod")]
    pub assertion_method: Option<Vec<String>>,
    pub service: Vec<Service>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BiometadataExtension>,
}

/// Verification method for authenticating control of the DID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    pub controller: String,
    #[serde(rename = "type")]
    pub vm_type: String,
    #[serde(rename = "publicKeyMultibase")]
    pub public_key_multibase: Option<String>,
    #[serde(rename = "publicKeyJwk")]
    pub public_key_jwk: Option<KeyJwk>,
}

/// JSON Web Key for cryptographic operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyJwk {
    pub kty: String,
    pub crv: String,
    pub x: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,
}

/// Service endpoint definition for DID document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    #[serde(rename = "type")]
    pub service_type: String,
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Biological metadata extension for research data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometadataExtension {
    pub title: String,
    pub description: Option<String>,
    pub researchers: Vec<Researcher>,
    pub keywords: Vec<String>,
    pub data_type: String,
    pub license: String,
    pub doi: Option<String>,
    pub handle: Option<String>,
    pub dataverse_link: Option<String>,
    pub related_identifiers: Option<Vec<RelatedIdentifier>>,
    pub dataset_size: Option<u64>,
    pub funding_info: Option<Vec<FundingInfo>>,
    pub creation_date: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

/// Researcher information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Researcher {
    pub name: String,
    pub orcid: Option<String>,
    pub role: String,
    pub affiliation: Option<String>,
    pub email: Option<String>,
}

/// Related identifier for cross-referencing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedIdentifier {
    pub identifier: String,
    pub identifier_type: String,
    pub relation_type: String,
}

/// Funding information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingInfo {
    pub funder_name: String,
    pub grant_id: Option<String>,
    pub award_title: Option<String>,
}

/// DID creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDCreationRequest {
    pub controller: String,
    pub public_key: String,
    pub service_endpoints: Vec<Service>,
    pub metadata: BiometadataExtension,
}

/// DID update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDUpdateRequest {
    pub controller: Option<String>,
    pub add_verification_method: Option<Vec<VerificationMethod>>,
    pub remove_verification_method: Option<Vec<String>>,
    pub add_service: Option<Vec<Service>>,
    pub remove_service: Option<Vec<String>>,
    pub update_metadata: Option<BiometadataExtension>,
}

/// Generate a new DID with the bio-did-seq method
pub fn generate_did() -> String {
    let uuid = Uuid::new_v4();
    format!("did:bio:{}", uuid.to_string())
}

/// Create a default DID document structure
pub fn create_default_did_document(
    did: &str,
    controller: &str,
    public_key: &str,
    metadata: BiometadataExtension,
) -> DIDDocument {
    let now = Utc::now();
    let verification_method_id = format!("{}#keys-1", did);

    DIDDocument {
        context: vec![
            "https://www.w3.org/ns/did/v1".to_string(),
            "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
            "https://w3id.org/biodata/v1".to_string(),
        ],
        id: did.to_string(),
        also_known_as: None,
        controller: vec![controller.to_string()],
        verification_method: vec![VerificationMethod {
            id: verification_method_id.clone(),
            controller: did.to_string(),
            vm_type: "Ed25519VerificationKey2020".to_string(),
            public_key_multibase: Some(public_key.to_string()),
            public_key_jwk: None,
        }],
        authentication: vec![verification_method_id],
        assertion_method: None,
        service: vec![Service {
            id: format!("{}#storage", did),
            service_type: "IPFSStorage".to_string(),
            service_endpoint: "https://ipfs.bio-did-seq.example/api".to_string(),
            description: Some("IPFS storage for biological research data".to_string()),
        }],
        created: now,
        updated: now,
        metadata: Some(metadata),
    }
}
