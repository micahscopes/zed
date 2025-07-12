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
    context_server_manager: Entity<ContextServerManager>,
}

impl McpPromptsListTool {
    pub fn new(context_server_manager: Entity<ContextServerManager>) -> Self {
        Self {
            context_server_manager,
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
        _messages: &[LanguageModelRequestMessage],
        _project: Entity<Project>,
        _action_log: Entity<ActionLog>,
        cx: &mut App,
    ) -> ToolResult {
        let input: McpPromptsListToolInput = match serde_json::from_value(input) {
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

            let mut all_prompts = Vec::new();

            for server in servers {
                let server_id = server.id().to_string();
                if let Some(protocol) = server.client() {
                    if protocol.capable(context_server::protocol::ServerCapability::Prompts) {
                        if let Ok(prompts) = protocol.list_prompts().await {
                            for prompt in prompts {
                                all_prompts.push(PromptInfo {
                                    server: server_id.clone(),
                                    name: prompt.name,
                                    description: prompt.description,
                                    arguments: prompt.arguments.unwrap_or_default().into_iter().map(|arg| {
                                        PromptArgument {
                                            name: arg.name,
                                            description: arg.description,
                                            required: arg.required.unwrap_or(false),
                                        }
                                    }).collect(),
                                });
                            }
                        }
                    }
                }
            }

            if all_prompts.is_empty() {
                return Ok("No MCP prompts available.".to_string());
            }

            let mut output = String::new();
            output.push_str("Available MCP prompts:\n\n");

            for prompt in all_prompts {
                output.push_str(&format!("Server: {}\n", prompt.server));
                output.push_str(&format!("Prompt: {}\n", prompt.name));
                if let Some(desc) = &prompt.description {
                    output.push_str(&format!("Description: {}\n", desc));
                }
                if !prompt.arguments.is_empty() {
                    output.push_str("Arguments:\n");
                    for arg in &prompt.arguments {
                        output.push_str(&format!("  - {} ({})", arg.name, if arg.required { "required" } else { "optional" }));
                        if let Some(desc) = &arg.description {
                            output.push_str(&format!(": {}", desc));
                        }
                        output.push('\n');
                    }
                }
                output.push('\n');
            }

            Ok(output)
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