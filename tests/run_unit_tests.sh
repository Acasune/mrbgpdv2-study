#!/bin/bash
docker compose -f ./tests/docker-compose.yml build --no-cache
docker compose -f ./tests/docker-compose.yml up -d
HOST_2_LOOPBACK_IP=10.100.220.3
docker compose -f ./tests/docker-compose.yml exec \
  -T host2 cargo test -- --test-threads=1 --nocapture
