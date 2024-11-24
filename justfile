#Just file to build and release the docker image for production

#Image names
shuttle := "trackscape"
worker := "trackscape-worker"
cronWorker := "trackscape-cron-worker"
dockerRepo := "fatfingers23"
tag := "latest"

#Build and release
newRelease:
    #Shuttle
    docker build --platform linux/amd64 -t {{shuttle}} -f dockerfiles/Shuttle.Dockerfile .
    docker tag {{shuttle}}:{{tag}} {{dockerRepo}}/{{shuttle}}:{{tag}}
    docker push {{dockerRepo}}/{{shuttle}}:{{tag}}

    #Worker
    docker build --platform linux/amd64 -t {{worker}} -f dockerfiles/Worker.Dockerfile .
    docker tag {{worker}}:{{tag}} {{dockerRepo}}/{{worker}}:{{tag}}
    docker push {{dockerRepo}}/{{worker}}:{{tag}}

    #Cron Worker
    docker build --platform linux/amd64 -t {{cronWorker}} -f dockerfiles/CronWorker.Dockerfile .
    docker tag {{cronWorker}}:{{tag}} {{dockerRepo}}/{{cronWorker}}:{{tag}}
    docker push {{dockerRepo}}/{{cronWorker}}:{{tag}}