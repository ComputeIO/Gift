use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};
use tracing::{info, warn};

use crate::config::declarative_providers::{
    register_declarative_provider, DeclarativeProviderConfig, ProviderEngine,
};
use crate::providers::base::{ModelInfo, ProviderType};
use crate::providers::init::get_registry_optional;
use crate::providers::provider_registry::ProviderRegistry;

const MODELS_DEV_URL: &str = "https://models.dev/api.json";
const CACHE_DIR: &str = "leaf/opencode";
const CACHE_TTL: Duration = Duration::from_secs(1800);

/// Default API URLs for npm packages that don't have an `api` field in models.dev.
/// These match the hardcoded defaults in each @ai-sdk/* package.
fn default_url_for_npm(npm: Option<&str>) -> Option<&'static str> {
    match npm {
        Some("@ai-sdk/openai") => Some("https://api.openai.com/v1"),
        Some("@ai-sdk/anthropic") => Some("https://api.anthropic.com/v1"),
        _ => None,
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProviderCatalog {
    #[serde(flatten)]
    pub providers: HashMap<String, ProviderInfo>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProviderInfo {
    pub id: Option<String>,
    pub name: String,
    pub api: Option<String>,
    pub env: Option<Vec<String>>,
    pub npm: Option<String>,
    #[serde(default)]
    pub models: HashMap<String, ModelDetail>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ModelDetail {
    pub id: Option<String>,
    pub name: String,
    pub family: Option<String>,
    #[serde(default)]
    pub reasoning: bool,
    #[serde(default)]
    pub tool_call: bool,
    pub cost: Option<CostInfo>,
    pub limit: LimitInfo,
    #[serde(default)]
    pub provider: Option<ModelProvider>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ModelProvider {
    pub npm: String,
    #[serde(default)]
    pub api: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CostInfo {
    pub input: f64,
    pub output: f64,
    #[serde(rename = "cache_read")]
    pub cache_read: Option<f64>,
    #[serde(rename = "cache_write")]
    pub cache_write: Option<f64>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LimitInfo {
    pub context: usize,
    pub input: Option<usize>,
    pub output: usize,
}

fn build_http_client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()?)
}

async fn fetch_catalog_from_network() -> Result<ProviderCatalog> {
    let client = build_http_client()?;
    let response = client.get(MODELS_DEV_URL).send().await?;
    let json: serde_json::Value = response.json().await?;
    let catalog: ProviderCatalog = serde_json::from_value(json)?;
    Ok(catalog)
}

fn get_cache_path() -> std::path::PathBuf {
    let home = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    home.join(CACHE_DIR).join("api.json")
}

async fn get_cached_catalog() -> Option<ProviderCatalog> {
    let path = get_cache_path();
    if !path.exists() {
        return None;
    }

    let content = tokio::fs::read_to_string(&path).await.ok()?;
    let catalog: ProviderCatalog = serde_json::from_str(&content).ok()?;
    Some(catalog)
}

fn is_cache_stale() -> bool {
    let path = get_cache_path();
    if !path.exists() {
        return true;
    }

    fs::metadata(&path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|time| {
            SystemTime::now()
                .duration_since(time)
                .unwrap_or(Duration::from_secs(u64::MAX))
                > CACHE_TTL
        })
        .unwrap_or(true)
}

async fn cache_catalog(catalog: &ProviderCatalog) -> Result<()> {
    let path = get_cache_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let json = serde_json::to_string_pretty(catalog)?;
    let tmp_path = path.with_extension("json.tmp");
    tokio::fs::write(&tmp_path, &json).await?;
    tokio::fs::rename(&tmp_path, &path).await?;
    Ok(())
}

fn get_model_npm(provider_npm: Option<&str>, model: &ModelDetail) -> String {
    let npm = model
        .provider
        .as_ref()
        .map(|p| p.npm.as_str())
        .or(provider_npm)
        .unwrap_or("unknown");
    npm.trim_start_matches("@ai-sdk/").to_string()
}

fn get_model_engine(provider_npm: Option<&str>, model: &ModelDetail) -> ProviderEngine {
    let npm = model
        .provider
        .as_ref()
        .map(|p| p.npm.as_str())
        .or(provider_npm);
    if npm == Some("@ai-sdk/anthropic") {
        ProviderEngine::Anthropic
    } else {
        ProviderEngine::OpenAI
    }
}

fn register_providers_from_catalog(
    registry: &mut ProviderRegistry,
    catalog: ProviderCatalog,
) -> Result<()> {
    for (provider_id, provider_info) in catalog.providers {
        if provider_info.models.is_empty() {
            warn!("Skipping provider {} with no models", provider_id);
            continue;
        }

        let provider_npm = provider_info.npm.as_deref();

        let base_url = match &provider_info.api {
            Some(url) if !url.is_empty() => url.clone(),
            _ => match default_url_for_npm(provider_npm) {
                Some(url) => url.to_string(),
                None => {
                    warn!(
                        "Skipping provider {} with no API URL and no default for npm {:?}",
                        provider_id, provider_npm
                    );
                    continue;
                }
            },
        };

        let mut groups: HashMap<String, (ProviderEngine, String, Vec<ModelInfo>)> = HashMap::new();

        for model in provider_info.models.values() {
            let engine = get_model_engine(provider_npm, model);
            let npm = get_model_npm(provider_npm, model);
            let model_api = model.provider.as_ref().and_then(|p| p.api.as_ref());
            let model_base_url = model_api.map(|s| s.as_str()).unwrap_or(&base_url);
            // Use npm as grouping key (not npm:url)
            let key = npm.to_string();

            let entry = groups
                .entry(key.clone())
                .or_insert_with(|| (engine, model_base_url.to_string(), Vec::new()));

            entry.2.push(ModelInfo {
                name: model.id.clone().unwrap_or_else(|| model.name.clone()),
                context_limit: model.limit.context,
                input_token_cost: model.cost.as_ref().map(|c| c.input),
                output_token_cost: model.cost.as_ref().map(|c| c.output),
                currency: None,
                supports_cache_control: None,
            });
        }

        for (key, (engine, model_base_url, models)) in &groups {
            let api_key_env = if let Some(ref env_list) = provider_info.env {
                if let Some(first_env) = env_list.first() {
                    first_env.clone()
                } else {
                    format!("{}_API_KEY", provider_id.to_uppercase())
                }
            } else {
                format!("{}_API_KEY", provider_id.to_uppercase())
            };

            // Always include engine in name: opencode-{provider_id}-{engine}
            let engine_name = key.trim_start_matches("@ai-sdk/");
            let name = format!("opencode-{}-{}", provider_id, engine_name);

            let display_name = format!("OpenCode::{}({})", provider_info.name, engine_name);

            let config = DeclarativeProviderConfig {
                name: name.clone(),
                display_name: display_name.clone(),
                engine: engine.clone(),
                description: None,
                api_key_env,
                base_url: model_base_url.clone(),
                models: models.clone(),
                headers: None,
                timeout_seconds: None,
                supports_streaming: Some(true),
                requires_auth: true,
                catalog_provider_id: Some(provider_id.clone()),
                base_path: None,
                env_vars: None,
                dynamic_models: None,
                skip_canonical_filtering: true,
            };

            register_declarative_provider(registry, config, ProviderType::Preferred);
            info!(
                "Registered OpenCode provider: {} (display: {}) with {} models",
                name,
                display_name,
                models.len()
            );
        }
    }

    Ok(())
}

pub async fn register_opencode_providers(registry: &mut ProviderRegistry) -> Result<()> {
    // Try cache first for fast startup
    if let Some(cached) = get_cached_catalog().await {
        info!("Using cached OpenCode provider catalog");
        register_providers_from_catalog(registry, cached)?;

        // Refresh in background if cache is stale
        if is_cache_stale() {
            tokio::spawn(async move {
                loop {
                    let opt = get_registry_optional();
                    match opt {
                        Some(registry) => {
                            info!("Refreshing OpenCode provider catalog in background");
                            match refresh_catalog(registry).await {
                                Ok(()) => info!("OpenCode provider catalog refreshed"),
                                Err(e) => warn!("Failed to refresh catalog: {}", e),
                            }
                            break;
                        }
                        None => {
                            std::thread::sleep(std::time::Duration::from_millis(500));
                        }
                    }
                }
            });
        }
        return Ok(());
    }

    // No cache — must fetch from network
    info!("Fetching OpenCode provider catalog from models.dev");
    let catalog = fetch_catalog_from_network().await?;
    cache_catalog(&catalog).await?;
    register_providers_from_catalog(registry, catalog)?;
    Ok(())
}

async fn refresh_catalog(registry: &RwLock<ProviderRegistry>) -> Result<()> {
    let catalog = fetch_catalog_from_network().await?;
    cache_catalog(&catalog).await?;
    let mut reg = registry.write().unwrap();
    register_providers_from_catalog(&mut reg, catalog)?;
    Ok(())
}
