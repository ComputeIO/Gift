use anyhow::Result;
use leaf::providers::provider_registry::ProviderConstructor;
use std::sync::Arc;
use tracing::info;

use crate::server::LeafAcpAgent;

pub struct AcpServerFactoryConfig {
    pub builtins: Vec<String>,
    pub data_dir: std::path::PathBuf,
    pub config_dir: std::path::PathBuf,
}

pub struct AcpServer {
    config: AcpServerFactoryConfig,
}

impl AcpServer {
    pub fn new(config: AcpServerFactoryConfig) -> Self {
        Self { config }
    }

    pub async fn create_agent(&self) -> Result<Arc<LeafAcpAgent>> {
        let config_path = self
            .config
            .config_dir
            .join(leaf::config::base::CONFIG_YAML_NAME);
        let config = leaf::config::Config::new(&config_path, "leaf")?;

        let leaf_mode = config
            .get_leaf_mode()
            .unwrap_or(leaf::config::LeafMode::Auto);
        let disable_session_naming = config.get_leaf_disable_session_naming().unwrap_or(false);

        let config_dir = self.config.config_dir.clone();
        let provider_factory: ProviderConstructor = Arc::new(move |model_config, extensions| {
            let config_dir = config_dir.clone();
            Box::pin(async move {
                let config_path = config_dir.join(leaf::config::base::CONFIG_YAML_NAME);
                let config = leaf::config::Config::new(&config_path, "leaf")?;
                let provider_name = config.get_leaf_provider()?;
                leaf::providers::create(&provider_name, model_config, extensions).await
            })
        });

        let agent = LeafAcpAgent::new(
            provider_factory,
            self.config.builtins.clone(),
            self.config.data_dir.clone(),
            self.config.config_dir.clone(),
            leaf_mode,
            disable_session_naming,
        )
        .await?;
        info!("Created new ACP agent");

        Ok(Arc::new(agent))
    }
}
