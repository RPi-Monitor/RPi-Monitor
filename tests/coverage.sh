#!/bin/bash
# Run the test suite under Devel::Cover and build the HTML report.
set -euo pipefail

cd "$(dirname "$0")"
cover -delete
# The report is built even when tests fail; the exit status stays pytest's.
status=0
PERL_COVERAGE=1 pytest -v --tb=short . || status=$?
cover -report html
echo "Report: file://$(pwd)/cover_db/coverage.html"
exit "${status}"
