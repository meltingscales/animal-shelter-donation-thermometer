use async_trait::async_trait;
use firestore::*;
use std::sync::Arc;

use crate::ThermometerConfig;

const COLLECTION_NAME: &str = "thermometer_configs";
const CONFIG_DOC_ID: &str = "current_config";

#[derive(Debug)]
pub enum StorageError {
    Firestore(String),
    NotFound,
    Serialization(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Firestore(msg) => write!(f, "Firestore error: {}", msg),
            StorageError::NotFound => write!(f, "Configuration not found"),
            StorageError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

#[async_trait]
pub trait ConfigStorage: Send + Sync {
    async fn load_config(&self) -> Result<ThermometerConfig, StorageError>;
    async fn save_config(&self, config: &ThermometerConfig) -> Result<(), StorageError>;
}

/// Firestore-based persistent storage
pub struct FirestoreStorage {
    db: FirestoreDb,
}

impl FirestoreStorage {
    pub async fn new(project_id: String) -> Result<Self, StorageError> {
        tracing::info!("Initializing Firestore storage for project: {}", project_id);

        let db = FirestoreDb::new(project_id)
            .await
            .map_err(|e| StorageError::Firestore(format!("Failed to initialize Firestore: {}", e)))?;

        tracing::info!("Firestore storage initialized successfully");
        Ok(Self { db })
    }
}

#[async_trait]
impl ConfigStorage for FirestoreStorage {
    async fn load_config(&self) -> Result<ThermometerConfig, StorageError> {
        tracing::debug!("Loading config from Firestore");

        let result: Option<ThermometerConfig> = self.db
            .fluent()
            .select()
            .by_id_in(COLLECTION_NAME)
            .obj()
            .one(CONFIG_DOC_ID)
            .await
            .map_err(|e| StorageError::Firestore(format!("Failed to read from Firestore: {}", e)))?;

        match result {
            Some(config) => {
                tracing::debug!("Config loaded successfully from Firestore");
                Ok(config)
            }
            None => {
                tracing::debug!("No config found in Firestore, returning default");
                // If no config exists, return default and save it
                let default_config = ThermometerConfig::default();
                self.save_config(&default_config).await?;
                Ok(default_config)
            }
        }
    }

    async fn save_config(&self, config: &ThermometerConfig) -> Result<(), StorageError> {
        tracing::debug!("Saving config to Firestore");

        self.db
            .fluent()
            .update()
            .in_col(COLLECTION_NAME)
            .document_id(CONFIG_DOC_ID)
            .object(config)
            .execute::<()>()
            .await
            .map_err(|e| StorageError::Firestore(format!("Failed to write to Firestore: {}", e)))?;

        tracing::debug!("Config saved successfully to Firestore");
        Ok(())
    }
}

/// In-memory storage (fallback when Firestore is not available)
pub struct InMemoryStorage {
    config: tokio::sync::RwLock<ThermometerConfig>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        tracing::info!("Using in-memory storage (data will not persist)");
        Self {
            config: tokio::sync::RwLock::new(ThermometerConfig::default()),
        }
    }
}

#[async_trait]
impl ConfigStorage for InMemoryStorage {
    async fn load_config(&self) -> Result<ThermometerConfig, StorageError> {
        Ok(self.config.read().await.clone())
    }

    async fn save_config(&self, config: &ThermometerConfig) -> Result<(), StorageError> {
        let mut stored_config = self.config.write().await;
        *stored_config = config.clone();
        Ok(())
    }
}

/// Create storage backend based on environment configuration
pub async fn create_storage() -> Arc<dyn ConfigStorage> {
    // Try to get GCP project ID from environment
    if let Ok(project_id) = std::env::var("GCP_PROJECT") {
        tracing::info!("GCP_PROJECT found: {}, attempting to use Firestore", project_id);

        match FirestoreStorage::new(project_id).await {
            Ok(storage) => {
                tracing::info!("Successfully initialized Firestore storage");
                return Arc::new(storage);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize Firestore: {}. Falling back to in-memory storage.", e);
            }
        }
    } else {
        tracing::info!("GCP_PROJECT not set, using in-memory storage");
    }

    Arc::new(InMemoryStorage::new())
}
