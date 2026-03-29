# Best Practices for Prompt Engineering in Sub-Agents

This document compiles best practices for writing effective prompts when delegating tasks to AI sub-agents. Focus is on AI-to-AI delegation patterns that maximize clarity and minimize errors. All guidance is language-agnostic and applicable to any programming language or coding project.

## Table of Contents

1. [Prompt Structure & Formatting](#1-prompt-structure--formatting)
2. [Instruction Clarity](#2-instruction-clarity)
3. [Input/Output Contracts](#3-inputoutput-contracts)
4. [Context Management](#4-context-management)
5. [Constraint Specification](#5-constraint-specification)
6. [Sub-Agent Specific Patterns](#6-sub-agent-specific-patterns)
7. [Appendix: Naming Conventions Reference Table](#7-appendix-naming-conventions-reference-table)

## 1. Prompt Structure & Formatting

### 1.1 Use Hierarchical Section Headers

Headers act as semantic markers that segment the prompt into logical units, reducing ambiguity.

```markdown
# Identity
You are a code analyzer specializing in code quality assessment.

# Task
Analyze the provided file for code quality issues.

# Scope
- Naming convention violations (see Naming Conventions Reference Table)
- Function size (flag functions > 10 lines)
- Structural problems

# Output Format
Return a JSON array of issues found.

# Reference
Apply conventions from: {{style_guide_path}}
```

### 1.2 Use Delimiters to Separate Content Sections

Delimiters prevent the sub-agent from confusing input data with instructions.

```markdown
# Task
Analyze the following code for naming convention issues.

<source_code>
define function "initialize":
    set x to 1
    set y to 2
    print x + y
</source_code>

# Conventions to Check
- Variables should use descriptive names appropriate for the project's language
- Avoid single-letter variable names except for loop indices
```

### 1.3 Place Instructions Before Data

Sub-agents process prompts sequentially; instructions first ensures the task is understood before encountering data.

```markdown
# Task
Analyze the provided source file for Single Responsibility Principle violations.

# Analysis Criteria
1. Identify all responsibilities handled by the class
2. Flag classes with more than one clear responsibility
3. Suggest how to split if violations found

# Input
<file_content>
define class "User" that inherits from "BaseEntity"
...
</file_content>
```

### 1.4 Use Consistent Formatting Throughout

Inconsistent formatting (mixing bullets, numbers, headers) causes the sub-agent to miss or misapply instructions.

```markdown
# Checklist
1. Naming conventions (per project language — see Naming Conventions Reference Table)
2. Function length (max 10 lines)
3. Code duplication (flag identical blocks > 3 lines)
4. Unused variables
5. Missing type annotations
```

### 1.5 Optimize Prompt Length

Include only relevant, actionable information — long enough to be unambiguous, short enough to avoid redundancy.

```markdown
# Role
Code reviewer

# Task
Review the provided code for bugs and style violations.

# Focus Areas
- Null reference risks
- Type safety issues
- Style guide compliance

<code>
{{code_content}}
</code>
```

## 2. Instruction Clarity

### 2.1 Use Positive Framing ("Do X" instead of "Don't Do Y")

Negative instructions leave room for interpretation; positive instructions provide clear direction.

```markdown
# Instructions
1. Use precise, specific language in all outputs
2. Process every file in the provided list
3. Validate each file for syntax errors before analysis
4. Base all conclusions only on explicitly provided information
```

**Avoid:** Conflicting instructions (e.g., "be thorough" combined with "keep it brief"). When depth and brevity conflict, specify exactly what to include at each severity level.

### 2.2 Provide Numbered Step-by-Step Instructions

Sequential numbered steps create an unambiguous execution order.

```markdown
# Execution Steps

1. **Discover**: List all source files (matching `*.<ext>`) in the `<source_dir>/` directory
2. **Parse**: For each file, extract class name, methods, and dependencies
3. **Analyze**: Check each method against the quality criteria
4. **Categorize**: Group issues by severity (Critical, Warning, Info)
5. **Validate**: Verify each suggested fix doesn't break dependencies
6. **Report**: Generate the output in the specified JSON format
```

### 2.3 Use Few-Shot Examples for Complex Outputs

Examples demonstrate the exact format and style expected and disambiguate instructions.

```markdown
# Output Format
Return results as a JSON array. Each issue follows this structure:

## Example Output
<example_output>
{
  "issues": [
    {
      "file": "<source_dir>/user.<ext>",
      "line": 45,
      "severity": "warning",
      "rule": "function-length",
      "message": "Function 'process_order' has 15 lines (max: 10)",
      "suggestion": "Extract lines 50-60 into a separate method"
    }
  ],
  "summary": {
    "total_issues": 1,
    "by_severity": {"critical": 0, "warning": 1, "info": 0}
  }
}
</example_output>
```

### 2.4 Specify Explicit Decision Criteria

Explicit thresholds remove ambiguity from judgment calls like "too long" or "significant."

```markdown
# Decision Criteria

| Check | Threshold | Action |
|-------|-----------|--------|
| Function length | > 10 lines | Flag as warning |
| Function length | > 20 lines | Flag as critical |
| Code duplication | > 3 identical lines | Flag as warning |
| Cyclomatic complexity | > 5 | Flag as warning |
| Cyclomatic complexity | > 10 | Flag as critical |
```

**Avoid:** Ambiguous quantifiers like "some," "few," "many," "usually." Use specific numbers or percentages.

### 2.5 Define Domain-Specific Terms

Terms like "module," "component," or "service" may mean different things in different contexts.

```markdown
# Definitions
- **Module**: A single source file that defines one class or cohesive unit of functionality
- **Component**: A composable class that encapsulates a single behavior or capability
- **Service**: A singleton that provides shared functionality across the application
- **Coupling**: Direct dependencies between modules (imports, references)

# Task
Analyze each module for coupling issues with other modules.
```

## 3. Input/Output Contracts

### 3.1 Define Input Schema Explicitly

Sub-agents need to know exactly what data they're receiving, its structure, and any constraints.

```markdown
# Input Schema

You will receive input in the following format:

<input_schema>
{
  "files": [
    {
      "path": "string (relative path from project root)",
      "content": "string (full file content)",
      "language": "string (e.g., 'python', 'typescript', 'rust')"
    }
  ],
  "config": {
    "max_function_lines": "integer (default: 10)",
    "check_types": "boolean (default: true)"
  }
}
</input_schema>

# Input Data
<input>
{{input_json}}
</input>
```

### 3.2 Specify Output Format with JSON Schema

Structured output enables the orchestrator to parse and act on sub-agent responses programmatically.

```markdown
# Output Requirements

Respond with **only** valid JSON matching this schema:

<output_schema>
{
  "type": "object",
  "required": ["status", "results"],
  "properties": {
    "status": {
      "type": "string",
      "enum": ["success", "partial", "error"]
    },
    "results": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["file", "issues"],
        "properties": {
          "file": {"type": "string"},
          "issues": {"type": "array"}
        }
      }
    },
    "error_message": {
      "type": "string",
      "description": "Only present if status is 'error'"
    }
  }
}
</output_schema>

Do not include any text before or after the JSON.
```

### 3.3 Include Error Handling Instructions

Without error handling instructions, sub-agents may fail silently or produce invalid output on edge cases.

```markdown
# Error Handling

| Scenario | Action |
|----------|--------|
| Empty file | Return `{"status": "success", "results": [], "note": "File is empty"}` |
| Parse error | Return `{"status": "error", "error_message": "Failed to parse: <reason>"}` |
| Unsupported language | Return `{"status": "error", "error_message": "Unsupported language: <lang>"}` |
| File not found | Return `{"status": "error", "error_message": "File not found: <path>"}` |

Never return unstructured error messages. Always use the JSON format.
```

**Avoid:** Missing edge case handling. Explicitly handle empty files, missing data, binary files, and multi-class files.

### 3.4 Define Validation Criteria

Validation criteria help sub-agents self-check their output before returning it.

```markdown
# Output Validation

Before returning your response, verify:

1. [ ] JSON is valid (no trailing commas, proper quotes)
2. [ ] All required fields are present
3. [ ] `status` is one of: "success", "partial", "error"
4. [ ] Every issue has: file, line, severity, message
5. [ ] Line numbers are positive integers
6. [ ] File paths match the input paths exactly
```

## 4. Context Management

### 4.1 Include Only Relevant Context

Unnecessary context consumes tokens, increases latency, and distracts from the actual task.

```markdown
# Context
- Project style guide: {{style_guide_excerpt}}
- Naming conventions: per project language (see Naming Conventions Reference Table)

# Task
Analyze the following file for naming convention violations.

<file path="<source_dir>/user.<ext>">
{{user_file_content}}
</file>
```

**Avoid:** Implicit assumptions. Always state file locations, entry points, language, and version explicitly.

### 4.2 Use Layered Context Architecture

Organizing context into layers (system, task, tool, memory) helps sub-agents understand the hierarchy and scope.

```markdown
# System Layer
You are a code quality analyzer.

# Task Layer
Analyze the provided file for Single Responsibility Principle violations.

# Tool Layer
Available tools:
- `read_file(path)`: Read file contents
- `grep_search(pattern, path)`: Search for patterns

# Memory Layer
Previous analysis context:
- Files with known coupling issues: <source_dir>/user.<ext>, <source_dir>/order.<ext>
```

### 4.3 Reference External Resources by Identifier

Keep prompts focused by referencing large documents by identifier instead of embedding them inline.

```markdown
# Reference Documents
- Style guide: Available via `read_file("docs/STYLE_GUIDE.md")`
- Architecture doc: Available via `read_file("docs/ARCHITECTURE.md")`

# Task
Check if the provided code follows the conventions in the style guide.
Fetch the style guide content if needed for specific rules.
```

### 4.4 Explicitly State Task Boundaries

Sub-agents should know exactly what is in-scope and out-of-scope.

```markdown
# Scope

## In Scope
- Naming convention violations
- Function length violations
- Missing type annotations

## Out of Scope (Do Not Check)
- Performance optimization
- Algorithm correctness
- Test coverage
- Documentation completeness

## Boundaries
- Only analyze files in `<source_dir>/` directory
- Skip files with `_test.<ext>` suffix
- Ignore third-party and generated directories
```

## 5. Constraint Specification

### 5.1 Define Tool Usage Restrictions

Sub-agents with access to tools need clear guidance on when to use them, in what order, and with what limitations.

```markdown
# Tool Usage

## Available Tools
| Tool | Purpose | Restrictions |
|------|---------|--------------|
| `read_file` | Read file contents | Max 5 calls per task |
| `grep_search` | Search codebase | Use before read_file to find targets |
| `list_dir` | List directory contents | Use only for discovery phase |

## Tool Priorities
1. Use `grep_search` to find relevant files first
2. Use `list_dir` only if grep returns no results
3. Use `read_file` only for files you will actually analyze

## Forbidden Actions
- Do not use `run_in_terminal`
- Do not modify any files
- Do not access files outside `<source_dir>/` directory
```

### 5.2 Set Quality Thresholds

Quality thresholds set expectations and prevent superficial or overly detailed reports.

```markdown
# Quality Requirements

## Minimum Requirements
- Analyze at least 80% of functions in each file
- Provide specific line numbers for all issues
- Include actionable fix suggestions for critical issues

## Maximum Limits
- Maximum 20 issues per file (prioritize by severity)
- Maximum 3 sentences per issue description
- Maximum 1 code example per suggestion

## Completeness Criteria
Mark analysis as complete only when:
- All functions have been checked
- All issues have severity assigned
- Summary statistics are accurate
```

**Avoid:** Over-constraining with rigid rules like "ALWAYS report exactly 10 issues." Use ranges and budgets instead.

### 5.3 Specify Time/Iteration Limits

Explicit limits prevent sub-agents from getting stuck in loops or spending excessive time on edge cases.

```markdown
# Execution Limits

- Maximum file reads: 10
- Maximum retry attempts per file: 2
- If analysis takes more than 5 tool calls, return partial results
- If a file cannot be parsed after 2 attempts, skip and log error

# Partial Results Protocol
If limits are reached:
1. Return all completed analysis
2. Set status to "partial"
3. Include list of incomplete items in "pending" field
```

## 6. Sub-Agent Specific Patterns

### 6.1 Pattern: Task Handoff Protocol

A clear handoff protocol ensures the sub-agent has everything needed and knows exactly what to return.

```markdown
# Task Handoff

## Task ID
TASK-001-analyze-user

## Delegated By
Orchestrator Agent (Phase 2: Detailed Analysis)

## Your Role
File Analyzer Sub-Agent

## Input Provided
<task_input>
{
  "file_path": "<source_dir>/user.<ext>",
  "analysis_type": "srp_check",
  "context": {
    "related_files": ["<source_dir>/user_service.<ext>", "<source_dir>/order_service.<ext>"]
  }
}
</task_input>

## Expected Output
Return JSON with your analysis results.

## Return Protocol
1. Complete the analysis
2. Format output per schema
3. Return ONLY the JSON output
4. Do not include commentary before/after JSON
```

### 6.2 Pattern: Progress Reporting for Long Tasks

Progress reporting helps orchestrators track status and handle partial failures.

```markdown
# Progress Reporting

For each file analyzed, emit a progress update:

<progress_format>
{
  "task_id": "TASK-001",
  "progress": {
    "completed": 3,
    "total": 10,
    "current_file": "<source_dir>/order.<ext>",
    "status": "in_progress"
  }
}
</progress_format>

Final response should include:
{
  "task_id": "TASK-001",
  "progress": {"completed": 10, "total": 10, "status": "complete"},
  "results": [...]
}
```

### 6.3 Pattern: Dependency Declaration

Explicit dependency declaration prevents execution of tasks with missing prerequisites.

```markdown
# Task Dependencies

## This Task
ID: TASK-003-detailed-analysis

## Depends On
- TASK-001-file-discovery (required: file list)
- TASK-002-initial-scan (required: severity classifications)

## Required Inputs from Dependencies
<dependency_inputs>
{
  "from_task_001": {
    "required_fields": ["file_list"],
    "expected_type": "array of file paths"
  },
  "from_task_002": {
    "required_fields": ["severity_map"],
    "expected_type": "object mapping file paths to severity"
  }
}
</dependency_inputs>

## Failure Protocol
If dependency outputs are missing or invalid:
1. Return status: "blocked"
2. Include missing_dependencies array
3. Do not attempt partial execution
```

### 6.4 Pattern: Idempotency Requirements

Sub-agents may be retried due to failures or timeouts; idempotent operations produce the same result regardless of retry count.

```markdown
# Idempotency

This task must be idempotent:

## Requirements
- Reading the same file twice produces identical output
- Analysis results depend only on input, not on execution time
- No side effects (do not write files, do not store state)

## Verification
Given identical inputs, your output must be byte-for-byte identical.

## Anti-Patterns to Avoid
- Including timestamps in output
- Using random sampling
- Depending on external state that may change
```

### 6.5 Pattern: Result Aggregation Hints

Aggregation hints simplify the orchestrator's job when merging results from parallel sub-agents.

```markdown
# Aggregation Metadata

Include this metadata to help with result aggregation:

<aggregation_hints>
{
  "merge_strategy": "concatenate",  // or "deduplicate", "union", "intersect"
  "sort_key": "severity",           // field to sort aggregated results by
  "group_by": "file",               // field to group results by
  "conflict_resolution": "keep_highest_severity"
}
</aggregation_hints>
```

## Summary: Quick Reference Checklist

Use this checklist when writing sub-agent prompts:

### Structure

- [ ] Clear section headers (Identity, Task, Input, Output)
- [ ] Delimiters separating instructions from data
- [ ] Instructions placed before data
- [ ] Consistent formatting throughout

### Clarity

- [ ] Positive framing (do X, not don't Y)
- [ ] Numbered step-by-step instructions
- [ ] Few-shot examples for complex outputs
- [ ] Explicit decision criteria with thresholds
- [ ] Domain terms defined

### Contracts

- [ ] Input schema explicitly defined
- [ ] Output format specified with schema
- [ ] Error handling instructions included
- [ ] Validation criteria stated

### Context

- [ ] Only relevant context included
- [ ] External resources referenced by ID
- [ ] Task boundaries explicitly stated
- [ ] In-scope vs out-of-scope defined

### Constraints

- [ ] Tool usage restrictions specified
- [ ] Quality thresholds defined
- [ ] Time/iteration limits set
- [ ] Partial result protocol included

### Sub-Agent Patterns

- [ ] Task handoff protocol followed
- [ ] Dependencies declared
- [ ] Idempotency requirements stated
- [ ] Aggregation hints provided (if applicable)

## 7. Appendix: Naming Conventions Reference Table

When writing prompts that check naming conventions, reference the appropriate conventions for the project's language.

### Rust

- Variables: snake_case (`let user_name`)
- Functions: snake_case (`fn get_user()`)
- Types / Structs / Enums: PascalCase (`struct UserAccount`)
- Constants: SCREAMING_SNAKE_CASE (`const MAX_RETRIES`)
- Modules / Files: snake_case (`user_account.rs`)
- Crates / Packages: snake_case with hyphens in Cargo.toml (`my-crate` / `my_crate`)

### C\#

- Variables (local): camelCase (`var userName`)
- Functions / Methods: PascalCase (`GetUser()`)
- Classes / Types: PascalCase (`class UserAccount`)
- Constants: PascalCase (`const int MaxRetries`)
- Files: PascalCase matching class name (`UserAccount.cs`)
- Namespaces: PascalCase (`MyApp.Services`)

### Python

- Variables: snake_case (`user_name`)
- Functions: snake_case (`def get_user()`)
- Classes: PascalCase (`class UserAccount`)
- Constants: SCREAMING_SNAKE_CASE (`MAX_RETRIES`)
- Modules / Files: snake_case (`user_account.py`)
- Packages: snake_case, short, no underscores preferred (`bookstore`)

### SQL

- Variables / Aliases: snake_case (`user_name`)
- Functions / Procedures: snake_case or PascalCase, varies by dialect (`get_user()`)
- Tables: snake_case, plural (`user_accounts`)
- Constants / Enum values: SCREAMING_SNAKE_CASE (`STATUS_ACTIVE`)
- Files: snake_case (`create_user_accounts.sql`)
- Schemas: snake_case (`public`, `app_data`)

### GDScript

- Variables: snake_case (`var user_name`)
- Functions: snake_case (`func get_user()`)
- Classes: PascalCase (`class_name UserAccount`)
- Constants: SCREAMING_SNAKE_CASE (`const MAX_RETRIES`)
- Files: snake_case (`user_account.gd`)
- Nodes / Scene names: PascalCase (`UserAccount`)

### Shell (Bash / Zsh)

- Variables (local): snake_case lowercase (`user_name="value"`)
- Environment variables: SCREAMING_SNAKE_CASE (`export MAX_RETRIES=3`)
- Functions: snake_case (`get_user()`)
- Constants: SCREAMING_SNAKE_CASE readonly (`readonly MAX_RETRIES=3`)
- Files / Scripts: snake_case or kebab-case (`get-user.sh`, `get_user.sh`)

### PowerShell

- Variables: PascalCase or camelCase (`$UserName`, `$userName`)
- Functions / Cmdlets: PascalCase Verb-Noun (`Get-User`)
- Classes: PascalCase (`class UserAccount`)
- Constants: PascalCase or SCREAMING_SNAKE_CASE (`$MaxRetries`)
- Files / Scripts: PascalCase Verb-Noun (`Get-User.ps1`)
- Modules: PascalCase (`MyApp.Services`)

**Last Updated:** February 2026
