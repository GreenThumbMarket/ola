// ChromaDB client implementation for vector storage

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;

use crate::rag::document::Document;
use super::vectordb::{VectorDB, SearchResult};

/// ChromaDB client wrapper (uses HTTP API directly)
pub struct ChromaClient {
    host: String,
    port: u16,
    collection_name: String,
    persist_directory: Option<String>,
    base_url: String,
}

impl ChromaClient {
    /// Create a new ChromaDB client
    pub fn new(host: &str, port: u16, collection_name: &str, persist_directory: Option<&str>) -> Result<Self> {
        let base_url = format!("http://{}:{}", host, port);
        Ok(Self {
            host: host.to_string(),
            port,
            collection_name: collection_name.to_string(),
            persist_directory: persist_directory.map(|s| s.to_string()),
            base_url,
        })
    }

    /// Connect to ChromaDB server and ensure collection exists
    async fn ensure_collection(&self) -> Result<()> {
        // Note: This is a simplified implementation
        // In a production environment, you would use proper ChromaDB client
        // For now, we'll just return Ok as a placeholder
        Ok(())
    }
}

#[async_trait]
impl VectorDB for ChromaClient {
    async fn init(&mut self) -> Result<()> {
        self.connect().await
    }

    async fn add_document(&mut self, document: Document, embedding: Vec<f32>) -> Result<String> {
        self.connect().await?;

        let collection = self.collection.as_ref()
            .ok_or_else(|| anyhow!("ChromaDB collection not initialized"))?;

        let metadata = HashMap::from([
            ("source".to_string(), json!(document.metadata.source)),
            ("chunk_index".to_string(), json!(document.metadata.chunk_index)),
            ("total_chunks".to_string(), json!(document.metadata.total_chunks)),
            ("file_type".to_string(), json!(document.metadata.file_type)),
            ("created_at".to_string(), json!(document.metadata.created_at.to_rfc3339())),
        ]);

        let result = collection.add(
            CollectionEntries {
                ids: vec![document.id.clone()],
                embeddings: Some(vec![embedding]),
                metadatas: Some(vec![metadata]),
                documents: Some(vec![document.content.clone()]),
            },
            None
        ).await?;

        Ok(document.id)
    }

    async fn add_documents(&mut self, documents: Vec<Document>, embeddings: Vec<Vec<f32>>) -> Result<Vec<String>> {
        self.connect().await?;

        if documents.len() != embeddings.len() {
            return Err(anyhow!("Number of documents must match number of embeddings"));
        }

        let collection = self.collection.as_ref()
            .ok_or_else(|| anyhow!("ChromaDB collection not initialized"))?;

        let ids: Vec<String> = documents.iter().map(|d| d.id.clone()).collect();
        let contents: Vec<String> = documents.iter().map(|d| d.content.clone()).collect();

        let metadatas: Vec<HashMap<String, serde_json::Value>> = documents.iter().map(|doc| {
            HashMap::from([
                ("source".to_string(), json!(doc.metadata.source)),
                ("chunk_index".to_string(), json!(doc.metadata.chunk_index)),
                ("total_chunks".to_string(), json!(doc.metadata.total_chunks)),
                ("file_type".to_string(), json!(doc.metadata.file_type)),
                ("created_at".to_string(), json!(doc.metadata.created_at.to_rfc3339())),
            ])
        }).collect();

        collection.add(
            CollectionEntries {
                ids: ids.clone(),
                embeddings: Some(embeddings),
                metadatas: Some(metadatas),
                documents: Some(contents),
            },
            None
        ).await?;

        Ok(ids)
    }

    async fn search(&self, query_embedding: &[f32], top_k: usize, threshold: f32) -> Result<Vec<SearchResult>> {
        let collection = self.collection.as_ref()
            .ok_or_else(|| anyhow!("ChromaDB collection not initialized"))?;

        let results = collection.query(
            QueryOptions {
                query_embeddings: Some(vec![query_embedding.to_vec()]),
                n_results: Some(top_k),
                ..Default::default()
            },
            None
        ).await?;

        let mut search_results = Vec::new();

        if let (Some(ids), Some(documents), Some(metadatas), Some(distances)) =
            (results.ids.get(0), results.documents.as_ref().and_then(|d| d.get(0)),
             results.metadatas.as_ref().and_then(|m| m.get(0)), results.distances.as_ref().and_then(|d| d.get(0))) {

            for i in 0..ids.len() {
                let distance = distances.get(i).copied().unwrap_or(1.0) as f32;
                let score = 1.0 / (1.0 + distance); // Convert distance to similarity score

                if score >= threshold {
                    if let (Some(doc_content), Some(metadata)) =
                        (documents.as_ref().and_then(|docs| docs.get(i)),
                         metadatas.as_ref().and_then(|metas| metas.get(i))) {

                        let document = Document {
                            id: ids[i].clone(),
                            content: doc_content.clone().unwrap_or_default(),
                            metadata: crate::rag::document::DocumentMetadata {
                                source: metadata.get("source")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                chunk_index: metadata.get("chunk_index")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0) as usize,
                                total_chunks: metadata.get("total_chunks")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0) as usize,
                                created_at: metadata.get("created_at")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                                    .unwrap_or_else(chrono::Utc::now),
                                file_type: metadata.get("file_type")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown")
                                    .to_string(),
                            },
                        };

                        search_results.push(SearchResult {
                            document,
                            score,
                            distance,
                        });
                    }
                }
            }
        }

        Ok(search_results)
    }

    async fn get_document(&self, doc_id: &str) -> Result<Option<Document>> {
        let collection = self.collection.as_ref()
            .ok_or_else(|| anyhow!("ChromaDB collection not initialized"))?;

        let result = collection.get(GetQuery {
            ids: Some(vec![doc_id.to_string()]),
            ..Default::default()
        }, None).await?;

        if let (Some(ids), Some(documents), Some(metadatas)) =
            (result.ids.first(), result.documents.as_ref().and_then(|d| d.first()),
             result.metadatas.as_ref().and_then(|m| m.first())) {

            if let (Some(content), Some(metadata)) = (documents.first(), metadatas.first()) {
                let document = Document {
                    id: doc_id.to_string(),
                    content: content.clone().unwrap_or_default(),
                    metadata: crate::rag::document::DocumentMetadata {
                        source: metadata.get("source")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        chunk_index: metadata.get("chunk_index")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize,
                        total_chunks: metadata.get("total_chunks")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize,
                        created_at: metadata.get("created_at")
                            .and_then(|v| v.as_str())
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(chrono::Utc::now),
                        file_type: metadata.get("file_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                    },
                };

                return Ok(Some(document));
            }
        }

        Ok(None)
    }

    async fn delete_document(&mut self, doc_id: &str) -> Result<bool> {
        let collection = self.collection.as_mut()
            .ok_or_else(|| anyhow!("ChromaDB collection not initialized"))?;

        collection.delete(None, Some(vec![doc_id.to_string()]), None).await?;
        Ok(true)
    }

    async fn clear(&mut self) -> Result<()> {
        self.connect().await?;

        if let Some(client) = &self.client {
            // Delete and recreate the collection
            client.delete_collection(&self.collection_name).await?;
            let new_collection = client.create_collection(&self.collection_name, None).await?;
            self.collection = Some(new_collection);
        }

        Ok(())
    }

    async fn list_documents(&self) -> Result<Vec<String>> {
        let collection = self.collection.as_ref()
            .ok_or_else(|| anyhow!("ChromaDB collection not initialized"))?;

        let result = collection.get(GetQuery::default(), None).await?;
        Ok(result.ids)
    }

    async fn document_count(&self) -> Result<usize> {
        let collection = self.collection.as_ref()
            .ok_or_else(|| anyhow!("ChromaDB collection not initialized"))?;

        let count = collection.count().await?;
        Ok(count)
    }

    async fn save(&self) -> Result<()> {
        // ChromaDB automatically persists if persist_directory is set
        Ok(())
    }

    async fn load(&mut self) -> Result<()> {
        self.connect().await
    }
}