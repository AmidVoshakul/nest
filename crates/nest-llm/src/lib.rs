//! LLM provider integration for Nest Agent Hypervisor
//!
//! Implements provider-agnostic LLM client with support for:
//! - Anthropic (Claude 3, Claude 3.5, Claude 3 Opus/Sonnet/Haiku)
//! - OpenAI (GPT-4o, GPT-4o mini, GPT-3.5 Turbo)
//! - OpenRouter (all supported models)
//!
//! Follows nanoclaw security best practices:
//! - Credentials never exposed to agents
//! - Token usage tracking and budgeting
//! - Rate limiting per agent
//! - No implicit permissions

pub mod repair;
pub mod sanitize;
pub mod prompt_hash;
pub mod model_catalog;

use std::{collections::HashMap, time::Duration};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use zeroize::{Zeroize, Zeroizing, ZeroizeOnDrop};

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Provider not supported: {0}")]
    UnsupportedProvider(String),
    
    #[error("API key not found for provider: {0}")]
    ApiKeyNotFound(String),
    
    #[error("Request failed: {0}")]
    RequestFailed(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Token budget exceeded")]
    TokenBudgetExceeded,
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum Provider {
    Anthropic,
    OpenAI,
    OpenRouter,
    Zai,
    Gemini,
    Ollama,
    Deepseek,
    Mistral,
    Groq,
    Together,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    
    #[serde(default)]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "input_schema")]
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub temperature: f32,
    
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,
    
    #[serde(default)]
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Usage,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Zeroize, ZeroizeOnDrop, Debug)]
pub struct LlmClient {
    #[zeroize(skip)]
    provider: Provider,
    api_key: Zeroizing<String>,
    #[zeroize(skip)]
    base_url: String,
    #[zeroize(skip)]
    client: reqwest::Client,
    
    // Rate limiting
    #[zeroize(skip)]
    rate_limit: Option<Duration>,
    #[zeroize(skip)]
    last_request: Option<std::time::Instant>,
    
    // Token budgeting
    #[zeroize(skip)]
    token_budget: u32,
    #[zeroize(skip)]
    tokens_used: u32,
    #[zeroize(skip)]
    budget_window_start: std::time::Instant,
}

impl LlmClient {
    /// Get the provider for this client
    pub fn provider(&self) -> Provider {
        self.provider
    }
    /// Create new LLM client for specified provider
    pub fn new(provider: Provider) -> Result<Self, LlmError> {
        let api_key = match provider {
            Provider::Anthropic => std::env::var("ANTHROPIC_API_KEY")
                .map_err(|_| LlmError::ApiKeyNotFound("anthropic".into()))?,
            Provider::OpenAI => std::env::var("OPENAI_API_KEY")
                .map_err(|_| LlmError::ApiKeyNotFound("openai".into()))?,
            Provider::OpenRouter => std::env::var("OPENROUTER_API_KEY")
                .map_err(|_| LlmError::ApiKeyNotFound("openrouter".into()))?,
            Provider::Zai => std::env::var("ZAI_API_KEY")
                .map_err(|_| LlmError::ApiKeyNotFound("zai".into()))?,
            Provider::Gemini => std::env::var("GOOGLE_API_KEY")
                .map_err(|_| LlmError::ApiKeyNotFound("gemini".into()))?,
            Provider::Ollama => "".into(), // Ollama doesn't require API key
            Provider::Deepseek => std::env::var("DEEPSEEK_API_KEY")
                .map_err(|_| LlmError::ApiKeyNotFound("deepseek".into()))?,
            Provider::Mistral => std::env::var("MISTRAL_API_KEY")
                .map_err(|_| LlmError::ApiKeyNotFound("mistral".into()))?,
            Provider::Groq => std::env::var("GROQ_API_KEY")
                .map_err(|_| LlmError::ApiKeyNotFound("groq".into()))?,
            Provider::Together => std::env::var("TOGETHER_API_KEY")
                .map_err(|_| LlmError::ApiKeyNotFound("together".into()))?,
        };

        let base_url = match provider {
            Provider::Anthropic => "https://api.anthropic.com/v1".into(),
            Provider::OpenAI => "https://api.openai.com/v1".into(),
            Provider::OpenRouter => "https://openrouter.ai/api/v1".into(),
            Provider::Zai => "https://api.z.ai/v1".into(),
            Provider::Gemini => "https://generativelanguage.googleapis.com/v1beta".into(),
            Provider::Ollama => "http://localhost:11434/v1".into(),
            Provider::Deepseek => "https://api.deepseek.com/v1".into(),
            Provider::Mistral => "https://api.mistral.ai/v1".into(),
            Provider::Groq => "https://api.groq.com/openai/v1".into(),
            Provider::Together => "https://api.together.xyz/v1".into(),
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        Ok(Self {
            provider,
            api_key: Zeroizing::new(api_key),
            base_url,
            client,
            rate_limit: None,
            last_request: None,
            token_budget: u32::MAX,
            tokens_used: 0,
            budget_window_start: std::time::Instant::now(),
        })
    }

    /// Get default model name for provider
    pub fn default_model(&self) -> &'static str {
        match self.provider {
            Provider::Anthropic => "claude-3-5-sonnet-20240620",
            Provider::OpenAI => "gpt-4o",
            Provider::OpenRouter => "anthropic/claude-3.5-sonnet",
            Provider::Zai => "zai-3",
            Provider::Gemini => "gemini-2.0-flash",
            Provider::Ollama => "llama3",
            Provider::Deepseek => "deepseek-chat",
            Provider::Mistral => "mistral-large-latest",
            Provider::Groq => "llama-3.1-70b-versatile",
            Provider::Together => "meta-llama/Llama-3.1-70B-Instruct-Turbo",
        }
    }

    /// Create client from provider name string
    pub fn from_name(name: &str) -> Result<Self, LlmError> {
        let provider = match name.to_lowercase().as_str() {
            "anthropic" | "claude" => Provider::Anthropic,
            "openai" | "gpt" => Provider::OpenAI,
            "openrouter" => Provider::OpenRouter,
            "zai" | "z.ai" => Provider::Zai,
            "gemini" | "google" => Provider::Gemini,
            "ollama" | "local" => Provider::Ollama,
            "deepseek" => Provider::Deepseek,
            "mistral" => Provider::Mistral,
            "groq" => Provider::Groq,
            "together" => Provider::Together,
            "default" => Provider::OpenRouter, // Default to OpenRouter which has free models
            _ => return Err(LlmError::UnsupportedProvider(name.into())),
        };

        eprintln!("🔌 LLM client created for provider: {:?}", provider);
        Self::new(provider)
    }

    /// Set token budget per hour
    pub fn with_token_budget(mut self, tokens_per_hour: u32) -> Self {
        self.token_budget = tokens_per_hour;
        self
    }

    /// Set rate limit
    pub fn with_rate_limit(mut self, requests_per_minute: u32) -> Self {
        self.rate_limit = Some(Duration::from_secs(60 / requests_per_minute as u64));
        self
    }

    /// Execute LLM request
    pub async fn chat_completion(&mut self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        // Check rate limit
        if let Some(limit) = self.rate_limit {
            if let Some(last) = self.last_request {
                if last.elapsed() < limit {
                    tokio::time::sleep(limit - last.elapsed()).await;
                }
            }
        }

        // Check token budget (reset hourly)
        if self.budget_window_start.elapsed() > Duration::from_secs(3600) {
            self.tokens_used = 0;
            self.budget_window_start = std::time::Instant::now();
        }

        if self.tokens_used >= self.token_budget {
            return Err(LlmError::TokenBudgetExceeded);
        }

        let response = match self.provider {
            Provider::Anthropic => self.call_anthropic(request).await?,
            Provider::OpenAI => self.call_openai(request).await?,
            Provider::OpenRouter => self.call_openrouter(request).await?,
            Provider::Zai => self.call_openai(request).await?,
            Provider::Gemini => self.call_openai(request).await?,
            Provider::Ollama => self.call_openai(request).await?,
            Provider::Deepseek => self.call_openai(request).await?,
            Provider::Mistral => self.call_openai(request).await?,
            Provider::Groq => self.call_openai(request).await?,
            Provider::Together => self.call_openai(request).await?,
        };

        // Update usage tracking
        self.tokens_used += response.usage.total_tokens;
        self.last_request = Some(std::time::Instant::now());

        Ok(response)
    }

    async fn call_anthropic(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        let mut body = serde_json::json!({
            "model": request.model,
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
            "messages": request.messages,
        });

        if let Some(system) = request.system_prompt {
            body["system"] = system.into();
        }

        if !request.tools.is_empty() {
            body["tools"] = serde_json::to_value(&request.tools).unwrap();
        }

        let resp = self.client.post(format!("{}/messages", self.base_url))
            .header("x-api-key", &*self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let error = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
            eprintln!("❌ OpenRouter request failed: {} - {}", status, error);
            return Err(LlmError::RequestFailed(error));
        }

        let json: serde_json::Value = resp.json()
            .await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        let mut content = None;
        let mut tool_calls = Vec::new();
        
        if let Some(content_array) = json["content"].as_array() {
            for item in content_array {
                if item["type"] == "text" {
                    content = item["text"].as_str().map(|s| s.to_string());
                } else if item["type"] == "tool_use" {
                    tool_calls.push(ToolCall {
                        id: item["id"].as_str().unwrap_or_default().to_string(),
                        name: item["name"].as_str().unwrap_or_default().to_string(),
                        arguments: item["input"].clone(),
                    });
                }
            }
        }

        let usage = Usage {
            prompt_tokens: json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32 
                + json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
        };

        Ok(LlmResponse {
            content,
            tool_calls,
            usage,
            model: json["model"].as_str().unwrap_or_default().to_string(),
        })
    }

    async fn call_openai(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        let mut messages = Vec::new();
        
        if let Some(system) = request.system_prompt {
            messages.push(serde_json::json!({
                "role": "system",
                "content": system
            }));
        }

        for msg in request.messages {
            messages.push(serde_json::json!({
                "role": match msg.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::Tool => "tool",
                    Role::System => "system",
                },
                "content": msg.content,
                "tool_call_id": msg.tool_call_id,
            }));
        }

        let mut body = serde_json::json!({
            "model": request.model,
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
            "messages": messages,
        });

        if !request.tools.is_empty() {
            body["tools"] = serde_json::to_value(&request.tools).unwrap();
        }

        let body_str = serde_json::to_string_pretty(&body).unwrap_or_default();
        eprintln!("📤 Request body (first 500 chars): {}", &body_str[..std::cmp::min(500, body_str.len())]);

        let resp = self.client.post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", &*self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/anomalyco/nest")
            .header("X-Title", "Nest Agent Hypervisor")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
            return Err(LlmError::RequestFailed(error));
        }

        let json: serde_json::Value = resp.json()
            .await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        let choice = &json["choices"][0];
        let content = choice["message"]["content"].as_str().map(|s| s.to_string());
        
        let usage = Usage {
            prompt_tokens: json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: json["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,
        };

        let mut tool_calls = Vec::new();
        if let Some(calls) = choice["message"]["tool_calls"].as_array() {
            for call in calls {
                tool_calls.push(ToolCall {
                    id: call["id"].as_str().unwrap_or_default().to_string(),
                    name: call["function"]["name"].as_str().unwrap_or_default().to_string(),
                    arguments: serde_json::from_str(call["function"]["arguments"].as_str().unwrap_or_default())
                        .unwrap_or_default(),
                });
            }
        }

        Ok(LlmResponse {
            content,
            tool_calls,
            usage,
            model: json["model"].as_str().unwrap_or_default().to_string(),
        })
    }

    async fn call_openrouter(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        // OpenRouter uses OpenAI-compatible API
        self.call_openai(request).await
    }
}

/// LLM provider registry for multiple agents
#[derive(Debug, Default)]
pub struct LlmRegistry {
    clients: HashMap<String, LlmClient>,
}

impl LlmRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create client for agent with specified provider and budget
    pub fn get_client(&mut self, agent_id: &str, provider: &str, token_budget: u32) -> Result<&mut LlmClient, LlmError> {
        if !self.clients.contains_key(agent_id) {
            let client = LlmClient::from_name(provider)?
                .with_token_budget(token_budget)
                .with_rate_limit(60); // 1 request per second default
            
            self.clients.insert(agent_id.to_string(), client);
        }

        Ok(self.clients.get_mut(agent_id).unwrap())
    }
}
