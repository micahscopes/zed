use std::sync::Arc;

use crate::schema::json_schema_for;
use anyhow::{Result, anyhow};
use assistant_tool::{ActionLog, Tool, ToolResult};
use project::context_server_store::ContextServerStore;
use gpui::{AnyWindowHandle, App, Entity, Task};
use language_model::{LanguageModel, LanguageModelRequest, LanguageModelToolSchemaFormat};
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
    name: Option<String>,
    description: Option<String>,
    mime_type: Option<String>,
}

pub struct McpResourcesListTool {
    context_server_store: Entity<ContextServerStore>,
}

impl McpResourcesListTool {
    pub fn new(context_server_store: Entity<ContextServerStore>) -> Self {
        Self {
            context_server_store,
        }
    }
}

impl Tool for McpResourcesListTool {
    fn name(&self) -> String {
        "mcp_resources_list".into()
    }

    fn description(&self) -> String {
        "List all available MCP resources from connected servers".into()
    }

    fn icon(&self) -> IconName {
        IconName::ListTree
    }

    fn needs_confirmation(&self, _: &serde_json::Value, _: &App) -> bool {
        false
    }

    fn may_perform_edits(&self) -> bool {
        false
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
        _request: Arc<LanguageModelRequest>,
        _project: Entity<Project>,
        _action_log: Entity<ActionLog>,
        _model: Arc<dyn LanguageModel>,
        _window: Option<AnyWindowHandle>,
        cx: &mut App,
    ) -> ToolResult {
        let input: McpResourcesListToolInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => return Task::ready(Err(anyhow!(err))).into(),
        };

        let _context_server_store = self.context_server_store.clone();
        let task = cx.spawn(async move |_cx| {
            // TODO: Implement proper MCP resource listing
            let server_msg = match &input.server_name {
                Some(name) => format!("from server '{}'", name),
                None => "from all servers".to_string(),
            };
            Err(anyhow!("MCP resource listing not yet implemented {}", server_msg))
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