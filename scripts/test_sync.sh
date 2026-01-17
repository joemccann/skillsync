#!/bin/bash
# One-time sync test script

echo "Testing SkillSync transformations..."
echo

# Create test directories
mkdir -p ~/.gemini/skills
mkdir -p ~/.gemini/commands

# Run the binary once (it will do initial sync then we'll kill it)
timeout 3s ./target/release/skillsync || true

echo
echo "=== Checking results ==="
echo

echo "1. Claude-style sync to ~/.gemini/skills/:"
ls -la ~/.gemini/skills/ | head -10

echo
echo "2. Gemini TOML sync to ~/.gemini/commands/:"
ls -la ~/.gemini/commands/*.toml 2>/dev/null || echo "  No TOML files found"

echo
echo "3. Sample TOML content (ui-skills.toml):"
if [ -f ~/.gemini/commands/ui-skills.toml ]; then
    head -5 ~/.gemini/commands/ui-skills.toml
else
    echo "  File not found"
fi

echo
echo "4. Antigravity skills sync:"
ls -la ~/.gemini/antigravity/skills/ | head -10
