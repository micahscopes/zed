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
    context_server_manager: Entity<ContextServerManager>,
}

impl McpPromptsGetTool {
    pub fn new(context_server_manager: Entity<ContextServerManager>) -> Self {
        Self {
            context_server_manager,
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

    fn run(
        self: Arc<Self>,
        input: serde_json::Value,
        _messages: &[LanguageModelRequestMessage],
        _project: Entity<Project>,
        _action_log: Entity<ActionLog>,
        cx: &mut App,
    ) -> ToolResult {
        let input: McpPromptsGetToolInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => return Task::ready(Err(anyhow!(err))).into(),
        };

        let context_server_manager = self.context_server_manager.clone();
        let task = cx.spawn(async move |cx| {
            let server = cx.update(|cx| {
                context_server_manager.read(cx).get_server(&input.server_name)
            })?;

            let server = server.ok_or_else(|| anyhow!("MCP server '{}' not found", input.server_name))?;
            
            let protocol = server.client()
                .ok_or_else(|| anyhow!("MCP server '{}' is not initialized", input.server_name))?;

            if !protocol.capable(context_server::protocol::ServerCapability::Prompts) {
                return Err(anyhow!("MCP server '{}' does not support prompts", input.server_name));
            }

            // Convert arguments to HashMap<String, String> if provided
            let arguments = if let Some(args) = input.arguments {
                match args {
                    Value::Object(map) => {
                        let mut string_map = std::collections::HashMap::new();
                        for (k, v) in map {
                            let value_str = match v {
                                Value::String(s) => s,
                                _ => v.to_string(),
                            };
                            string_map.insert(k, value_str);
                        }
                        string_map
                    }
                    _ => std::collections::HashMap::new(),
                }
            } else {
                std::collections::HashMap::new()
            };

            let response = protocol.run_prompt(&input.prompt_name, arguments).await
                .map_err(|e| anyhow!("Failed to get prompt: {}", e))?;

            let mut output = String::new();
            
            if let Some(desc) = &response.description {
                output.push_str(&format!("Prompt: {}\n", desc));
                output.push_str("---\n");
            }

            output.push_str("Messages:\n\n");

            for message in &response.messages {
                output.push_str(&format!("Role: {:?}\n", message.role));
                output.push_str("Content:\n");
                
                match &message.content {
                    context_server::types::MessageContent::Text { text, annotations: _ } => {
                        output.push_str(&text);
                    }
                    context_server::types::MessageContent::Resource { resource, annotations: _ } => {
                        output.push_str(&format!("Resource: {}\n", resource.uri));
                        if let Some(mime_type) = &resource.mime_type {
                            output.push_str(&format!("MIME type: {}\n", mime_type));
                        }
                        // Note: ResourceContents doesn't have text/blob fields - those are in the response
                        output.push_str("[Resource content would be fetched separately]\n");
                    }
                    context_server::types::MessageContent::Image { data, mime_type, annotations: _ } => {
                        output.push_str(&format!("[Image: {} bytes ({})]", data.len(), mime_type));
                    }
                }
                output.push_str("\n\n");
            }

            Ok(output)
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