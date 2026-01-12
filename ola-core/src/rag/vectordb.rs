// Unified Vector Database abstraction for RAG

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::rag::document::Document;

/// Configuration for vector database backends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorDBConfig {
    /// In-memory HNSW vector store (default)
    InMemory {
        storage_path: String,
    },
    /// ChromaDB vector database
    ChromaDB {
        host: String,
        port: u16,
        collection_name: String,
        persist_directory: Option<String>,
    },
}

impl Default for VectorDBConfig {
    fn default() -> Self {
        VectorDBConfig::InMemory {
            storage_path: std::env::var("HOME")
                .map(|h| format!("{}/.ola/rag", h))
                .unwrap_or_else(|_| ".ola/rag".to_string()),
        }
    }
}

/// Search result from vector database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: Document,
    pub score: f32,
    pub distance: f32,
}

/// Trait for vector database operations
#[async_trait]
pub trait VectorDB: Send + Sync {
    /// Initialize the vector database
    async fn init(&mut self) -> Result<()>;

    /// Add a document with its embedding
    async fn add_document(&mut self, document: Document, embedding: Vec<f32>) -> Result<String>;

    /// Add multiple documents with embeddings
    async fn add_documents(&mut self, documents: Vec<Document>, embeddings: Vec<Vec<f32>>) -> Result<Vec<String>>;

    /// Search for similar documents
    async fn search(&self, query_embedding: &[f32], top_k: usize, threshold: f32) -> Result<Vec<SearchResult>>;

    /// Get a document by ID
    async fn get_document(&self, doc_id: &str) -> Result<Option<Document>>;

    /// Delete a document by ID
    async fn delete_document(&mut self, doc_id: &str) -> Result<bool>;

    /// Clear all documents
    async fn clear(&mut self) -> Result<()>;

    /// List all document IDs
    async fn list_documents(&self) -> Result<Vec<String>>;

    /// Get total document count
    async fn document_count(&self) -> Result<usize>;

    /// Save/persist the database
    async fn save(&self) -> Result<()>;

    /// Load/restore the database
    async fn load(&mut self) -> Result<()>;
}

/// Factory function to create appropriate VectorDB implementation
pub async fn create_vector_db(config: VectorDBConfig) -> Result<Box<dyn VectorDB>> {
    match config {
        VectorDBConfig::InMemory { storage_path } => {
            let mut db = super::vectorstore::VectorStore::new(&storage_path)?;
            db.init().await?;
            Ok(Box::new(db))
        }
        VectorDBConfig::ChromaDB { host, port, collection_name, persist_directory } => {
            let mut db = super::chroma::ChromaClient::new(
                &host,
                port,
                &collection_name,
                persist_directory.as_deref(),
            )?;
            db.init().await?;
            Ok(Box::new(db))
        }
    }
}