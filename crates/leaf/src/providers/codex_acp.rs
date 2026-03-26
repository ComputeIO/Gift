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

const CODEX_ACP_PROVIDER_NAME: &str = "codex-acp";
const CODEX_ACP_DOC_URL: &str = "https://github.com/zed-industries/codex-acp";
const CODEX_ACP_BINARY: &str = "codex";

pub struct CodexAcpProvider;

impl ProviderDef for CodexAcpProvider {
    type Provider = AcpProvider;

    fn metadata() -> ProviderMetadata {
        ProviderMetadata::new(
            CODEX_ACP_PROVIDER_NAME,
            "Codex CLI",
            "Use goose with your ChatGPT Plus/Pro subscription via the codex-acp adapter.",
            ACP_CURRENT_MODEL,
            vec![],
            CODEX_ACP_DOC_URL,
            vec![],
        )
        .with_setup_steps(vec![
            "Install the ACP adapter: `npm install -g @zed-industries/codex-acp`",
            "Run `codex` once to authenticate with your OpenAI account",
            "Set in your goose config file (`~/.config/goose/config.yaml` on macOS/Linux):\n  GOOSE_PROVIDER: codex-acp\n  GOOSE_MODEL: current",
            "Restart goose for changes to take effect",
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
                .resolve(CODEX_ACP_BINARY)?;
            let work_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let env = vec![];
            let leaf_mode = config.get_leaf_mode().unwrap_or(LeafMode::Auto);
            let mcp_servers = extension_configs_to_mcp_servers(&extensions);

            let permission_mapping = PermissionMapping {
                allow_option_id: Some("approved".to_string()),
                reject_option_id: Some("abort".to_string()),
                rejected_tool_status: sacp::schema::ToolCallStatus::Failed,
            };

            let mode_mapping = HashMap::from([
                (LeafMode::Auto, "never".to_string()),
                (LeafMode::SmartApprove, "on-request".to_string()),
                (LeafMode::Approve, "on-request".to_string()),
                (LeafMode::Chat, "never".to_string()),
            ]);

            let (approval_policy, sandbox_mode) = map_leaf_mode(leaf_mode);

            let mut args = vec![
                "-c".to_string(),
                format!("approval_policy={approval_policy}"),
                "-c".to_string(),
                format!("sandbox_mode={sandbox_mode}"),
            ];

            let has_http_mcp = mcp_servers
                .iter()
                .any(|s| matches!(s, sacp::schema::McpServer::Http(_)));
            if has_http_mcp {
                args.extend([
                    "-c".to_string(),
                    "sandbox_workspace_write.network_access=true".to_string(),
                ]);
            }

            let provider_config = AcpProviderConfig {
                command: resolved_command,
                args,
                env,
                env_remove: vec![],
                work_dir,
                mcp_servers,
                session_mode_id: None,
                mode_mapping,
                permission_mapping,
                notification_callback: None,
            };

            let metadata = Self::metadata();
            AcpProvider::connect(metadata.name, model, leaf_mode, provider_config).await
        })
    }
}

// Codex sandbox scope determines what needs approval: operations within the
// sandbox are auto-approved, operations outside it trigger on-request prompts.
// So Approve uses read-only sandbox to force write approvals through goose.
fn map_leaf_mode(leaf_mode: LeafMode) -> (&'static str, &'static str) {
    match leaf_mode {
        LeafMode::Auto => ("never", "danger-full-access"),
        LeafMode::SmartApprove => ("on-request", "workspace-write"),
        LeafMode::Approve => ("on-request", "read-only"),
        LeafMode::Chat => ("never", "read-only"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(LeafMode::Auto, "never", "danger-full-access")]
    #[test_case(LeafMode::SmartApprove, "on-request", "workspace-write")]
    #[test_case(LeafMode::Approve, "on-request", "read-only")]
    #[test_case(LeafMode::Chat, "never", "read-only")]
    fn test_map_leaf_mode(mode: LeafMode, expected_approval: &str, expected_sandbox: &str) {
        let (approval, sandbox) = map_leaf_mode(mode);
        assert_eq!(approval, expected_approval);
        assert_eq!(sandbox, expected_sandbox);
    }
}
