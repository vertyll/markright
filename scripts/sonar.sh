#!/usr/bin/env bash
# Runs the checks, measures the test coverage, then analyses the repository with the SonarQube scanner, which runs
# Clippy itself. The scanner reads sonar-project.properties and lcov.info, and finds the server in SONAR_HOST_URL and
# the token in SONAR_TOKEN.
#
# Usage: SONAR_HOST_URL=<server> SONAR_TOKEN=<token> bash scripts/sonar.sh
# Needs: what scripts/check.sh needs, cargo-llvm-cov for the coverage, and Node; the scanner is fetched at the version
# pinned here.
set -euo pipefail
cd "$(dirname "$0")/.."

bash scripts/check.sh
cargo llvm-cov --workspace --locked --lcov --output-path lcov.info
npx --yes @sonar/scan@5.0.1
