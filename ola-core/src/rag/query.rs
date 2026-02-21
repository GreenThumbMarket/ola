// Query processing for RAG

use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::rag::vectorstore::SearchResult;
use crate::config::Config;

/// A RAG query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagQuery {
    pub query: String,
    pub max_context_tokens: Option<usize>,
    pub temperature: Option<f32>,
}

/// Result of a RAG query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub query: String,
    pub context: String,
    pub retrieved_documents: Vec<SearchResult>,
}

impl QueryResult {
    /// Generate a response using the configured LLM provider
    pub async fn generate_response(&self) -> Result<String> {
        // Construct the prompt with context
        let prompt = format!(
            "Based on the following context, answer the question.\n\n\
             Context:\n{}\n\n\
             Question: {}\n\n\
             Answer:",
            self.context, self.query
        );

        // Use the configured LLM provider
        let config = Config::load()?;
        let provider_config = config.get_active_provider()
            .ok_or_else(|| anyhow::anyhow!("No active provider configured"))?;

        // Call the appropriate API based on provider
        match provider_config.provider.as_str() {
            "OpenAI" => generate_openai_response(&prompt, &provider_config).await,
            "Anthropic" => generate_anthropic_response(&prompt, &provider_config).await,
            "Gemini" => generate_gemini_response(&prompt, &provider_config).await,
            "Ollama" => generate_ollama_response(&prompt, &provider_config).await,
            _ => Err(anyhow::anyhow!("Unsupported provider: {}", provider_config.provider))
        }
    }

    /// Get a summary of retrieved documents
    pub fn get_sources_summary(&self) -> String {
        if self.retrieved_documents.is_empty() {
            return "No relevant documents found.".to_string();
        }

        let sources: Vec<String> = self.retrieved_documents
            .iter()
            .map(|doc| {
                format!("- {} (score: {:.2})",
                       doc.document.metadata.source,
                       doc.score)
            })
            .collect();

        format!("Retrieved {} documents:\n{}",
                self.retrieved_documents.len(),
                sources.join("\n"))
    }
}

/// Generate response using OpenAI
async fn generate_openai_response(prompt: &str, provider_config: &crate::config::ProviderConfig) -> Result<String> {
    let api_key = if provider_config.api_key.is_empty() {
        std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("OpenAI API key not found"))?
    } else {
        provider_config.api_key.clone()
    };

    let model = provider_config.model.as_ref()
        .unwrap_or(&"gpt-4".to_string())
        .clone();

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": "You are a helpful assistant that answers questions based on the provided context."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.7,
            "max_tokens": 1000,
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
    }

    let response_json: serde_json::Value = response.json().await?;

    let content = response_json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid response format from OpenAI"))?;

    Ok(content.to_string())
}

/// Generate response using Anthropic
async fn generate_anthropic_response(prompt: &str, provider_config: &crate::config::ProviderConfig) -> Result<String> {
    let api_key = if provider_config.api_key.is_empty() {
        std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| anyhow::anyhow!("Anthropic API key not found"))?
    } else {
        provider_config.api_key.clone()
    };

    let model = provider_config.model.as_ref()
        .unwrap_or(&"claude-3-sonnet-20240229".to_string())
        .clone();

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": model,
            "system": "You are a helpful assistant that answers questions based on the provided context.",
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "max_tokens": 1000,
            "temperature": 0.7,
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("Anthropic API error: {}", error_text));
    }

    let response_json: serde_json::Value = response.json().await?;

    let content = response_json
        .get("content")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid response format from Anthropic"))?;

    Ok(content.to_string())
}

/// Generate response using Gemini
async fn generate_gemini_response(prompt: &str, provider_config: &crate::config::ProviderConfig) -> Result<String> {
    let api_key = if provider_config.api_key.is_empty() {
        std::env::var("GEMINI_API_KEY")
            .map_err(|_| anyhow::anyhow!("Gemini API key not found"))?
    } else {
        provider_config.api_key.clone()
    };

    let model = provider_config.model.as_ref()
        .unwrap_or(&"gemini-1.5-pro".to_string())
        .clone();

    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }],
            "generationConfig": {
                "temperature": 0.7,
                "maxOutputTokens": 1000,
            }
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("Gemini API error: {}", error_text));
    }

    let response_json: serde_json::Value = response.json().await?;

    let content = response_json
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid response format from Gemini"))?;

    Ok(content.to_string())
}

/// Generate response using Ollama
async fn generate_ollama_response(prompt: &str, provider_config: &crate::config::ProviderConfig) -> Result<String> {
    let model = provider_config.model.as_ref()
        .ok_or_else(|| anyhow::anyhow!("No model specified for Ollama"))?
        .clone();

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.7,
            }
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("Ollama API error: {}", error_text));
    }

    let response_json: serde_json::Value = response.json().await?;

    let content = response_json
        .get("response")
        .and_then(|r| r.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid response format from Ollama"))?;

    Ok(content.to_string())
}