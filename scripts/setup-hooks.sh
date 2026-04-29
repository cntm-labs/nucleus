#!/bin/sh
# Activate the tracked git hooks under .githooks/.
#
# Run once per clone. Re-running is safe and idempotent.
set -e

ROOT="$(git rev-parse --show-toplevel)"

git -C "$ROOT" config core.hooksPath .githooks
chmod +x "$ROOT"/.githooks/*

echo "✅ Git hooks activated (core.hooksPath = .githooks)"
echo "   Active hooks: $(ls "$ROOT"/.githooks | tr '\n' ' ')"
echo ""
echo "Tip: set SONAR_TOKEN and install sonar-scanner to enable the SonarQube step."
