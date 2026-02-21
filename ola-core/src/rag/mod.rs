// RAG (Retrieval-Augmented Generation) module for Ola
// Provides document loading, embedding, vector search, and query capabilities

pub mod document;
pub mod embeddings;
pub mod vectorstore;
pub mod vectordb;
pub mod chroma;
pub mod query;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub use document::{Document, DocumentLoader};
pub use embeddings::{EmbeddingModel, EmbeddingProvider};
pub use vectorstore::VectorStore;
pub use vectordb::{VectorDB, VectorDBConfig, create_vector_db};
pub use vectordb::SearchResult as VectorDBSearchResult;
pub use query::{RagQuery, QueryResult};

/// Configuration for RAG system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    /// Vector database configuration
    pub vector_db: VectorDBConfig,
    /// Embedding model to use
    pub embedding_model: String,
    /// Number of documents to retrieve for context
    pub top_k: usize,
    /// Similarity threshold for retrieval
    pub similarity_threshold: f32,
    /// Maximum context length in tokens
    pub max_context_tokens: usize,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            vector_db: VectorDBConfig::default(),
            embedding_model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            top_k: 3,
            similarity_threshold: 0.5,
            max_context_tokens: 2000,
        }
    }
}

/// Main RAG system that coordinates document loading, embedding, and retrieval
pub struct RagSystem {
    config: RagConfig,
    vector_db: Box<dyn VectorDB>,
    embedding_provider: Box<dyn EmbeddingProvider>,
    document_loader: DocumentLoader,
}

impl RagSystem {
    /// Create a new RAG system with the given configuration
    pub async fn new(config: RagConfig) -> Result<Self> {
        // Initialize vector database
        let vector_db = create_vector_db(config.vector_db.clone()).await?;

        // Initialize other components
        let embedding_provider = embeddings::create_embedding_provider(&config.embedding_model)?;
        let document_loader = DocumentLoader::new();

        Ok(Self {
            config,
            vector_db,
            embedding_provider,
            document_loader,
        })
    }

    /// Index any file (automatically detects PDF or text)
    pub async fn index_file(&mut self, file_path: &Path) -> Result<String> {
        // Detect file type based on extension
        let extension = file_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "pdf" => self.index_pdf(file_path).await,
            _ => self.index_text(file_path).await,
        }
    }

    /// Load and index a PDF document
    pub async fn index_pdf(&mut self, pdf_path: &Path) -> Result<String> {
        // Load the PDF
        let documents = self.document_loader.load_pdf(pdf_path)?;

        // Generate embeddings for each document chunk
        let mut embeddings = Vec::new();
        for doc in &documents {
            let embedding = self.embedding_provider.embed(&doc.content).await?;
            embeddings.push(embedding);
        }

        // Add all documents to vector db
        let doc_ids = self.vector_db.add_documents(documents, embeddings).await?;

        // Save the updated index
        self.vector_db.save().await?;

        Ok(format!("Indexed {} document chunks from {} (PDF)",
                   doc_ids.len(),
                   pdf_path.display()))
    }

    /// Load and index a text file
    pub async fn index_text(&mut self, text_path: &Path) -> Result<String> {
        let documents = self.document_loader.load_text(text_path)?;

        // Generate embeddings for each document chunk
        let mut embeddings = Vec::new();
        for doc in &documents {
            let embedding = self.embedding_provider.embed(&doc.content).await?;
            embeddings.push(embedding);
        }

        // Add all documents to vector db
        let doc_ids = self.vector_db.add_documents(documents, embeddings).await?;

        // Save the updated index
        self.vector_db.save().await?;

        Ok(format!("Indexed {} document chunks from {} (Text)",
                   doc_ids.len(),
                   text_path.display()))
    }

    /// Query the RAG system
    pub async fn query(&self, query_text: &str) -> Result<QueryResult> {
        // Generate embedding for the query
        let query_embedding = self.embedding_provider.embed(query_text).await?;

        // Search for similar documents
        let search_results = self.vector_db.search(
            &query_embedding,
            self.config.top_k,
            self.config.similarity_threshold
        ).await?;

        // Build context from retrieved documents
        let context = search_results
            .iter()
            .map(|r| r.document.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");

        // Convert VectorDB SearchResults to the format expected by QueryResult
        let retrieved_docs = search_results.iter().map(|r| {
            vectorstore::SearchResult {
                document: r.document.clone(),
                score: r.score,
            }
        }).collect();

        Ok(QueryResult {
            query: query_text.to_string(),
            context,
            retrieved_documents: retrieved_docs,
        })
    }

    /// List all indexed documents
    pub async fn list_documents(&self) -> Result<Vec<String>> {
        self.vector_db.list_documents().await
    }

    /// Clear all indexed documents
    pub async fn clear_index(&mut self) -> Result<()> {
        self.vector_db.clear().await?;
        self.vector_db.save().await?;
        Ok(())
    }

    /// Get statistics about the index
    pub async fn get_stats(&self) -> Result<RagStats> {
        Ok(RagStats {
            total_documents: self.vector_db.document_count().await?,
            index_size_bytes: 0, // ChromaDB doesn't expose index size directly
            embedding_model: self.config.embedding_model.clone(),
            vector_db_type: match &self.config.vector_db {
                VectorDBConfig::InMemory { .. } => "InMemory".to_string(),
                VectorDBConfig::ChromaDB { .. } => "ChromaDB".to_string(),
            },
        })
    }
}

/// Statistics about the RAG system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagStats {
    pub total_documents: usize,
    pub index_size_bytes: u64,
    pub embedding_model: String,
    pub vector_db_type: String,
}