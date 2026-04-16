---
name: skill-refiner
description: Analyze project and filter skills using permission.skill in opencode.json
---

# Skill Refiner

Analyze project and configure skills using permission patterns in opencode.json.

## How Skills Discovery Works

OpenCode automatically discovers skills from these directories:
- `.opencode/skills/<name>/SKILL.md`
- `.agents/skills/<name>/SKILL.md`  
- `.claude/skills/<name>/SKILL.md`
- `~/.config/opencode/skills/<name>/SKILL.md`

All discovered skills appear in `<available_skills>` list in the skill tool description.

## Why Permissions Matter

The agent sees ALL discovered skills (name + description) by default. Using `permission.skill` with:
- `allow` - skill is available to agent
- `deny` - skill is HIDDEN from agent (not in available list)
- `ask` - user must approve before loading

This affects token usage - hidden skills won't clutter the context.

## Workflow

### Step 1: Analyze Project
Scan root config files to detect technologies:

| File | Language/Framework |
|------|-------------------|
| package.json | JavaScript/TypeScript |
| Cargo.toml | Rust |
| go.mod | Go |
| pyproject.toml | Python |
| Gemfile | Ruby |
| pom.xml | Java |
| pubspec.yaml | Dart/Flutter |

Detect from contents:
- Dependencies (React, Next.js, Express, FastAPI, Django, Gin, etc.)
- Frameworks and databases
- Build tools and bundlers

### Step 2: Map Technologies to Skills

| If Project Uses | Keep These Skills |
|----------------|------------------|
| React / Next.js | react-patterns, nextjs-app-router-patterns, frontend-developer, typescript-pro |
| TypeScript | typescript-pro |
| Tailwind CSS | tailwind-patterns |
| Node.js / Express | nodejs-backend-patterns, javascript-pro |
| Go / Gin / Echo | golang-backend-patterns |
| Python / FastAPI | fastapi-pro, python-pro, python-patterns, pydantic-models-py |
| Python / Django | python-pro, python-patterns |
| Rust | rust-pro, rust-async-patterns, systems-programming-rust-project |
| PostgreSQL | postgresql |
| Prisma | prisma-expert |
| Neon (serverless) | neon-postgres |
| WordPress | wordpress |
| Flutter | flutter-expert |
| React Native | react-native-architecture |
| iOS | ios-developer |
| Mobile | mobile-developer |
| Docker | docker-expert |
| Vercel | vercel-deployment |
| CI/CD | github-actions-templates, cicd-automation-workflow-automate |
| Testing | playwright-skill, e2e-testing-patterns, test-driven-development |
| Security | security-audit, security-scanning-security-sast |

### Step 3: ALWAYS Keep Core Skills
These are mandatory for code quality:
- `lint-and-validate` - mandatory for code quality checks
- `code-reviewer` - for peer reviews
- `clean-code` - for best practices

### Step 4: Update opencode.json

CORRECT schema - use permission patterns:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "instructions": [
    "Project uses [detected technologies].",
    "Run [build command] to execute.",
    "Use ONLY these skills: [list], ignore all others."
  ],
  "mcp": {
    "filesystem": { "enabled": true },
    "searxng": { "enabled": true },
    "context7": { "enabled": true },
    "chrome-devtools": { "enabled": true },
    "playwright_mcp": { "enabled": false },
    "sequential-thinking": { "enabled": true }
  },
  "permission": {
    "skill": {
      "detected-skill-1": "allow",
      "detected-skill-2": "allow",
      "detected-skill-N": "allow",
      "lint-and-validate": "allow",
      "code-reviewer": "allow",
      "clean-code": "allow",
      "*": "deny"
    }
  }
}
```

Key points:
1. Use `$schema` for validation
2. Use `"instructions"` to tell agent which skills to use
3. Use `"permission.skill"` with specific allow + `"*": "deny"` pattern
4. Enable MCPs based on detected tech, disable unused
5. Core skills always `"allow"`

### Step 5: Update BOTH Files (REQUIRED)

Update BOTH files to ensure consistency - NO CONTRADICTIONS between config and instructions:

**a) .opencode/opencode.json**
Write configured JSON with permission patterns (see Step 4 above).

**b) AGENTS.md** 
Create/update to match opencode.json EXACTLY - skills must be the same:

```markdown
# AGENTS.md

## Project Type
[description with detected tech stack]

## Run Commands
- `[build command]` - build and run
- `[test command]` - run tests

## Key Files
- [list main files]

## Notes
[Any project-specific notes]

## Skills - USE ONLY THESE
When asked to use skills, only invoke:

| Category | Skills |
|----------|--------|
| Core (always) | lint-and-validate, code-reviewer, clean-code |
| [Tech 1] | skill-a, skill-b |

**IGNORE all other skills** - they are not applicable to this project.

## OpenCode Config Location
`.opencode/opencode.json` (project-local)
```

CRITICAL: Skills in AGENTS.md MUST match `permission.skill` in opencode.json exactly.

## MCP Recommendations by Tech

| Tool | Use When |
|------|----------|
| filesystem | Always - file operations |
| searxng | Always - web research |
| context7 | Always - documentation lookup |
| sequential-thinking | Complex planning |
| chrome-devtools | Frontend/E2E testing |
| playwright_mcp | E2E browser automation |
| fetch | HTTP requests |

Disable unused MCPs to reduce context.

## Output Summary

After completion, summarize:

- **Detected technologies**: [list]
- **Skills allowed** (permission allow): [list]
- **Skills denied** (permission deny): [list]
- **MCPs enabled**: [list]
- **MCPs disabled**: [list]
- **Files updated**: [list]

## Important

1. **NEVER delete** files from `.agents/skills/` or `.opencode/skills/` folder - only update config
2. **ALWAYS include** core skills (lint-and-validate, code-reviewer, clean-code) with `"allow"`
3. **Use wildcard pattern** `"*": "deny"` to hide all non-specified skills
4. **Verify** with schema: https://opencode.ai/config.json
5. **If unclear** about technology - ask user before proceeding
