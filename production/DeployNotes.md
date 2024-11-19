# Docker builds and push commands
 `docker build --platform linux/amd64 -t {image name} -f dockerfiles/{name of docker file} .`

 `docker tag {image name}:latest fatfingers23/{repo name}:latest`

 `docker push fatfingers23/{repo name}:latest`

 # Repo Names with tags
 - fatfingers23/trackscape:latest (Main shuttle application)
 - fatfingers23/trackscape-worker:latest (Job worker )
 - fatfingers23/trackscape-cron-worker:latest (Binary to fire off jobs on cron schedule)
