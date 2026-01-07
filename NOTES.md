# Redis

redis-cli

# Meilisearch

Get first 10 items in the index

curl "http://127.0.0.1:7700/indexes/foods/documents?offset=0&limit=10" -H "Authorization: Bearer $(cat /run/secrets/MEILI_MASTER_KEY)"

Get item based on their id

curl "http://127.0.0.1:7700/indexes/foods/documents/7" -H "Authorization: Bearer $(cat /run/secrets/MEILI_MASTER_KEY)"
