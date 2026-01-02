# TODO List

- [ ] Changing names of items to perhaps just be capital since no need to keep for meilisearch id

- [ ] Monitoring stack

- [ ] Rust communication with proxy (exposing ports)
- [ ] Rust endpoints
- [ ] Basic frontend

# Done

## 1/2/26

- [x] Meilisearch initialization fix, using id as unique identifier instead of string due to certain naming conventions like no spaces

## 12/23/25

- [x] Rust communication with other containers
- [x] Redis initialization
- [x] Process + protobuf redesign to include location and maps
- [x] Fixed getting admin key bug with Meilisearch, only deploying services as the main app needs the key
- [x] Removed envsubt as does not seem to be doing anything

## 12/16/25

- [x] Rpxy JWT cookie + Moka lookup
- [x] Rpxy handling slow and fast paths

## 12/04/25

- [x] Unit tests for process/sanitize

## 12/02/25

- [x] Rust workspace for process and server
- [x] Rust shear CI
- [x] Proto file
- [x] Food processing
- [x] Repo cron job for foods

## 12/01/25

- [x] Docker image build, push, and prune CI
- [x] Just shell fix, specify bash

## 11/27/25

- [x] Docker secret setup
- [x] Just (script automation)
- [x] Environment variable setup

## 11/26/25

- [x] Basic Meilisearch, Redis, Rust Docker Swarm
