import { defineRailway, github, postgres, project, redis, service, volume } from "railway/iac";

export default defineRailway(() => {
  const trackscapeDiscordBot = github("Aid55/trackscape-discord-bot");

  const Postgres = postgres("Postgres", { region: "us-east4-eqdc4a" });
  Postgres.networking = { privateNetworkEndpoint: "postgres" };
  const Redis = redis("Redis", { region: "us-east4-eqdc4a" });
  Redis.deploy = { startCommand: "/bin/sh -c \"rm -rf $RAILWAY_VOLUME_MOUNT_PATH/lost+found/ && exec docker-entrypoint.sh redis-server --requirepass $REDIS_PASSWORD --save 60 1 --dir $RAILWAY_VOLUME_MOUNT_PATH\"" };
  Redis.networking = { privateNetworkEndpoint: "redis" };
  const redisVolume = volume("redis-volume", { alerts: { usage: { "100": {}, "80": {}, "95": {} } }, allowOnlineResize: true, region: "us-east4-eqdc4a", sizeMB: 50000 });
  const postgresVolume = volume("postgres-volume", { alerts: { usage: { "100": {}, "80": {}, "95": {} } }, allowOnlineResize: true, region: "us-east4-eqdc4a", sizeMB: 50000 });
  const cronWorker = service("cron-worker", {
    source: trackscapeDiscordBot,
    replicas: { "us-east4-eqdc4a": 1 },
  });
  const worker = service("worker", {
    source: trackscapeDiscordBot,
    replicas: { "us-east4-eqdc4a": 1 },
  });
  const api = service("api", {
    source: trackscapeDiscordBot,
    replicas: { "us-east4-eqdc4a": 1 },
  });
  const bot = service("bot", {
    source: trackscapeDiscordBot,
    replicas: { "us-east4-eqdc4a": 1 },
  });

  return project("trackscape", {
    variables: { managed: false },
    resources: [cronWorker, worker, api, bot, Postgres, Redis, redisVolume, postgresVolume],
  });
});
