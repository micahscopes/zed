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
pub struct McpResourcesReadToolInput {
    /// The URI of the resource to read
    uri: String,
}

pub struct McpResourcesReadTool {
    context_server_store: Entity<ContextServerStore>,
}

impl McpResourcesReadTool {
    pub fn new(context_server_store: Entity<ContextServerStore>) -> Self {
        Self {
            context_server_store,
        }
    }
}

impl Tool for McpResourcesReadTool {
    fn name(&self) -> String {
        "mcp_resources_read".into()
    }

    fn description(&self) -> String {
        "Read the contents of an MCP resource".into()
    }

    fn icon(&self) -> IconName {
        IconName::File
    }

    fn needs_confirmation(&self, _: &serde_json::Value, _: &App) -> bool {
        false
    }

    fn may_perform_edits(&self) -> bool {
        false
    }

    fn input_schema(&self, format: LanguageModelToolSchemaFormat) -> Result<serde_json::Value> {
        json_schema_for::<McpResourcesReadToolInput>(format)
    }

    fn ui_text(&self, input: &serde_json::Value) -> String {
        let input: McpResourcesReadToolInput = serde_json::from_value(input.clone()).unwrap_or_default();
        format!("Read MCP resource: {}", input.uri)
    }

    fn run(
        self: Arc<Self>,
        input: serde_json::Value,
        _request: Arc<LanguageModelRequest>,
        _project: Entity<Project>,
        _action_log: Entity<ActionLog>,
        _model: Arc<dyn LanguageModel>,
        _window: Option<AnyWindowHandle>,
        _cx: &mut App,
    ) -> ToolResult {
        let input: McpResourcesReadToolInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => return Task::ready(Err(anyhow!(err))).into(),
        };

        // TODO: Implement when resources/read is added to the protocol implementation
        let error_msg = format!(
            "Reading MCP resources is not yet implemented. Resource URI: {}",
            input.uri
        );
        Task::ready(Err(anyhow!(error_msg))).into()
    }
}

impl Default for McpResourcesReadToolInput {
    fn default() -> Self {
        Self {
            uri: String::new(),
        }
    }
}