#!/bin/bash
# Generate test1.json by running the test1 generator
cargo run --bin automated_test_clock -- --create-test1-json --test-length-steps 33
