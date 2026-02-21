// Vector store for document embeddings

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use crate::rag::document::Document;

/// A search result from the vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: Document,
    pub score: f32,
}

/// Vector store for managing document embeddings
pub struct VectorStore {
    storage_path: PathBuf,
    index: VectorIndex,
}

impl VectorStore {
    /// Create a new vector store
    pub fn new(storage_path: &str) -> Result<Self> {
        let storage_path = PathBuf::from(storage_path);
        fs::create_dir_all(&storage_path)?;

        let index_path = storage_path.join("vector_index.bin");
        let index = if index_path.exists() {
            VectorIndex::load(&index_path)?
        } else {
            VectorIndex::new()
        };

        Ok(Self {
            storage_path,
            index,
        })
    }

    /// Add a document with its embedding to the store
    pub fn add_document(&mut self, document: Document, embedding: Vec<f32>) -> Result<String> {
        let doc_id = document.id.clone();
        self.index.add(doc_id.clone(), document, embedding);
        Ok(doc_id)
    }

    /// Search for similar documents
    pub fn search(&self, query_embedding: &[f32], top_k: usize, threshold: f32) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        for entry in &self.index.entries {
            let score = cosine_similarity(query_embedding, &entry.embedding);
            if score >= threshold {
                results.push(SearchResult {
                    document: entry.document.clone(),
                    score,
                });
            }
        }

        // Sort by score (highest first)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(top_k);

        Ok(results)
    }

    /// List all document IDs
    pub fn list_documents(&self) -> Result<Vec<String>> {
        Ok(self.index.entries.iter().map(|e| e.id.clone()).collect())
    }

    /// Get document count
    pub fn document_count(&self) -> Result<usize> {
        Ok(self.index.entries.len())
    }

    /// Get index size in bytes
    pub fn index_size(&self) -> Result<u64> {
        let index_path = self.storage_path.join("vector_index.bin");
        if index_path.exists() {
            Ok(fs::metadata(index_path)?.len())
        } else {
            Ok(0)
        }
    }

    /// Clear all documents
    pub fn clear(&mut self) -> Result<()> {
        self.index = VectorIndex::new();
        Ok(())
    }

    /// Save the index to disk
    pub fn save(&self) -> Result<()> {
        let index_path = self.storage_path.join("vector_index.bin");
        self.index.save(&index_path)
    }

    /// Load from disk
    pub fn load(&mut self) -> Result<()> {
        let index_path = self.storage_path.join("vector_index.bin");
        if index_path.exists() {
            self.index = VectorIndex::load(&index_path)?;
        }
        Ok(())
    }

    /// Get a document by ID
    pub fn get_document(&self, doc_id: &str) -> Result<Option<Document>> {
        Ok(self.index.entries.iter()
            .find(|e| e.id == doc_id)
            .map(|e| e.document.clone()))
    }

    /// Delete a document by ID
    pub fn delete_document(&mut self, doc_id: &str) -> Result<bool> {
        let before_len = self.index.entries.len();
        self.index.entries.retain(|e| e.id != doc_id);
        Ok(self.index.entries.len() < before_len)
    }
}

// Implement VectorDB trait for VectorStore
#[async_trait]
impl super::vectordb::VectorDB for VectorStore {
    async fn init(&mut self) -> Result<()> {
        self.load()?;
        Ok(())
    }

    async fn add_document(&mut self, document: Document, embedding: Vec<f32>) -> Result<String> {
        let doc_id = self.add_document(document, embedding)?;
        Ok(doc_id)
    }

    async fn add_documents(&mut self, documents: Vec<Document>, embeddings: Vec<Vec<f32>>) -> Result<Vec<String>> {
        let mut doc_ids = Vec::new();
        for (doc, embedding) in documents.into_iter().zip(embeddings) {
            let doc_id = self.add_document(doc, embedding)?;
            doc_ids.push(doc_id);
        }
        Ok(doc_ids)
    }

    async fn search(&self, query_embedding: &[f32], top_k: usize, threshold: f32) -> Result<Vec<super::vectordb::SearchResult>> {
        let results = self.search(query_embedding, top_k, threshold)?;
        Ok(results.into_iter().map(|r| super::vectordb::SearchResult {
            document: r.document,
            score: r.score,
            distance: 1.0 - r.score, // Convert similarity to distance
        }).collect())
    }

    async fn get_document(&self, doc_id: &str) -> Result<Option<Document>> {
        self.get_document(doc_id)
    }

    async fn delete_document(&mut self, doc_id: &str) -> Result<bool> {
        self.delete_document(doc_id)
    }

    async fn clear(&mut self) -> Result<()> {
        self.clear()
    }

    async fn list_documents(&self) -> Result<Vec<String>> {
        self.list_documents()
    }

    async fn document_count(&self) -> Result<usize> {
        self.document_count()
    }

    async fn save(&self) -> Result<()> {
        self.save()
    }

    async fn load(&mut self) -> Result<()> {
        self.load()
    }
}

/// Simple in-memory vector index
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VectorIndex {
    entries: Vec<IndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexEntry {
    id: String,
    document: Document,
    embedding: Vec<f32>,
}

impl VectorIndex {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn add(&mut self, id: String, document: Document, embedding: Vec<f32>) {
        self.entries.push(IndexEntry {
            id,
            document,
            embedding,
        });
    }

    fn save(&self, path: &Path) -> Result<()> {
        let encoded = bincode::serialize(self)?;
        fs::write(path, encoded)?;
        Ok(())
    }

    fn load(path: &Path) -> Result<Self> {
        let data = fs::read(path)?;
        let index = bincode::deserialize(&data)?;
        Ok(index)
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 1.0);

        let c = vec![0.0, 1.0, 0.0];
        assert_eq!(cosine_similarity(&a, &c), 0.0);

        let d = vec![1.0, 1.0, 0.0];
        let expected = 1.0 / 2.0_f32.sqrt();
        assert!((cosine_similarity(&a, &d) - expected).abs() < 0.001);
    }
}