#!/usr/bin/env bash
# git clean -Xf
find . -name "*.dot" -type f -delete
find ./crates/ -name "*.sv" -type f -delete
find ./tests/ -name "*.sv" -type f -delete
find ./tests/ -name "*.csv" -type f -delete
