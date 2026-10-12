# AGENTS.md

OSRS clan-chat Discord bot: Rust workspace + Vue 3 dashboard. The RuneLite
"TrackScape Connector" plugin connects to the API over a websocket; in-game
broadcasts are parsed into jobs that post to Discord. State lives in PostgreSQL (sqlx, compile-time checked queries with committed `.sqlx` offline metadata; schema migrations live in `trackscape-discord-shared/migrations/` and run automatically at API startup); background jobs are celery-style tasks brokered over Redis.

## Workspace layout

- `trackscape-discord-api` — actix-web service. REST under `/api/*`,
  RuneLite plugin websocket at `/api/chat/ws`, and it serves the built
  dashboard from `./trackscape-discord-api/ui/` (path relative to repo root).
- `trackscape-discord-bot` — serenity Discord gateway bot. Slash
  commands live in `src/commands/`; a new command must be added to BOTH
  `get_commands()` and the `interaction_create` match in `src/main.rs`.
- `trackscape-discord-shared` — shared library used by everything: Postgres/sqlx database layer (migrations in `migrations/`, embedded and exported as `database::MIGRATOR`), DTOs, OSRS broadcast parsing (`osrs_broadcast_extractor.rs`), celery job definitions, external API clients. Most tests live here. The `legacy-mongo` cargo feature exposes the old Mongo document shapes for the one-off migrator only.
- `wom-rs` — vendored Wise Old Man API client (formerly a git dependency on
  fatfingers23/wom_rs @ 56ba91e). Maintained in-tree; its `Country` enum is
  patched with WOM's GB constituent-country codes plus an `Unknown` fallback
  so new WOM country codes can't break group deserialization.
- `trackscape-discord-job-worker` / `trackscape-discord-cron-job-worker` —
  plain binaries: job worker consumes Redis queues `celery` and
  `cron_job_queue`; cron worker enqueues scheduled jobs (e.g. WOM sync).
- `trackscape-discord-ui` — Vue 3 + Vite + Tailwind/daisyUI dashboard, NOT a
  cargo workspace member. Talks to the API via `VITE_API_BASE_URL`
  (see its `.env`).

## Dev commands

Prereqs: Rust stable, Docker, Node 20.

    docker compose up -d        # postgres :5432 + redis :6379 — must be running
    cp .env.save .env           # fill in DISCORD_TOKEN, DEV_GUILD_ID (numeric), ...

- The API and bot are separate binaries. Run each from the repo ROOT in its own
  terminal: `cargo run --bin trackscape-discord-api` (serves the dashboard on
  :8000 by default; override with `HOST`/`PORT` — the API resolves its UI path
  relative to the root) and `cargo run --bin trackscape-discord-bot` (serenity
  gateway bot, no inbound port).
- Background jobs need the workers too (separate terminals):
  `cargo run --bin trackscape-discord-job-worker`,
  `cargo run --bin trackscape-discord-cron-job-worker`. All processes read the
  root `.env` via dotenv. Log verbosity is controlled with `RUST_LOG`
  (defaults to `info`); the API uses `env_logger`, the bot `tracing-subscriber`.
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
  actix expects (mirroring `dockerfiles/Api.Dockerfile`): copy
  `trackscape-discord-ui/dist/` to `trackscape-discord-api/ui/`. The API still
  boots without it. That directory is NOT gitignored — don't commit build
  output.
- `rust-toolchain.toml` pins Rust 1.99.0; rustup auto-installs it when building in the
  repo. When bumping Rust, keep it in sync with CI (`dtolnay/rust-toolchain@…`) and the
  Docker builder images (`rust:<version>-bookworm`).
- Slash commands register on startup. With `DEV_GUILD_ID` set they register to
  that guild instantly; unset, they register globally (slow to propagate on
  Discord's side).
- PR expectations (`.github/CONTRIBUTING.md`): run check/build/test first, and
  create/link an issue for bugs and features before the PR.
- sqlx queries are compile-time checked. Building with a live DB requires `DATABASE_URL` and the schema applied (`just migrate`); CI and Docker builds use the committed `.sqlx` offline metadata (`SQLX_OFFLINE=true`). After changing any query or migration, run `just sqlx-prepare` and commit the `.sqlx` directory.
- `trackscape-discord-migrator` copies the legacy MongoDB data into Postgres (`MONGO_DB_URL` + `DATABASE_URL`). Safe to re-run BEFORE the Postgres stack goes live: without `--truncate-first` it refreshes in place (updates changed rows, restores missing ones, never duplicates); with `--truncate-first` it rebuilds an exact mirror. Row ids are derived deterministically from the old ObjectIds, which is what makes re-runs idempotent. NEVER run it after cutover — Postgres is the source of truth then, and a run would overwrite live data (truncate would wipe it).

## Release (maintainers)

`just newRelease` builds the 4 production images (linux/amd64) and pushes to
Docker Hub (`fatfingers23/trackscape-api`, `-bot`, `-worker`, `-cron-worker`).
Production runs `production/production-docker-compose.yml`.
