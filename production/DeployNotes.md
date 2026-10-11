# Docker builds and push commands
 `docker build --platform linux/amd64 -t {image name} -f dockerfiles/{name of docker file} .`

 `docker tag {image name}:latest fatfingers23/{repo name}:latest`

 `docker push fatfingers23/{repo name}:latest`

 # Repo Names with tags
 - fatfingers23/trackscape-api:latest (actix-web API + serves the dashboard UI)
 - fatfingers23/trackscape-bot:latest (serenity Discord gateway bot)
 - fatfingers23/trackscape-worker:latest (Job worker )
 - fatfingers23/trackscape-cron-worker:latest (Binary to fire off jobs on cron schedule)

 # Deploy order
 The `trackscape-api` and `trackscape-bot` images replace the old combined `fatfingers23/trackscape` shuttle image.
 Build and push them BEFORE rolling out the updated `production-docker-compose.yml`, otherwise the `api`/`bot`
 services in the new compose file will not have an image to pull.
