# TODO List

- [ ] Rust endpoints

  - [ ] Votes endpoint to update voting selection
  - [ ] Verification endpoint that takes the email code

- [ ] Rust search endpoint

  - [ ] Able to take the same parameters as normal meilisearch, allowing for multiple uses such as fetch all, sort, filter, etc.
  - [ ] Frontend exposed: filters, sorting, limit, offset
  - [ ] Backend core: filters, sorting, limit (number of results), offset, attribute to highlight, highlight tag

- [ ] Monitoring stack

- [ ] Basic frontend as submodule

- [ ] Github actions to ssh in to change deployment

# Done

## 1/6/26

- [x] Reducing excessive workflows using conditional file changes

- [x] Reverse proxy as submodule

## 1/3/26

- [x] Docker build/push CI using caching for both backend and proxy

  - [x] Separating proxy and backend builds by repo CI for parralel execution

- [x] Remote custom image deployment

## 1/2/26

- [x] Proxy docker image working with backend

- [x] Rust port exposed

- [x] Fetching foods from a range

- [x] Maintaining capitals from original foods

- [x] Remote fetch of custom images to avoid build times

- [x] Verbose feature for additional information messages

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
