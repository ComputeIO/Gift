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

const CLAUDE_ACP_PROVIDER_NAME: &str = "claude-acp";
const CLAUDE_ACP_DOC_URL: &str = "https://github.com/zed-industries/claude-agent-acp";
const CLAUDE_ACP_BINARY: &str = "claude-agent-acp";

pub struct ClaudeAcpProvider;

impl ProviderDef for ClaudeAcpProvider {
    type Provider = AcpProvider;

    fn metadata() -> ProviderMetadata {
        ProviderMetadata::new(
            CLAUDE_ACP_PROVIDER_NAME,
            "Claude Code",
            "Use leaf with your Claude Code subscription via the claude-agent-acp adapter.",
            ACP_CURRENT_MODEL,
            vec![],
            CLAUDE_ACP_DOC_URL,
            vec![],
        )
        .with_setup_steps(vec![
            "Install the ACP adapter: `npm install -g @zed-industries/claude-agent-acp`",
            "Ensure your Claude CLI is authenticated (run `claude` to verify)",
            "Set in your leaf config file (`~/.config/leaf/config.yaml` on macOS/Linux):\n  LEAF_PROVIDER: claude-acp\n  LEAF_MODEL: current",
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
                .resolve(CLAUDE_ACP_BINARY)?;
            let leaf_mode = config.get_leaf_mode().unwrap_or(LeafMode::Auto);

            let permission_mapping = PermissionMapping {
                allow_option_id: Some("allow".to_string()),
                reject_option_id: Some("reject".to_string()),
                rejected_tool_status: sacp::schema::ToolCallStatus::Failed,
            };

            let mode_mapping = HashMap::from([
                (LeafMode::Auto, "bypassPermissions".to_string()),
                (LeafMode::Approve, "default".to_string()),
                (LeafMode::SmartApprove, "acceptEdits".to_string()),
                (LeafMode::Chat, "plan".to_string()),
            ]);

            let provider_config = AcpProviderConfig {
                command: resolved_command,
                args: vec![],
                env: vec![],
                env_remove: vec!["CLAUDECODE".to_string()],
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
