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
pub struct McpPromptsListToolInput {
    /// The name of the MCP server to list prompts from. If not specified, lists prompts from all servers.
    #[serde(default)]
    server_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PromptInfo {
    server: String,
    name: String,
    description: Option<String>,
    arguments: Vec<PromptArgument>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PromptArgument {
    name: String,
    description: Option<String>,
    required: bool,
}

pub struct McpPromptsListTool {
    context_server_store: Entity<ContextServerStore>,
}

impl McpPromptsListTool {
    pub fn new(context_server_store: Entity<ContextServerStore>) -> Self {
        Self {
            context_server_store,
        }
    }
}

impl Tool for McpPromptsListTool {
    fn name(&self) -> String {
        "mcp_prompts_list".into()
    }

    fn needs_confirmation(&self, _: &serde_json::Value, _: &App) -> bool {
        false
    }

    fn description(&self) -> String {
        "List available prompts from Model Context Protocol (MCP) servers. Returns prompt names, descriptions, and required arguments.".into()
    }

    fn icon(&self) -> IconName {
        IconName::ListTree
    }

    fn input_schema(&self, format: LanguageModelToolSchemaFormat) -> Result<serde_json::Value> {
        json_schema_for::<McpPromptsListToolInput>(format)
    }

    fn may_perform_edits(&self) -> bool {
        false
    }

    fn ui_text(&self, input: &serde_json::Value) -> String {
        let input: McpPromptsListToolInput = serde_json::from_value(input.clone()).unwrap_or_default();
        match input.server_name {
            Some(server) => format!("List MCP prompts from {}", server),
            None => "List all MCP prompts".to_string(),
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
        let input: McpPromptsListToolInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => return Task::ready(Err(anyhow!(err))).into(),
        };

        let _context_server_store = self.context_server_store.clone();
        let task = cx.spawn(async move |_cx| {
            // TODO: Implement proper MCP prompt listing
            // For now, return a placeholder error since the API is not fully implemented
            let server_msg = match &input.server_name {
                Some(name) => format!("from server '{}'", name),
                None => "from all servers".to_string(),
            };
            Err(anyhow!("MCP prompt listing not yet implemented {}", server_msg))
        });

        task.into()
    }
}

impl Default for McpPromptsListToolInput {
    fn default() -> Self {
        Self {
            server_name: None,
        }
    }
}