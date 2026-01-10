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



# Rust Rpxy
# Assuming proxy repo is a sibling
[doc]
proxy:
	git submodule update --init --remote

proxy-config:
	cd ./deploy/reverse_proxy && envsubst < config > config.toml

proxy-submodules:
	cd ./submodules/rust-rpxy && \
	git submodule update --init

proxy-init:
    just proxy
    just proxy-config
    just proxy-submodules

    if [ ! -d deploy/reverse_proxy/log ]; then \
    	mkdir deploy/reverse_proxy/log; \
    fi
    if [ ! -d deploy/reverse_proxy/cache ]; then \
        mkdir deploy/reverse_proxy/cache; \
    fi
    if [ ! -d deploy/reverse_proxy/acme_registry ]; then \
        mkdir deploy/reverse_proxy/acme_registry; \
    fi





# Deployment 
# Debug deployment only finishes if containers sucessfully created
[doc]
build service="all":
	if [ "{{service}}" == "services" ]; then \
	    docker buildx bake -f docker.build.yml meilisearch redis grafana; \
	else \
		docker buildx bake -f docker.build.yml; \
	fi

deploy target="all":
	just proxy-init
	if [ "{{target}}" != "production" ]; then \
		just clean-reusable; \
	fi

	if [ "{{target}}" == "services" ] || [ "{{target}}" == "remote" ] || [ "{{target}}" == "production" ]; then \
		just build services; \
	else \
		just build; \
	fi

	if [ "{{target}}" == "debug" ]; then \
		docker stack deploy -c deploy/docker.services.yml -c deploy/docker.app.yml app --detach=false; \
	elif [ "{{target}}" == "services" ]; then \
		docker stack deploy -c deploy/docker.services.yml app --detach=false; \
	elif [ "{{target}}" == "remote" ] || [ "{{target}}" == "production" ]; then \
		PROXY_IMAGE="ghcr.io/dadal00/reverse_proxy:latest" \
		RUST_IMAGE="ghcr.io/dadal00/app_rust:latest" \
		docker stack deploy -c deploy/docker.services.yml -c deploy/docker.app.yml app --detach=false; \
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



# Local Testing 
[doc]
payload:
	cd tester && cargo payload

jwt:
	cd tester && cargo jwt


