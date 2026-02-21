// Document loading and processing for RAG

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use chrono::{DateTime, Utc};

/// Represents a document chunk for indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: DocumentMetadata,
}

/// Metadata associated with a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub source: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub created_at: DateTime<Utc>,
    pub file_type: String,
}

/// Document loader for various file formats
pub struct DocumentLoader {
    chunk_size: usize,
    chunk_overlap: usize,
}

impl DocumentLoader {
    /// Create a new document loader
    pub fn new() -> Self {
        Self {
            chunk_size: 1000,  // Characters per chunk
            chunk_overlap: 200, // Overlap between chunks
        }
    }

    /// Load and chunk a PDF file
    pub fn load_pdf(&self, pdf_path: &Path) -> Result<Vec<Document>> {
        if !pdf_path.exists() {
            return Err(anyhow!("PDF file does not exist: {}", pdf_path.display()));
        }

        // Extract text from PDF
        let text = pdf_extract::extract_text(pdf_path)
            .map_err(|e| anyhow!("Failed to extract text from PDF: {}", e))?;

        // Split into chunks
        let chunks = self.split_text(&text);

        // Create documents from chunks
        let source = pdf_path.to_string_lossy().to_string();
        let documents = chunks
            .into_iter()
            .enumerate()
            .map(|(idx, content)| {
                Document {
                    id: format!("{}-chunk-{}",
                               uuid::Uuid::new_v4().to_string(),
                               idx),
                    content,
                    metadata: DocumentMetadata {
                        source: source.clone(),
                        chunk_index: idx,
                        total_chunks: 0, // Will be updated
                        created_at: Utc::now(),
                        file_type: "pdf".to_string(),
                    },
                }
            })
            .collect::<Vec<_>>();

        // Update total_chunks in metadata
        let total = documents.len();
        let documents = documents
            .into_iter()
            .map(|mut doc| {
                doc.metadata.total_chunks = total;
                doc
            })
            .collect();

        Ok(documents)
    }

    /// Load and chunk a text file
    pub fn load_text(&self, text_path: &Path) -> Result<Vec<Document>> {
        if !text_path.exists() {
            return Err(anyhow!("Text file does not exist: {}", text_path.display()));
        }

        let text = fs::read_to_string(text_path)?;
        let chunks = self.split_text(&text);

        let source = text_path.to_string_lossy().to_string();
        let file_extension = text_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("txt");

        let documents = chunks
            .into_iter()
            .enumerate()
            .map(|(idx, content)| {
                Document {
                    id: format!("{}-chunk-{}",
                               uuid::Uuid::new_v4().to_string(),
                               idx),
                    content,
                    metadata: DocumentMetadata {
                        source: source.clone(),
                        chunk_index: idx,
                        total_chunks: 0, // Will be updated
                        created_at: Utc::now(),
                        file_type: file_extension.to_string(),
                    },
                }
            })
            .collect::<Vec<_>>();

        // Update total_chunks
        let total = documents.len();
        let documents = documents
            .into_iter()
            .map(|mut doc| {
                doc.metadata.total_chunks = total;
                doc
            })
            .collect();

        Ok(documents)
    }

    /// Load content from a string
    pub fn load_from_string(&self, content: &str, source: &str) -> Result<Vec<Document>> {
        let chunks = self.split_text(content);

        let documents = chunks
            .into_iter()
            .enumerate()
            .map(|(idx, content)| {
                Document {
                    id: format!("{}-chunk-{}",
                               uuid::Uuid::new_v4().to_string(),
                               idx),
                    content,
                    metadata: DocumentMetadata {
                        source: source.to_string(),
                        chunk_index: idx,
                        total_chunks: 0, // Will be updated
                        created_at: Utc::now(),
                        file_type: "text".to_string(),
                    },
                }
            })
            .collect::<Vec<_>>();

        // Update total_chunks
        let total = documents.len();
        let documents = documents
            .into_iter()
            .map(|mut doc| {
                doc.metadata.total_chunks = total;
                doc
            })
            .collect();

        Ok(documents)
    }

    /// Split text into overlapping chunks
    fn split_text(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();

        if chars.len() <= self.chunk_size {
            // Text is smaller than chunk size, return as single chunk
            return vec![text.to_string()];
        }

        let mut start = 0;
        while start < chars.len() {
            let end = std::cmp::min(start + self.chunk_size, chars.len());
            let chunk: String = chars[start..end].iter().collect();
            chunks.push(chunk);

            if end >= chars.len() {
                break;
            }

            // Move start position with overlap
            start = end - self.chunk_overlap;
        }

        chunks
    }

    /// Set custom chunk size
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Set custom chunk overlap
    pub fn with_overlap(mut self, overlap: usize) -> Self {
        self.chunk_overlap = overlap;
        self
    }
}