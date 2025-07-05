use actix_web::{middleware as actix_middleware, App, HttpServer};
use base64::engine::general_purpose::STANDARD as Base64Engine;
use base64::Engine;
use clap::{Parser, Subcommand};
use env_logger::Env;
use std::io;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use std::env;

mod config;
mod database;
mod errors;
mod middleware;
mod models;
mod routes;
mod services;
mod stream;
mod utils;

use config::Config;
use middleware::rate_limiter::UserRateLimiter;
use services::ipfs_service::IPFSService;
use services::did_service::DIDService;
use services::bioagents_service::BioAgentsService;
use services::dataverse_service::DataverseService;
use services::ucan_service::UcanService;

// Post-quantum crypto imports
use pqcrypto_dilithium::dilithium5;
use pqcrypto_kyber::kyber1024;
use pqcrypto_traits::kem::{PublicKey as KemPublicKey, SecretKey as KemSecretKey};
use pqcrypto_traits::sign::{PublicKey as SignPublicKey, SecretKey as SignSecretKey};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Generate post-quantum cryptographic keys
#[derive(Subcommand)]
enum Commands {
    /// cargo run -- generate-keys
    /// cargo run -- generate-keys --output /path/to/keys
    GenerateKeys {
        /// Output directory for keys (default: current directory)
        #[arg(short, long, default_value = ".")]
        output: String,
    },
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::GenerateKeys { output }) => {
            generate_keys(&output)?;
            Ok(())
        }
        None => start_server().await,
    }
}

fn generate_keys(output_dir: &str) -> io::Result<()> {
    // Ensure output directory exists
    std::fs::create_dir_all(output_dir)?;

    // Generate Kyber1024 KEM keys
    let (pk_kem, sk_kem) = kyber1024::keypair();
    // Generate Dilithium5 signature keys
    let (pk_sign, sk_sign) = dilithium5::keypair();

    // Save keys in Base64 for consistency and portability
    let pk_kem_b64 = Base64Engine.encode(pk_kem.as_bytes());
    let sk_kem_b64 = Base64Engine.encode(sk_kem.as_bytes());
    let pk_sign_b64 = Base64Engine.encode(pk_sign.as_bytes());
    let sk_sign_b64 = Base64Engine.encode(sk_sign.as_bytes());

    // Save Base64 encoded keys
    std::fs::write(format!("{}/kyber1024_public.key", output_dir), pk_kem_b64)?;
    std::fs::write(format!("{}/kyber1024_secret.key", output_dir), sk_kem_b64)?;
    std::fs::write(format!("{}/dilithium5_public.key", output_dir), pk_sign_b64)?;
    std::fs::write(format!("{}/dilithium5_secret.key", output_dir), sk_sign_b64)?;

    log::info!(
        "Base64-encoded keys generated successfully in {}:",
        output_dir
    );
    println!("- kyber1024_public.key (KEM public key)");
    println!("- kyber1024_secret.key (KEM secret key)");
    println!("- dilithium5_public.key (Signature public key)");
    println!("- dilithium5_secret.key (Signature secret key)");

    Ok(())
}

async fn start_server() -> io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let config = Config::from_env().map_err(|e| {
        log::error!("Failed to load configuration: {}", e);
        io::Error::new(io::ErrorKind::Other, "Configuration loading failed")
    })?;

    // Initialize IPFS service
    let ipfs_service = IPFSService::new(&config).await.map_err(|e| {
        log::error!("Failed to initialize IPFS service: {}", e);
        io::Error::new(io::ErrorKind::Other, "IPFS service initialization failed")
    })?;
    let ipfs_service = Arc::new(ipfs_service);
    
    // Initialize database connection pool
    let db_pool = database::init_db_pool(&config.database_url).await.map_err(|e| {
        log::error!("Failed to initialize database pool: {}", e);
        io::Error::new(io::ErrorKind::Other, "Database initialization failed")
    })?;
    let db_pool = Arc::new(db_pool);

    // Initialize DID service
    let did_service = DIDService::new(db_pool.clone(), ipfs_service.clone());
    let did_service = Arc::new(did_service);
    
    // Initialize BioAgents service
    let bioagents_service = BioAgentsService::new(
        &env::var("BIOAGENTS_API_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
        &env::var("BIOAGENTS_API_KEY").unwrap_or_else(|_| "default-api-key".to_string())
    );
    let bioagents_service = Arc::new(bioagents_service);
    
    // Initialize Dataverse service
    let dataverse_service = DataverseService::new(
        &env::var("DATAVERSE_API_URL").unwrap_or_else(|_| "https://dataverse.harvard.edu/api".to_string()),
        &env::var("DATAVERSE_API_KEY").unwrap_or_else(|_| "".to_string())
    );
    let dataverse_service = Arc::new(dataverse_service);
    
    // Initialize UCAN service
    let ucan_service = UcanService::new(db_pool.clone()).await.map_err(|e| {
        log::error!("Failed to initialize UCAN service: {}", e);
        io::Error::new(io::ErrorKind::Other, "UCAN service initialization failed")
    })?;
    let ucan_service = Arc::new(ucan_service);

    // Create app state
    let app_state = routes::AppState {
        ipfs_service: ipfs_service.clone(),
        did_service: did_service.clone(),
        bioagents_service: bioagents_service.clone(),
        dataverse_service: dataverse_service.clone(),
        ucan_service: ucan_service.clone(),
    };
    
    let rate_limiter = UserRateLimiter::new();

    start_task_cleanup(ipfs_service.clone());

    let bind_address = config.bind_address.clone();
    log::info!("Starting server at {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(app_state.clone()))
            .wrap(actix_middleware::Logger::default())
            .wrap(rate_limiter.clone())
            .configure(routes::init_routes)
    })
    // Use number of CPUs, capped at 8
    .workers(num_cpus::get().min(8))
    .bind(&bind_address)
    .map_err(|e| {
        log::error!("Failed to bind server to {}: {}", bind_address, e);
        e
    })?
    .run()
    .await
}

/// Spawns a background task to periodically clean up old tasks
fn start_task_cleanup(ipfs_service: Arc<IPFSService>) {
    tokio::spawn(async move {
        // Cleanup every 2 hours
        let mut interval = interval(Duration::from_secs(7200));
        loop {
            interval.tick().await;
            match utils::cleanup_old_tasks(&ipfs_service.db_pool, ipfs_service.tasks.clone()).await
            {
                Ok(()) => log::info!("Task cleanup completed successfully"),
                Err(e) => log::error!("Task cleanup failed: {}", e),
            }
            ipfs_service.cleanup_rate_limiters().await;
        }
    });
}

pub mod crypto_utils {
    use super::*;

    pub fn load_kyber_keys(
        pub_path: &str,
        sec_path: &str,
    ) -> io::Result<(kyber1024::PublicKey, kyber1024::SecretKey)> {
        let pk_data = std::fs::read(pub_path)?;
        let sk_data = std::fs::read(sec_path)?;

        let pk_bytes = Base64Engine
            .decode(&pk_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let sk_bytes = Base64Engine
            .decode(&sk_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let pk = kyber1024::PublicKey::from_bytes(&pk_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let sk = kyber1024::SecretKey::from_bytes(&sk_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok((pk, sk))
    }

    pub fn load_dilithium_keys(
        pub_path: &str,
        sec_path: &str,
    ) -> io::Result<(dilithium5::PublicKey, dilithium5::SecretKey)> {
        let pk_data = std::fs::read(pub_path)?;
        let sk_data = std::fs::read(sec_path)?;

        let pk_bytes = Base64Engine
            .decode(&pk_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let sk_bytes = Base64Engine
            .decode(&sk_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let pk = dilithium5::PublicKey::from_bytes(&pk_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let sk = dilithium5::SecretKey::from_bytes(&sk_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok((pk, sk))
    }
}
