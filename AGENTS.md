# AGENTS.md

OSRS clan-chat Discord bot: Rust workspace + Vue 3 dashboard. The RuneLite
"TrackScape Connector" plugin connects to the API over a websocket; in-game
broadcasts are parsed into jobs that post to Discord. State lives in MongoDB;
background jobs are celery-style tasks brokered over Redis.

## Workspace layout

- `trackscape-discord-api` — actix-web service (shuttle). REST under `/api/*`,
  RuneLite plugin websocket at `/api/chat/ws`, and it serves the built
  dashboard from `./trackscape-discord-api/ui/` (path relative to repo root).
- `trackscape-discord-bot` — serenity Discord gateway bot (shuttle). Slash
  commands live in `src/commands/`; a new command must be added to BOTH
  `get_commands()` and the `interaction_create` match in `src/main.rs`.
- `trackscape-discord-shared` — shared library used by everything: Mongo
  database layer (no migrations — collections are created on demand), DTOs,
  OSRS broadcast parsing (`osrs_broadcast_extractor.rs`), celery job
  definitions, external API clients. Most tests live here.
- `trackscape-discord-job-worker` / `trackscape-discord-cron-job-worker` —
  plain binaries (not shuttle): job worker consumes Redis queues `celery` and
  `cron_job_queue`; cron worker enqueues scheduled jobs (e.g. WOM sync).
- `trackscape-discord-ui` — Vue 3 + Vite + Tailwind/daisyUI dashboard, NOT a
  cargo workspace member. Talks to the API via `VITE_API_BASE_URL`
  (see its `.env`).

## Dev commands

Prereqs: Rust stable, Docker, `cargo-shuttle` v0.48.1, Node 20.

    docker compose up -d        # mongo :27017 + redis :6379 — must be running
    cp .env.save .env           # fill in DISCORD_TOKEN, DEV_GUILD_ID (numeric), ...
    cargo shuttle run           # from repo ROOT

- `cargo shuttle run` from the workspace root starts BOTH shuttle services:
  API on :8000, bot on :8001. Run it from the root — the API resolves its UI
  path relative to the root.
- Background jobs need the workers too (separate terminals):
  `cargo run --bin trackscape-discord-job-worker`,
  `cargo run --bin trackscape-discord-cron-job-worker`. All processes read the
  root `.env` via dotenv.
- Most features need api + bot + redis + a worker running together — the bot
  and api call each other, and Discord posting happens inside jobs.

## Verification (what CI runs)

- Rust: `cargo check` and `cargo test` at the workspace root.
- Nearly all tests are in shared: `cargo test -p trackscape-discord-shared`;
  single test e.g. `cargo test -p trackscape-discord-shared test_pet_extractor`.
  New broadcast types should get extractor test vectors like the existing ones.
- UI (in `trackscape-discord-ui/`, use npm — CI does; `bun.lockb` is
  gitignored): `npm run lint`, `npm run build` (includes type-check),
  `npm run dev`, `npm run test:unit`.

## Gotchas

- Serving the dashboard locally requires building the UI and placing it where
  actix expects (mirroring `dockerfiles/Shuttle.Dockerfile`): copy
  `trackscape-discord-ui/dist/` to `trackscape-discord-api/ui/`. The API still
  boots without it. That directory is NOT gitignored — don't commit build
  output.
- `toolchain.toml` is malformed (`stable=cargo bu2024-06-10`, committed).
  rustup ignores it; builds use your default stable toolchain. Don't try to
  install that "channel".
- Stale per-crate `Cargo.lock` files in `trackscape-discord-api/` and
  `trackscape-discord-bot/` are leftovers — the workspace root `Cargo.lock`
  governs dependency resolution.
- Slash commands register on startup. With `DEV_GUILD_ID` set they register to
  that guild instantly; unset, they register globally (slow to propagate on
  Discord's side).
- PR expectations (`.github/CONTRIBUTING.md`): run check/build/test first, and
  create/link an issue for bugs and features before the PR.

## Release (maintainers)

`just newRelease` builds the 3 production images (linux/amd64) and pushes to
Docker Hub (`fatfingers23/trackscape`, `-worker`, `-cron-worker`). Production
runs `production/production-docker-compose.yml`.
