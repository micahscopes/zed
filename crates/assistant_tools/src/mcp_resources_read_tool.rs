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
pub struct McpResourcesReadToolInput {
    /// The name of the MCP server that has the resource
    server_name: String,
    /// The URI of the resource to read
    resource_uri: String,
}

pub struct McpResourcesReadTool {
    context_server_manager: Entity<ContextServerManager>,
}

impl McpResourcesReadTool {
    pub fn new(context_server_manager: Entity<ContextServerManager>) -> Self {
        Self {
            context_server_manager,
        }
    }
}

impl Tool for McpResourcesReadTool {
    fn name(&self) -> String {
        "mcp_resources_read".into()
    }

    fn needs_confirmation(&self, _: &serde_json::Value, _: &App) -> bool {
        false
    }

    fn description(&self) -> String {
        "Read the content of a specific resource from an MCP server. NOTE: This functionality is not yet implemented in Zed's MCP integration.".into()
    }

    fn icon(&self) -> IconName {
        IconName::FileDoc
    }

    fn input_schema(&self, format: LanguageModelToolSchemaFormat) -> Result<serde_json::Value> {
        json_schema_for::<McpResourcesReadToolInput>(format)
    }

    fn ui_text(&self, input: &serde_json::Value) -> String {
        let input: McpResourcesReadToolInput = serde_json::from_value(input.clone()).unwrap_or_default();
        format!("Read MCP resource '{}' from {}", input.resource_uri, input.server_name)
    }

    fn run(
        self: Arc<Self>,
        input: serde_json::Value,
        _messages: &[LanguageModelRequestMessage],
        _project: Entity<Project>,
        _action_log: Entity<ActionLog>,
        _cx: &mut App,
    ) -> ToolResult {
        let input: McpResourcesReadToolInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => return Task::ready(Err(anyhow!(err))).into(),
        };

        // TODO: Implement when resources/read is added to the protocol implementation
        let error_msg = format!(
            "Reading MCP resources is not yet implemented in Zed. \
            The resource '{}' from server '{}' cannot be read at this time. \
            You can see available resources using mcp_resources_list.",
            input.resource_uri, input.server_name
        );

        Task::ready(Err(anyhow!(error_msg))).into()
    }
}

impl Default for McpResourcesReadToolInput {
    fn default() -> Self {
        Self {
            server_name: String::new(),
            resource_uri: String::new(),
        }
    }
}