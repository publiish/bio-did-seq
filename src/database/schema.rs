use log::info;
use mysql_async::{prelude::*, Pool};

/// Initializes the database schema by creating necessary tables if they don't exist
pub async fn init_schema(pool: &Pool) -> Result<(), mysql_async::Error> {
    let mut conn = pool.get_conn().await?;

    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS users (
            id INT PRIMARY KEY AUTO_INCREMENT,
            username VARCHAR(50) NOT NULL UNIQUE,
            email VARCHAR(100) NOT NULL UNIQUE,
            password_hash VARCHAR(255) NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_email (email)
        )",
    )
    .await?;

    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS file_metadata (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            cid VARCHAR(100) NOT NULL UNIQUE,
            name VARCHAR(255) NOT NULL,
            size BIGINT NOT NULL,
            timestamp DATETIME NOT NULL,
            user_id INT NOT NULL,
            task_id VARCHAR(36),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            INDEX idx_cid (cid),
            INDEX idx_user_id (user_id),
            INDEX idx_task_id (task_id)
        )",
    )
    .await?;

    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS upload_tasks (
            task_id VARCHAR(36) PRIMARY KEY,
            user_id INT NOT NULL,
            status VARCHAR(20) NOT NULL,
            cid VARCHAR(100),
            error TEXT,
            progress DOUBLE DEFAULT 0.0,
            started_at DATETIME NOT NULL,
            completed_at DATETIME,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            INDEX idx_user_id (user_id)
        )",
    )
    .await?;
    
    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS did_documents (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            did VARCHAR(255) NOT NULL UNIQUE,
            cid VARCHAR(100) NOT NULL,
            user_id INT NOT NULL,
            dataverse_doi VARCHAR(255),
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            INDEX idx_did (did),
            INDEX idx_cid (cid),
            INDEX idx_user_id (user_id)
        )",
    )
    .await?;
    
    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS ucan_tokens (
            id VARCHAR(36) PRIMARY KEY,
            user_id INT NOT NULL,
            token TEXT NOT NULL,
            audience_did VARCHAR(255) NOT NULL,
            issued_at DATETIME NOT NULL,
            expires_at DATETIME NOT NULL,
            revoked BOOLEAN DEFAULT FALSE,
            revoked_at DATETIME,
            delegated_from VARCHAR(255),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            INDEX idx_user_id (user_id),
            INDEX idx_audience (audience_did),
            INDEX idx_delegated_from (delegated_from)
        )",
    )
    .await?;

    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS research_papers (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            title VARCHAR(255) NOT NULL,
            authors JSON NOT NULL,
            abstract_text TEXT,
            doi VARCHAR(100),
            publication_date VARCHAR(50),
            journal VARCHAR(255),
            keywords JSON,
            cid VARCHAR(100) NOT NULL,
            did VARCHAR(255) NOT NULL,
            biological_entities JSON,
            knowledge_graph_cid VARCHAR(100),
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            user_id INT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (did) REFERENCES did_documents(did) ON DELETE CASCADE,
            INDEX idx_cid (cid),
            INDEX idx_did (did),
            INDEX idx_doi (doi),
            INDEX idx_user_id (user_id)
        )",
    )
    .await?;

    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS bioagent_tasks (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            task_id VARCHAR(100) NOT NULL UNIQUE,
            user_id INT NOT NULL,
            cid VARCHAR(100) NOT NULL,
            status VARCHAR(20) NOT NULL,
            progress FLOAT DEFAULT 0.0,
            result_cid VARCHAR(100),
            created_at DATETIME NOT NULL,
            completed_at DATETIME,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            INDEX idx_task_id (task_id),
            INDEX idx_user_id (user_id),
            INDEX idx_cid (cid)
        )",
    )
    .await?;

    info!("Database schema initialized");
    Ok(())
}
