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
use serde_json::Value;
use ui::IconName;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct McpPromptsGetToolInput {
    /// The name of the MCP server that has the prompt
    server_name: String,
    /// The name of the prompt to retrieve
    prompt_name: String,
    /// Arguments to pass to the prompt
    #[serde(default)]
    arguments: Option<Value>,
}

pub struct McpPromptsGetTool {
    context_server_store: Entity<ContextServerStore>,
}

impl McpPromptsGetTool {
    pub fn new(context_server_store: Entity<ContextServerStore>) -> Self {
        Self {
            context_server_store,
        }
    }
}

impl Tool for McpPromptsGetTool {
    fn name(&self) -> String {
        "mcp_prompts_get".into()
    }

    fn needs_confirmation(&self, _: &serde_json::Value, _: &App) -> bool {
        false
    }

    fn description(&self) -> String {
        "Get a specific prompt from an MCP server with optional arguments. Returns the prompt messages ready to be used.".into()
    }

    fn icon(&self) -> IconName {
        IconName::FileCode
    }

    fn input_schema(&self, format: LanguageModelToolSchemaFormat) -> Result<serde_json::Value> {
        json_schema_for::<McpPromptsGetToolInput>(format)
    }

    fn ui_text(&self, input: &serde_json::Value) -> String {
        let input: McpPromptsGetToolInput = serde_json::from_value(input.clone()).unwrap_or_default();
        format!("Get MCP prompt '{}' from {}", input.prompt_name, input.server_name)
    }

    fn may_perform_edits(&self) -> bool {
        false
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
        let input: McpPromptsGetToolInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => return Task::ready(Err(anyhow!(err))).into(),
        };

        let _context_server_store = self.context_server_store.clone();
        let task = cx.spawn(async move |_cx| {
            // TODO: Implement proper MCP prompt retrieval
            // For now, return a placeholder error since the API is not fully implemented
            Err(anyhow!("MCP prompt retrieval not yet implemented for server '{}', prompt '{}'", input.server_name, input.prompt_name))
        });

        task.into()
    }
}

impl Default for McpPromptsGetToolInput {
    fn default() -> Self {
        Self {
            server_name: String::new(),
            prompt_name: String::new(),
            arguments: None,
        }
    }
}