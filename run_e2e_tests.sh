#!/bin/bash
cover -delete tests/cover_db
PERL_COVERAGE=1 cargo test --manifest-path tests/rust-e2e/Cargo.toml
cover -report html_basic -outputfile coverage.html tests/cover_db
firefox /workspace/RPi-Monitor/tests/cover_db/coverage.html 
