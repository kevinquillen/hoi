#!/usr/bin/env bash
set -e

echo "Building hoi..."
cargo build

echo "Running unit tests..."
cargo test

echo "Running integration tests..."
cargo test --test integration_test

echo "All tests passed!"