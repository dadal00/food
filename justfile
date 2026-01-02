# Sets environment
set dotenv-filename := ".env"
# Specify shell
set shell := ["bash", "-uc"]



# General Utilities
[doc]
hex:
	openssl rand -hex 32

clean-dir dir:
    find {{dir}} -mindepth 1 ! -name '.gitignore' -exec rm -rf {} +

clean-reverse-proxy:
	just clean-dir deploy/reverse_proxy/log
	just clean-dir deploy/reverse_proxy/cache
	just clean-dir deploy/reverse_proxy/acme_registry

clean-reusable:
	just clean-dir deploy/redis/data
	just clean-reverse-proxy

clean:
	@echo "Cleaning mounted volumes..."
	just clean-dir deploy/meilisearch/data
	just clean-reusable



# Docker Utilities
[doc]
erase:
	yes | docker system prune -a --volumes

join:
	docker swarm init

leave:
	docker swarm leave --force

network:
	docker network create --driver overlay --attachable --opt encrypted app_network

# Clean has an argument now so remember it may take it as one, hence why we need all here
[doc]
init:
	yes | just erase clean join network secrets



# Secrets
[doc]
secret name value:
	echo "{{ value }}" | docker secret create {{ name }} -

secrets mode="default":
	if [ "{{mode}}" == "clear" ]; then \
		docker secret rm MEILI_MASTER_KEY; \
		docker secret rm MEILI_ADMIN_KEY; \
	else \
		just hex | xargs -I{} just secret MEILI_MASTER_KEY "{}"; \
		just grab-meili-key; \
		just hex | xargs -I{} just secret JWT_KEY "{}"; \
	fi
	
meili-key:
	docker exec -i $(docker ps -q -f name=meilisearch) sh -c \
	'curl -H "Authorization: Bearer $(cat /run/secrets/MEILI_MASTER_KEY)" http://localhost:7700/keys' \
	| jq -r '.results[] | select(.name=="Default Admin API Key") | .key'

# Variables defined mid recipe will not be availble unless defined in the SAME shell session
[doc]
grab-meili-key:
	just secret MEILI_ADMIN_KEY 'abc'
	just deploy services

	admin_key=`just meili-key` && \
	just kill && \
	docker secret rm MEILI_ADMIN_KEY && \
	just secret MEILI_ADMIN_KEY $admin_key



# Deployment 
# Debug deployment only finishes if containers sucessfully created
[doc]
build service="all":
	if [ "{{service}}" == "all" ]; then \
		docker compose -f docker.build.custom.yml -f docker.build.services.yml build; \
	elif [ "{{service}}" == "custom" ]; then \
		docker compose -f docker.build.custom.yml build; \
	elif [ "{{service}}" == "services" ]; then \
		docker compose -f docker.build.services.yml build; \
	else \
		docker compose -f docker.build.services.yml build {{service}}; \
	fi

deploy mode="default":
	just clean-reusable

	if [ "{{mode}}" == "remote" ]; then \
		export RUST_IMAGE := "ghcr.io/dadal00/app_rust:latest"; \
		just build services; \
	elif [ "{{mode}}" == "services" ]; then \
		just build services; \
	else \
		just build; \
	fi

	if [ "{{mode}}" == "debug" ]; then \
		docker stack deploy -c deploy/docker.services.yml -c deploy/docker.app.yml app --detach=false; \
	elif [ "{{mode}}" == "services" ]; then \
		docker stack deploy -c deploy/docker.services.yml app --detach=false; \
	else \
		docker stack deploy -c deploy/docker.services.yml -c deploy/docker.app.yml app --detach=true; \
	fi

kill:
	@echo "Removing stack..."
	docker stack rm app

	@echo "Checking for removal..."
	@while docker stack services app | grep -q "^app"; do \
		printf "."; \
		sleep 1; \
	done

# Rust Rpxy
# Assuming proxy repo is a sibling
# [doc]
# proxy:
# 	envsubst < rpxy.config.toml > config.toml
# 	export RUST_IMAGE := "ghcr.io/dadal00/app_rust:latest"; \

# 	cd ../rust-rpxy && \
# 	cargo build --release && \
# 	./target/release/rpxy --config ../food/config.toml
	


