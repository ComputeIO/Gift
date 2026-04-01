use anyhow::Result;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::acp::{
    extension_configs_to_mcp_servers, AcpProvider, AcpProviderConfig, PermissionMapping,
    ACP_CURRENT_MODEL,
};
use crate::config::search_path::SearchPaths;
use crate::config::{Config, LeafMode};
use crate::model::ModelConfig;
use crate::providers::base::{ProviderDef, ProviderMetadata};

const COPILOT_ACP_PROVIDER_NAME: &str = "copilot-acp";
pub const COPILOT_ACP_DEFAULT_MODEL: &str = "current";
const COPILOT_ACP_DOC_URL: &str = "https://github.com/github/copilot-cli";
const COPILOT_ACP_BINARY: &str = "copilot";

const MODE_AGENT: &str = "https://agentclientprotocol.com/protocol/session-modes#agent";
const MODE_PLAN: &str = "https://agentclientprotocol.com/protocol/session-modes#plan";

pub struct CopilotAcpProvider;

impl ProviderDef for CopilotAcpProvider {
    type Provider = AcpProvider;

    fn metadata() -> ProviderMetadata {
        ProviderMetadata::new(
            COPILOT_ACP_PROVIDER_NAME,
            "GitHub Copilot CLI (ACP)",
            "Use leaf with your GitHub Copilot subscription via the Copilot CLI.",
            ACP_CURRENT_MODEL,
            vec![],
            COPILOT_ACP_DOC_URL,
            vec![],
        )
        .with_setup_steps(vec![
            "Install the Copilot CLI: `npm install -g @github/copilot`",
            "Run `copilot login` to authenticate with your GitHub account",
            "Set in your leaf config file (`~/.config/leaf/config.yaml` on macOS/Linux):\n  LEAF_PROVIDER: copilot-acp\n  LEAF_MODEL: current",
            "Restart leaf for changes to take effect",
        ])
    }

    fn from_env(
        model: ModelConfig,
        extensions: Vec<crate::config::ExtensionConfig>,
    ) -> BoxFuture<'static, Result<AcpProvider>> {
        Box::pin(async move {
            let config = Config::global();
            let resolved_command = SearchPaths::builder()
                .with_npm()
                .resolve(COPILOT_ACP_BINARY)?;
            let leaf_mode = config.get_leaf_mode().unwrap_or(LeafMode::Auto);

            let permission_mapping = PermissionMapping::default();

            let mut args = vec!["--acp".to_string()];
            if model.model_name != COPILOT_ACP_DEFAULT_MODEL {
                args.push("--model".to_string());
                args.push(model.model_name.clone());
            }

            let mode_mapping = HashMap::from([
                (LeafMode::Auto, MODE_AGENT.to_string()),
                (LeafMode::Approve, MODE_AGENT.to_string()),
                (LeafMode::SmartApprove, MODE_AGENT.to_string()),
                (LeafMode::Chat, MODE_PLAN.to_string()),
            ]);

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
