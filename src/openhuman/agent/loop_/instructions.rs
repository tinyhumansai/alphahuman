use crate::openhuman::tools::Tool;
use std::fmt::Write;

/// Build the tool instruction block for the system prompt so the LLM knows
/// how to invoke tools.
pub(crate) fn build_tool_instructions(tools_registry: &[Box<dyn Tool>]) -> String {
    let mut instructions = String::new();
    instructions.push_str("\n## Tool Use Protocol\n\n");
    instructions.push_str("To use a tool, wrap a JSON object in <tool_call></tool_call> tags:\n\n");
    instructions.push_str("```\n<tool_call>\n{\"name\": \"tool_name\", \"arguments\": {\"param\": \"value\"}}\n</tool_call>\n```\n\n");

    // Explicit anti-hallucination rules
    instructions.push_str("### Rules (MUST follow)\n\n");
    instructions.push_str(
        "1. **ALWAYS use <tool_call> tags** when a task requires a tool. \
         NEVER narrate what you would do — emit the actual tags.\n",
    );
    instructions.push_str(
        "2. **NEVER describe a tool call in prose** (e.g. \"I'll run ls\" or \
         \"Let me check the file\") without also emitting the <tool_call> tags. \
         If you mention a tool, you must call it.\n",
    );
    instructions.push_str(
        "3. **Use the exact tool names** listed below. \
         Do not invent tool names that are not in the list.\n",
    );
    instructions.push_str("4. You may use **multiple tool calls** in a single response.\n");
    instructions.push_str(
        "5. After tool execution, results appear in <tool_result> tags. \
         Continue reasoning with the results until you can give a final answer.\n",
    );
    instructions.push_str(
        "6. Only respond **without** a tool call when the answer requires \
         no tool (e.g. general knowledge, math, or conversation).\n\n",
    );

    instructions.push_str("### Example\n\n");
    instructions.push_str("User: \"what's the date?\"\nCorrect response:\n<tool_call>\n{\"name\":\"shell\",\"arguments\":{\"command\":\"date\"}}\n</tool_call>\n\n");

    instructions.push_str("### Available Tools\n\n");

    for tool in tools_registry {
        let _ = writeln!(
            instructions,
            "**{}**: {}\nParameters: `{}`\n",
            tool.name(),
            tool.description(),
            tool.parameters_schema()
        );
    }

    instructions
}
