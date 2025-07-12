use std::sync::Arc;

use crate::schema::json_schema_for;
use anyhow::{Result, anyhow};
use assistant_tool::{ActionLog, Tool, ToolResult};
use context_server::manager::ContextServerManager;
use gpui::{App, Entity, Task};
use language_model::{LanguageModelRequestMessage, LanguageModelToolSchemaFormat};
use project::Project;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ui::IconName;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct McpResourcesListToolInput {
    /// The name of the MCP server to list resources from. If not specified, lists resources from all servers.
    #[serde(default)]
    server_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResourceInfo {
    server: String,
    uri: String,
    name: String,
    description: Option<String>,
    mime_type: Option<String>,
}

pub struct McpResourcesListTool {
    context_server_manager: Entity<ContextServerManager>,
}

impl McpResourcesListTool {
    pub fn new(context_server_manager: Entity<ContextServerManager>) -> Self {
        Self {
            context_server_manager,
        }
    }
}

impl Tool for McpResourcesListTool {
    fn name(&self) -> String {
        "mcp_resources_list".into()
    }

    fn needs_confirmation(&self, _: &serde_json::Value, _: &App) -> bool {
        false
    }

    fn description(&self) -> String {
        "List available resources from Model Context Protocol (MCP) servers. Returns resource URIs, names, descriptions, and MIME types.".into()
    }

    fn icon(&self) -> IconName {
        IconName::FileTree
    }

    fn input_schema(&self, format: LanguageModelToolSchemaFormat) -> Result<serde_json::Value> {
        json_schema_for::<McpResourcesListToolInput>(format)
    }

    fn ui_text(&self, input: &serde_json::Value) -> String {
        let input: McpResourcesListToolInput = serde_json::from_value(input.clone()).unwrap_or_default();
        match input.server_name {
            Some(server) => format!("List MCP resources from {}", server),
            None => "List all MCP resources".to_string(),
        }
    }

    fn run(
        self: Arc<Self>,
        input: serde_json::Value,
        _messages: &[LanguageModelRequestMessage],
        _project: Entity<Project>,
        _action_log: Entity<ActionLog>,
        cx: &mut App,
    ) -> ToolResult {
        let input: McpResourcesListToolInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => return Task::ready(Err(anyhow!(err))).into(),
        };

        let context_server_manager = self.context_server_manager.clone();
        let task = cx.spawn(async move |cx| {
            let servers = cx.update(|cx| {
                let manager = context_server_manager.read(cx);
                if let Some(server_name) = &input.server_name {
                    manager.get_server(server_name).map(|s| vec![s]).unwrap_or_default()
                } else {
                    manager.all_servers()
                }
            })?;

            let mut all_resources = Vec::new();

            for server in servers {
                let server_id = server.id().to_string();
                if let Some(protocol) = server.client() {
                    if protocol.capable(context_server::protocol::ServerCapability::Resources) {
                        if let Ok(response) = protocol.list_resources().await {
                            for resource in response.resources {
                                all_resources.push(ResourceInfo {
                                    server: server_id.clone(),
                                    uri: resource.uri.to_string(),
                                    name: resource.name,
                                    description: resource.description,
                                    mime_type: resource.mime_type,
                                });
                            }
                        }
                    }
                }
            }

            if all_resources.is_empty() {
                return Ok("No MCP resources available.".to_string());
            }

            let mut output = String::new();
            output.push_str("Available MCP resources:\n\n");

            // Group by server
            let mut by_server: std::collections::HashMap<String, Vec<&ResourceInfo>> = std::collections::HashMap::new();
            for resource in &all_resources {
                by_server.entry(resource.server.clone()).or_default().push(resource);
            }

            for (server, resources) in by_server {
                output.push_str(&format!("Server: {}\n", server));
                for resource in resources {
                    output.push_str(&format!("  - {} ({})", resource.uri, resource.name));
                    if let Some(mime) = &resource.mime_type {
                        output.push_str(&format!(" [{}]", mime));
                    }
                    if let Some(desc) = &resource.description {
                        output.push_str(&format!("\n    {}", desc));
                    }
                    output.push('\n');
                }
                output.push('\n');
            }

            Ok(output)
        });

        task.into()
    }
}

impl Default for McpResourcesListToolInput {
    fn default() -> Self {
        Self {
            server_name: None,
        }
    }
}