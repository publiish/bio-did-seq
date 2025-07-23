use serde::{Deserialize, Serialize};

/// Post-Quantum Safe Auth Token Header.
#[derive(Serialize, Deserialize)]
pub struct TokenHeader {
    pub alg: String,
    pub typ: String,
    pub nonce: String,
}

/// PQS Token Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    // PQC signature
    pub signature: Vec<u8>,
    // Issued at timestamp
    pub iat: usize,
    pub nonce: String,
}

/// Auth Response containing PQS token
#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}

/// User authentication model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: i64,
    pub id: i64,
    pub username: String,
    pub roles: Vec<String>,
}

impl AuthUser {
    #[allow(dead_code)]
    pub fn new(user_id: i64, username: String, roles: Vec<String>) -> Self {
        Self {
            user_id,
            // Set both user_id and id to the same value for compatibility
            id: user_id,
            username,
            roles,
        }
    }

    #[allow(dead_code)]
    pub fn is_admin(&self) -> bool {
        self.roles.contains(&"admin".to_string())
    }
}

/// Login request model
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// API key request model
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiKeyRequest {
    pub name: String,
    pub expires_in_days: Option<i32>,
}

/// Login response model
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: AuthUser,
}

/// API key response model
#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub key: String,
    pub expires_at: String,
}
