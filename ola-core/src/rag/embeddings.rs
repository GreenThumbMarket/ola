// Embedding generation for RAG

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// Trait for embedding providers
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate an embedding for the given text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Get the dimension of embeddings produced
    fn dimension(&self) -> usize;

    /// Get the model name
    fn model_name(&self) -> &str;
}

/// Available embedding models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingModel {
    /// OpenAI embeddings
    OpenAIAda002,
    OpenAI3Small,
    OpenAI3Large,

    /// Local sentence transformer models
    AllMiniLmL6V2,
    AllMpnetBaseV2,

    /// Use the configured LLM provider's embedding API
    ProviderDefault,
}

impl EmbeddingModel {
    pub fn as_str(&self) -> &str {
        match self {
            Self::OpenAIAda002 => "text-embedding-ada-002",
            Self::OpenAI3Small => "text-embedding-3-small",
            Self::OpenAI3Large => "text-embedding-3-large",
            Self::AllMiniLmL6V2 => "sentence-transformers/all-MiniLM-L6-v2",
            Self::AllMpnetBaseV2 => "sentence-transformers/all-mpnet-base-v2",
            Self::ProviderDefault => "provider-default",
        }
    }

    pub fn dimension(&self) -> usize {
        match self {
            Self::OpenAIAda002 => 1536,
            Self::OpenAI3Small => 1536,
            Self::OpenAI3Large => 3072,
            Self::AllMiniLmL6V2 => 384,
            Self::AllMpnetBaseV2 => 768,
            Self::ProviderDefault => 768, // Default assumption
        }
    }
}

/// Create an embedding provider based on model name
pub fn create_embedding_provider(model_name: &str) -> Result<Box<dyn EmbeddingProvider>> {
    // For now, we'll use the API-based provider
    // In the future, we can add local model support with Candle
    match model_name {
        name if name.starts_with("text-embedding") => {
            Ok(Box::new(OpenAIEmbeddingProvider::new(name)?))
        }
        name if name.starts_with("sentence-transformers") => {
            // For now, fallback to OpenAI
            // TODO: Implement local sentence transformer support
            Ok(Box::new(OpenAIEmbeddingProvider::default()))
        }
        _ => Ok(Box::new(OpenAIEmbeddingProvider::default()))
    }
}

/// OpenAI embedding provider
pub struct OpenAIEmbeddingProvider {
    model: String,
    dimension: usize,
}

impl OpenAIEmbeddingProvider {
    pub fn new(model: &str) -> Result<Self> {
        let dimension = match model {
            "text-embedding-ada-002" => 1536,
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            _ => return Err(anyhow!("Unknown OpenAI embedding model: {}", model)),
        };

        Ok(Self {
            model: model.to_string(),
            dimension,
        })
    }
}

impl Default for OpenAIEmbeddingProvider {
    fn default() -> Self {
        Self {
            model: "text-embedding-3-small".to_string(),
            dimension: 1536,
        }
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Load configuration to get API key
        let config = crate::config::Config::load()?;
        let provider_config = config.get_active_provider()
            .ok_or_else(|| anyhow!("No active provider configured"))?;

        if provider_config.provider != "OpenAI" {
            return Err(anyhow!("OpenAI provider not configured"));
        }

        let api_key = if provider_config.api_key.is_empty() {
            std::env::var("OPENAI_API_KEY")
                .map_err(|_| anyhow!("OpenAI API key not found"))?
        } else {
            provider_config.api_key.clone()
        };

        // Make API request
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "input": text,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }

        let response_json: serde_json::Value = response.json().await?;

        // Extract embedding from response
        let embedding = response_json
            .get("data")
            .and_then(|d| d.get(0))
            .and_then(|d| d.get("embedding"))
            .and_then(|e| e.as_array())
            .ok_or_else(|| anyhow!("Invalid response format from OpenAI"))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(embedding)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

/// Simple mock embedding provider for testing
pub struct MockEmbeddingProvider {
    dimension: usize,
}

impl MockEmbeddingProvider {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Generate a deterministic embedding based on text hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        let mut embedding = Vec::with_capacity(self.dimension);
        for i in 0..self.dimension {
            let value = ((hash.wrapping_add(i as u64) % 1000) as f32) / 1000.0;
            embedding.push(value);
        }

        Ok(embedding)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        "mock-embeddings"
    }
}