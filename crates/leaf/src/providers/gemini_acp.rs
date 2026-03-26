use std::collections::HashMap;

use anyhow::Result;
use futures::future::BoxFuture;
use std::path::PathBuf;

use crate::acp::{
    extension_configs_to_mcp_servers, AcpProvider, AcpProviderConfig, PermissionMapping,
    ACP_CURRENT_MODEL,
};
use crate::config::search_path::SearchPaths;
use crate::config::{Config, LeafMode};
use crate::model::ModelConfig;
use crate::providers::base::{ProviderDef, ProviderMetadata};

const GEMINI_ACP_PROVIDER_NAME: &str = "gemini-acp";
const GEMINI_ACP_DOC_URL: &str = "https://github.com/google-gemini/gemini-cli";

pub struct GeminiAcpProvider;

impl ProviderDef for GeminiAcpProvider {
    type Provider = AcpProvider;

    fn metadata() -> ProviderMetadata {
        ProviderMetadata::new(
            GEMINI_ACP_PROVIDER_NAME,
            "Gemini CLI (ACP)",
            "Use leaf with your Google Gemini subscription via the Gemini CLI.",
            ACP_CURRENT_MODEL,
            vec![],
            GEMINI_ACP_DOC_URL,
            vec![],
        )
        .with_setup_steps(vec![
            "Install the Gemini CLI: `npm install -g @google/gemini-cli`",
            "Run `gemini` once to authenticate with your Google account",
            "Set in your leaf config file (`~/.config/leaf/config.yaml` on macOS/Linux):\n  LEAF_PROVIDER: gemini-acp\n  LEAF_MODEL: current",
            "Restart leaf for changes to take effect",
        ])
    }

    fn from_env(
        model: ModelConfig,
        extensions: Vec<crate::config::ExtensionConfig>,
    ) -> BoxFuture<'static, Result<AcpProvider>> {
        Box::pin(async move {
            let config = Config::global();
            let command_name: String = config.get_gemini_cli_command().unwrap_or_default().into();
            let resolved_command = SearchPaths::builder().with_npm().resolve(&command_name)?;
            let leaf_mode = config.get_leaf_mode().unwrap_or(LeafMode::Auto);

            let permission_mapping = PermissionMapping {
                allow_option_id: Some("allow".to_string()),
                reject_option_id: Some("reject".to_string()),
                rejected_tool_status: sacp::schema::ToolCallStatus::Failed,
            };

            let mode_mapping = HashMap::from([
                (LeafMode::Auto, "yolo".to_string()),
                (LeafMode::Approve, "default".to_string()),
                (LeafMode::SmartApprove, "auto_edit".to_string()),
                (LeafMode::Chat, "plan".to_string()),
            ]);

            let mut args = vec!["--acp".to_string()];
            if model.model_name != "default" {
                args.push("--model".to_string());
                args.push(model.model_name.clone());
            }

            let provider_config = AcpProviderConfig {
                command: resolved_command,
                args,
                env: vec![],
                env_remove: vec![],
                work_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                mcp_servers: extension_configs_to_mcp_servers(&extensions),
                session_mode_id: Some(mode_mapping[&leaf_mode].clone()),
                mode_mapping,
                permission_mapping,
                notification_callback: None,
            };

            let metadata = Self::metadata();
            AcpProvider::connect(metadata.name, model, leaf_mode, provider_config).await
        })
    }
}
