#Just file to build and release the docker image for production

#Image names
api := "trackscape-api"
bot := "trackscape-bot"
worker := "trackscape-worker"
cronWorker := "trackscape-cron-worker"
dockerRepo := "fatfingers23"
tag := "latest"

#Build and release
newRelease:
    #API
    docker build --platform linux/amd64 -t {{api}} -f dockerfiles/Api.Dockerfile .
    docker tag {{api}}:{{tag}} {{dockerRepo}}/{{api}}:{{tag}}
    docker push {{dockerRepo}}/{{api}}:{{tag}}

    #Bot
    docker build --platform linux/amd64 -t {{bot}} -f dockerfiles/Bot.Dockerfile .
    docker tag {{bot}}:{{tag}} {{dockerRepo}}/{{bot}}:{{tag}}
    docker push {{dockerRepo}}/{{bot}}:{{tag}}

    #Worker
    docker build --platform linux/amd64 -t {{worker}} -f dockerfiles/Worker.Dockerfile .
    docker tag {{worker}}:{{tag}} {{dockerRepo}}/{{worker}}:{{tag}}
    docker push {{dockerRepo}}/{{worker}}:{{tag}}

    #Cron Worker
    docker build --platform linux/amd64 -t {{cronWorker}} -f dockerfiles/CronWorker.Dockerfile .
    docker tag {{cronWorker}}:{{tag}} {{dockerRepo}}/{{cronWorker}}:{{tag}}
    docker push {{dockerRepo}}/{{cronWorker}}:{{tag}}
