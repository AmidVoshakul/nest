//! Model catalog for all supported LLM providers

use super::Provider;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCatalogEntry {
    pub id: String,
    pub provider: Provider,
    pub name: String,
    pub description: String,
    pub context_window: u32,
    pub max_output: u32,
    pub tier: ModelTier,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModelTier {
    Free,
    Fast,
    Balanced,
    Premium,
    Enterprise,
}

impl std::fmt::Display for ModelTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelTier::Free => write!(f, "free"),
            ModelTier::Fast => write!(f, "fast"),
            ModelTier::Balanced => write!(f, "balanced"),
            ModelTier::Premium => write!(f, "premium"),
            ModelTier::Enterprise => write!(f, "enterprise"),
        }
    }
}

#[derive(Debug, Clone, Default)]
#[allow(clippy::vec_init_then_push)] // We intentionally build incrementally for maintainability
pub struct ModelCatalog {
    models: Vec<ModelCatalogEntry>,
}

impl ModelCatalog {
    /// Create a new model catalog with all known models
    #[allow(clippy::vec_init_then_push)] // We intentionally build incrementally for maintainability
    pub fn new() -> Self {
        let mut models = Vec::new();

        // Anthropic
        models.push(ModelCatalogEntry {
            id: "claude-3-5-sonnet-20240620".into(),
            provider: Provider::Anthropic,
            name: "Claude 3.5 Sonnet".into(),
            description: "Balanced performance and speed".into(),
            context_window: 200000,
            max_output: 8192,
            tier: ModelTier::Premium,
        });

        models.push(ModelCatalogEntry {
            id: "claude-3-opus-20240229".into(),
            provider: Provider::Anthropic,
            name: "Claude 3 Opus".into(),
            description: "Highest quality reasoning".into(),
            context_window: 200000,
            max_output: 4096,
            tier: ModelTier::Enterprise,
        });

        models.push(ModelCatalogEntry {
            id: "claude-3-haiku-20240307".into(),
            provider: Provider::Anthropic,
            name: "Claude 3 Haiku".into(),
            description: "Fast and lightweight".into(),
            context_window: 200000,
            max_output: 4096,
            tier: ModelTier::Fast,
        });

        // OpenAI
        models.push(ModelCatalogEntry {
            id: "gpt-4o".into(),
            provider: Provider::OpenAI,
            name: "GPT-4o".into(),
            description: "Latest GPT-4 model with vision".into(),
            context_window: 128000,
            max_output: 4096,
            tier: ModelTier::Premium,
        });

        models.push(ModelCatalogEntry {
            id: "gpt-4o-mini".into(),
            provider: Provider::OpenAI,
            name: "GPT-4o Mini".into(),
            description: "Fast and cheap GPT-4".into(),
            context_window: 128000,
            max_output: 4096,
            tier: ModelTier::Fast,
        });

        models.push(ModelCatalogEntry {
            id: "gpt-3.5-turbo".into(),
            provider: Provider::OpenAI,
            name: "GPT-3.5 Turbo".into(),
            description: "Legacy fast model".into(),
            context_window: 16384,
            max_output: 4096,
            tier: ModelTier::Balanced,
        });

        // Groq
        models.push(ModelCatalogEntry {
            id: "llama-3.1-70b-versatile".into(),
            provider: Provider::Groq,
            name: "Llama 3.1 70B".into(),
            description: "Fast open source model".into(),
            context_window: 131072,
            max_output: 8192,
            tier: ModelTier::Free,
        });

        models.push(ModelCatalogEntry {
            id: "llama-3.1-8b-instant".into(),
            provider: Provider::Groq,
            name: "Llama 3.1 8B".into(),
            description: "Ultra fast small model".into(),
            context_window: 131072,
            max_output: 8192,
            tier: ModelTier::Free,
        });

        models.push(ModelCatalogEntry {
            id: "gemma2-9b-it".into(),
            provider: Provider::Groq,
            name: "Gemma 2 9B".into(),
            description: "Google's latest open model".into(),
            context_window: 8192,
            max_output: 8192,
            tier: ModelTier::Free,
        });

        // Deepseek
        models.push(ModelCatalogEntry {
            id: "deepseek-chat".into(),
            provider: Provider::Deepseek,
            name: "Deepseek V3".into(),
            description: "Deepseek reasoning model".into(),
            context_window: 64000,
            max_output: 4096,
            tier: ModelTier::Balanced,
        });

        // Mistral
        models.push(ModelCatalogEntry {
            id: "mistral-large-latest".into(),
            provider: Provider::Mistral,
            name: "Mistral Large".into(),
            description: "Mistral's flagship model".into(),
            context_window: 128000,
            max_output: 8192,
            tier: ModelTier::Premium,
        });

        // Gemini
        models.push(ModelCatalogEntry {
            id: "gemini-2.0-flash".into(),
            provider: Provider::Gemini,
            name: "Gemini 2.0 Flash".into(),
            description: "Google's latest fast model".into(),
            context_window: 1000000,
            max_output: 8192,
            tier: ModelTier::Free,
        });

        // z.ai
        models.push(ModelCatalogEntry {
            id: "zai-3".into(),
            provider: Provider::Zai,
            name: "ZAI-3".into(),
            description: "z.ai flagship model".into(),
            context_window: 128000,
            max_output: 8192,
            tier: ModelTier::Premium,
        });

        // Ollama
        models.push(ModelCatalogEntry {
            id: "llama3".into(),
            provider: Provider::Ollama,
            name: "Llama 3".into(),
            description: "Local Llama 3 model".into(),
            context_window: 8192,
            max_output: 4096,
            tier: ModelTier::Free,
        });

        // Together.ai
        models.push(ModelCatalogEntry {
            id: "meta-llama/Llama-3.1-70B-Instruct-Turbo".into(),
            provider: Provider::Together,
            name: "Llama 3.1 70B Turbo".into(),
            description: "Fast Llama 3.1 via Together".into(),
            context_window: 131072,
            max_output: 8192,
            tier: ModelTier::Balanced,
        });

        Self { models }
    }

    /// List all models in the catalog
    pub fn list_models(&self, provider_filter: Option<Provider>) -> Vec<&ModelCatalogEntry> {
        self.models
            .iter()
            .filter(move |m| provider_filter.is_none_or(|p| m.provider == p))
            .collect()
    }

    /// Get a model by ID
    pub fn get_model(&self, id: &str) -> Option<&ModelCatalogEntry> {
        self.models.iter().find(|m| m.id == id)
    }

    /// List all supported providers
    pub fn list_providers(&self) -> Vec<Provider> {
        let mut providers = Vec::new();
        for model in &self.models {
            if !providers.contains(&model.provider) {
                providers.push(model.provider);
            }
        }
        providers
    }
}
