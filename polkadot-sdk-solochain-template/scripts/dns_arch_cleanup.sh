#!/bin/bash

# Remove all docker containers belonging to the substrate-template image
docker ps -a --format "{{.ID}} {{.Image}}" | grep "substrate-template" | awk '{print $1}' | xargs docker rm -f

# Remove trailing images and all unused networks
docker system prune -af

# Remove folder containing all spec files
rm -rf all_specs *.txt *.json *.sh