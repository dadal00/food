# Sets environment
set dotenv-filename := ".env"
# Specify shell
set shell := ["bash", "-uc"]



# General Utilities
[doc]
hex:
	openssl rand -hex 32



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

init:
	yes | just erase join network secrets



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
	fi
	
meili-key:
	docker exec -i $(docker ps -q -f name=meilisearch) sh -c \
	'curl -H "Authorization: Bearer $(cat /run/secrets/MEILI_MASTER_KEY)" http://localhost:7700/keys' \
	| jq -r '.results[] | select(.name=="Default Admin API Key") | .key'

# Variables defined mid recipe will not be availble unless defined in the SAME shell session
[doc]
grab-meili-key:
	just secret MEILI_ADMIN_KEY 'abc'
	just deploy debug

	admin_key=`just meili-key` && \
	just kill && \
	docker secret rm MEILI_ADMIN_KEY && \
	just secret MEILI_ADMIN_KEY $admin_key



# Deployment 
# Debug deployment only finishes if containers sucessfully created
[doc]
build service="all":
	if [ "{{service}}" == "all" ]; then \
		envsubst < deploy/docker.build.yml | docker compose -f deploy/docker.build.yml build; \
	elif [ "{{service}}" == "custom" ]; then \
		envsubst < deploy/docker.build.yml | docker compose -f deploy/docker.build.yml build ${CUSTOM_IMAGES}; \
	else \
		envsubst < deploy/docker.build.yml | docker compose -f deploy/docker.build.yml build {{service}}; \
	fi

deploy mode="default":	
	just build

	if [ "{{mode}}" == "debug" ]; then \
		envsubst < deploy/docker.swarm.yml | docker stack deploy -c deploy/docker.swarm.yml app --detach=false; \
	else \
	 	envsubst < deploy/docker.swarm.yml | docker stack deploy -c deploy/docker.swarm.yml app --detach=true; \
	fi

kill:
	@echo "Removing stack..."
	docker stack rm app

	@echo "Checking for removal..."
	@while docker stack services app | grep -q "^app"; do \
		printf "."; \
		sleep 1; \
	done

clean-dir dir:
    find {{dir}} -mindepth 1 ! -name '.gitignore' -exec rm -rf {} +

clean:
    @echo "Cleaning mounted volumes..."
    just clean-dir deploy/meilisearch/data
    just clean-dir deploy/redis/data


