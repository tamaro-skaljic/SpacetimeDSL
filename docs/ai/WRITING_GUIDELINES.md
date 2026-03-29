# General-Purpose Guidelines for Writing Instructions for AI Coding Agents

Author precise instructions before you code. This document distills vendor guidance, academic studies, and practitioner know-how into a reusable playbook that applies to any AI coding assistant or workflow.

## Research basis

- **Primary documentation**: Microsoft Learn’s Copilot prompt foundations, Anthropic’s Claude prompt engineering series, Cursor Community Rules for AI, Windsurf’s prompt engineering guide, and Cline’s prompting handbook provide the core patterns summarized below. [^1] [^2] [^3] [^4] [^5] [^6]
- **Supplementary studies**: Google Scholar scans (Nov 17 2025) on prompt engineering across Copilot, Claude, Cursor, Windsurf, and Cline show that rigorous prompts improve security posture, evaluation fidelity, collaboration patterns, and developer confidence. [^7] [^8] [^9] [^10] [^11]

## Core success pillars (4 S + validation)

For every instruction package, confirm the following pillars:

### Single-tasked

**Why it matters:** Focused requests reduce ambiguity and boost completion quality, keeping the agent’s attention on one outcome at a time. [^1]

**Quick self-check:** If the prompt can be split into multiple deliverables, rewrite it as separate asks before proceeding.

### Specific

**Why it matters:** Explicit verbs, metrics, and artifacts keep responses on-rails and aligned with stakeholder expectations. [^2] [^3]

**Quick self-check:** Underline each verb; whenever one feels vague, replace it with a concrete action and named artifact.

### Short

**Why it matters:** Concise wording lowers latency and leaves more space for the context payload that agents depend on. [^1] [^5]

**Quick self-check:** Read the request aloud—if it takes longer than 20 seconds or requires a breath mid-sentence, tighten it.

### Surrounded with context

**Why it matters:** Agents need environment, audience, dependency, and workflow clues to act correctly and avoid rework. [^1] [^4] [^5]

**Quick self-check:** Ask whether a new teammate could execute the task from the prompt alone; if not, add the missing context.

### Validated upfront

**Why it matters:** Naming tests and reviewers early prevents insecure outputs and clarifies what “done” means before any code is written. [^7] [^9] [^11]

**Quick self-check:** Record the exact test, metric, or reviewer in the prompt header so validation can happen immediately after delivery.

## Instruction design lifecycle

1. **Assess** — Capture the user outcome, acceptance criteria, risks, and stakeholders before engaging an agent.
2. **Draft** — Write a plain-language brief that balances goal, scope, context, and constraints. Sanity-check it with a teammate when stakes are high. [^3]
3. **Encode** — Translate the brief into the right surface (chat message, rule file, template) while preserving structure, references, and success tests. [^1] [^4] [^5] [^6]
4. **Orchestrate & iterate** — Run the prompt, inspect outputs, and ask for revisions via critique loops instead of rewriting code manually. [^1] [^2]
5. **Validate** — Execute the named tests, lint checks, or reviewers. Capture outcomes directly in the thread or log. [^7] [^9]
6. **Maintain** — Version-control instruction docs, retire stale guidance, and document deltas so future prompts inherit accurate guardrails. [^4] [^5] [^6]

## Prompt blueprint

Use the following skeleton to encode any instruction set:

```txt
Goal:
Success test:
Context:
Constraints & roles:
Process / iteration plan:
Validation & hand-off:
```

### Goal

State the single desired outcome in one sentence so the agent understands the mission statement. Example: “Implement a caching layer for search results.”

### Success test

Declare exactly how acceptance will be measured, including commands or KPIs when possible. Example: “Pass `npm test search-cache` with latency under 50 ms.”

### Context

Provide files, environments, dependencies, and audiences so the agent knows where the work lives. Example: “Service: `api/search`, depends on the Redis cluster.”

### Constraints & roles

Encode compliance requirements, security posture, reviewer expectations, and the persona the agent should adopt. Example: “Act as an OWASP reviewer while following company lint rules.”

### Process / iteration plan

Describe the reasoning style, critique loops, or delivery order to guide how the agent should work. Example: “List assumptions, share an outline, await approval, then implement.”

### Validation & hand-off

Specify the tests to run, artifacts to attach, and summary details required for completion. Example: “Run `pytest -k cache`, then post a diff summary plus any follow-up tasks.”

## Universal prompt checklist

- **Goal & success metric stated?** (tests, lint targets, KPIs) [^2] [^7]
- **Context packaged?** (files, tech stack, stakeholders) [^1] [^3] [^5]
- **Constraints encoded?** (security, compliance, performance budgets) [^1] [^5] [^6]
- **Examples or references included?** (snippets, diffs, style guides) [^1] [^2]
- **Iteration plan documented?** (reasoning steps, critique cadence, approval gates) [^1] [^2] [^6]
- **Memory/scope management noted?** (reset instructions, ignored paths, context gating) [^4] [^5] [^6]
- **Validation + reviewer named?** (tests, QA owner, acceptance timestamp) [^7] [^9]

## Cross-cutting tactics

1. **Define success & evaluation hooks early** — Agents deliver better code when tests and reviewers are baked into the ask. [^2]
2. **Package purposeful context** — Summarize audience, workflow stage, files, APIs, and configs so the agent sees the same landscape you do. [^1] [^3] [^5]
3. **Show, then tell** — Pair requirements with zero/one/few-shot exemplars or references to align tone and structure. [^1] [^2]
4. **Assign roles & constraints** — Explicitly state security posture, performance budgets, accessibility needs, or reviewer personas to steer reasoning. [^1] [^5] [^6]
5. **Plan iteration loops** — Ask for assumptions, outlines, or diffs before code lands; critique outputs and request revisions instead of starting over. [^1] [^2]
6. **Control scope & memory** — Reset chats, gate large repos with globbed context, and exclude secrets/noise via ignore lists or summaries. [^1] [^4] [^5] [^6]
7. **Document guardrails** — Store recurring instructions in versioned rule files, templates, or runbooks so teams inherit the same expectations. [^4] [^5] [^6]

## Validation planning

- **Tests & tooling**: Identify the exact commands (unit tests, linters, smoke scripts) the agent must run or prepare. Provide command snippets when possible.
- **Metrics & thresholds**: Define latency budgets, coverage targets, or error budgets to flag regressions immediately.
- **Review cadence**: Name human reviewers or automated gates (CI jobs, security scanners) that must sign off.
- **Evidence capture**: Request logs, diff summaries, or checklists so validation results are recorded alongside the prompt.

## Research-backed signals

- **Security**: Prompt hardening plus explicit requirements reduce insecure infrastructure-as-code and OSS patterns. [^7] [^9] [^10]
- **Evaluation**: Structured templates improve reproducibility and accuracy in empirical studies. [^8] [^9]
- **Collaboration**: Clear prompts reshape team workflows, balancing autonomy and oversight across IDE agents. [^7] [^10] [^11]
- **Education**: Prompt scaffolding accelerates onboarding for students and new teammates exploring large repos. [^7] [^10]
- **Ecosystems**: Comparative analyses highlight rule files, prompt integrity, and context gating as key differentiators across tools. [^9] [^11]

## References

[^1]: Microsoft Learn – *Prompt engineering foundations and best practices* (GitHub Copilot). <https://learn.microsoft.com/en-us/training/modules/introduction-prompt-engineering-with-github-copilot/2-prompt-engineering-foundations-best-practices>
[^2]: Claude Docs – *Prompt engineering overview*. <https://docs.claude.com/claude/docs/constructing-a-prompt>
[^3]: Claude Docs – *Be clear, direct, and detailed*. <https://docs.claude.com/en/docs/build-with-claude/prompt-engineering/be-clear-and-direct>
[^4]: Cursor Community – *Rules for AI*. <https://cursorcommunity.com/docs/rules-for-ai>
[^5]: Windsurf Docs – *Prompt Engineering*. <https://docs.windsurf.com/best-practices/prompt-engineering>
[^6]: Cline Docs – *Prompt Engineering Guide*. <https://docs.cline.bot/prompting/prompt-engineering-guide>
[^7]: Google Scholar search – “GitHub Copilot prompt guidelines”. <https://scholar.google.com/scholar?q=GitHub+Copilot+prompt+guidelines>
[^8]: Google Scholar search – “Claude prompt guidelines”. <https://scholar.google.com/scholar?q=Claude+prompt+guidelines>
[^9]: Google Scholar search – “Cursor AI prompt engineering”. <https://scholar.google.com/scholar?q=Cursor+AI+prompt+engineering>
[^10]: Google Scholar search – “Windsurf AI prompt”. <https://scholar.google.com/scholar?q=Windsurf+AI+prompt>
[^11]: Google Scholar search – “Cline AI coding agent”. <https://scholar.google.com/scholar?q=Cline+AI+coding+agent>
