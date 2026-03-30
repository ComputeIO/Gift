#!/bin/bash
# Test script for lead/worker provider functionality

# Set up test environment variables
export LEAF_PROVIDER="openai"
export LEAF_MODEL="gpt-4o-mini"
export OPENAI_API_KEY="test-key"

# Test 1: Default behavior (no lead/worker)
echo "Test 1: Default behavior (no lead/worker)"
unset LEAF_LEAD_MODEL
unset LEAF_WORKER_MODEL
unset LEAF_LEAD_TURNS

# Test 2: Lead/worker with same provider
echo -e "\nTest 2: Lead/worker with same provider"
export LEAF_LEAD_MODEL="gpt-4o"
export LEAF_WORKER_MODEL="gpt-4o-mini"
export LEAF_LEAD_TURNS="3"

# Test 3: Lead/worker with default worker (uses main model)
echo -e "\nTest 3: Lead/worker with default worker"
export LEAF_LEAD_MODEL="gpt-4o"
unset LEAF_WORKER_MODEL
export LEAF_LEAD_TURNS="5"

echo -e "\nConfiguration examples:"
echo "- Default: Uses LEAF_MODEL for all turns"
echo "- Lead/Worker: Set LEAF_LEAD_MODEL to use a different model for initial turns"
echo "- LEAF_LEAD_TURNS: Number of turns to use lead model (default: 5)"
echo "- LEAF_WORKER_MODEL: Model to use after lead turns (default: LEAF_MODEL)"