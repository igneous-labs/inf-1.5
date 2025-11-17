#!/bin/bash

REPO_DIR="$(dirname "$0")/../../.."
DOCKER_COMPOSE_LOCAL_VALIDATOR="$REPO_DIR/docker-compose-local-validator.yml"

# Ensure cleanup happens on script exit
cleanup() {
    # Clean up solana-test-validator container
    echo "Cleaning up containers..."
    docker compose -f $DOCKER_COMPOSE_LOCAL_VALIDATOR down -v
}

trap cleanup EXIT

docker compose -f $DOCKER_COMPOSE_LOCAL_VALIDATOR up -d --wait

vitest run "$@"
