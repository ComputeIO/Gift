mod common;
mod provider;

pub use common::{map_permission_response, PermissionDecision, PermissionMapping};
pub use provider::{extension_configs_to_mcp_servers, AcpProvider, AcpProviderConfig};

/// Sentinel: resolved to SessionModelState.current_model_id at connect time.
pub const ACP_CURRENT_MODEL: &str = "current";
