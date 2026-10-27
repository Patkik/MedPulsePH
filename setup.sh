#!/usr/bin/env bash
set -e

mkdir -p .claude/skills/nfc-sun-crypto
mkdir -p .claude/skills/split-vault-typestate
mkdir -p .claude/skills/fhir-medical-api
mkdir -p .claude/skills/firebase-deploy-scale
mkdir -p .claude/skills/anti-slop-logo

echo "✅ Created .claude/skills/ directories for nfc-sun-crypto, split-vault-typestate, fhir-medical-api, firebase-deploy-scale, and anti-slop-logo."
echo "📌 Next: Paste the SKILL.md files, AGENTS.md, and .mcp.json above into their respective paths."
ln -sf AGENTS.md CLAUDE.md
echo "🔗 Linked CLAUDE.md -> AGENTS.md for cross-agent compatibility."
