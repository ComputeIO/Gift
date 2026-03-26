use dotenvy::dotenv;
use futures::StreamExt;
use leaf::agents::{Agent, AgentEvent, ExtensionConfig, SessionConfig};
use leaf::config::{LeafMode, DEFAULT_EXTENSION_DESCRIPTION, DEFAULT_EXTENSION_TIMEOUT};
use leaf::conversation::message::Message;
use leaf::providers::create_with_named_model;
use leaf::providers::databricks::DATABRICKS_DEFAULT_MODEL;
use leaf::session::session_manager::SessionType;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();

    let provider =
        create_with_named_model("databricks", DATABRICKS_DEFAULT_MODEL, Vec::new()).await?;

    let agent = Agent::new();

    let session = agent
        .config
        .session_manager
        .create_session(
            PathBuf::default(),
            "max-turn-test".to_string(),
            SessionType::Hidden,
            LeafMode::default(),
        )
        .await?;

    agent.update_provider(provider, &session.id).await?;

    let config = ExtensionConfig::stdio(
        "developer",
        "./target/debug/leaf",
        DEFAULT_EXTENSION_DESCRIPTION,
        DEFAULT_EXTENSION_TIMEOUT,
    )
    .with_args(vec!["mcp", "developer"]);
    agent.add_extension(config, &session.id).await?;

    println!("Extensions:");
    for extension in agent.list_extensions().await {
        println!("  {}", extension);
    }

    let session_config = SessionConfig {
        id: session.id,
        schedule_id: None,
        max_turns: None,
        retry_config: None,
    };

    let user_message = Message::user()
        .with_text("can you summarize the readme.md in this dir using just a haiku?");

    let mut stream = agent.reply(user_message, session_config, None).await?;

    while let Some(Ok(AgentEvent::Message(message))) = stream.next().await {
        println!("{}", serde_json::to_string_pretty(&message)?);
        println!("\n");
    }

    Ok(())
}
