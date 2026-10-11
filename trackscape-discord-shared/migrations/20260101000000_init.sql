-- Initial schema for the PostgreSQL port of the TrackScape Discord bot.
--
-- Conventions:
-- * Discord snowflakes (guild/channel/player ids) are stored as BIGINT.
--   They are u64 in Rust; the DB layer casts at the boundary.
-- * Row ids are UUID v4, generated client side (mirrors the old client side
--   ObjectId generation).
-- * Enums are stored as TEXT and converted in Rust via their existing serde
--   derives, so adding a variant never needs a migration.
-- * Embedded Mongo documents (drop_item, broadcast) are JSONB.
-- * All guild scoped tables cascade from guilds, so deleting a guild cleans
--   up its data (the old Mongo layer orphaned these rows).

-- Lookup table for verification codes: one row per registered Discord guild.
CREATE TABLE guilds (
    id uuid PRIMARY KEY,
    guild_id bigint NOT NULL UNIQUE,
    clan_name text NULL,
    broadcast_channel bigint NULL,
    clan_chat_channel bigint NULL,
    leagues_broadcast_channel bigint NULL,
    drop_price_threshold bigint NULL,
    disallowed_broadcast_types text[] NOT NULL DEFAULT '{}',
    verification_code text NOT NULL,
    hashed_verification_code text NOT NULL UNIQUE,
    min_quest_difficulty text NULL,
    min_diary_tier text NULL,
    pk_value_threshold bigint NULL,
    wom_id bigint NULL,
    created_at timestamptz NULL,
    custom_drop_broadcast_filter jsonb NULL,
    collection_log_max_percentage double precision NULL
);

CREATE TABLE clan_mates (
    id uuid PRIMARY KEY,
    guild_id bigint NOT NULL REFERENCES guilds (guild_id) ON DELETE CASCADE,
    player_name text NOT NULL,
    wom_player_id bigint NULL,
    previous_names text[] NOT NULL DEFAULT '{}',
    "rank" text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (guild_id, player_name)
);

CREATE INDEX clan_mates_player_name_idx ON clan_mates (player_name);
CREATE INDEX clan_mates_guild_id_idx ON clan_mates (guild_id);

-- One row per drop broadcast (embedded item document in drop_item).
CREATE TABLE drop_logs (
    id uuid PRIMARY KEY,
    guild_id bigint NOT NULL REFERENCES guilds (guild_id) ON DELETE CASCADE,
    drop_item jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX drop_logs_guild_created_idx ON drop_logs (guild_id, created_at);

-- One row per broadcast that was sent to a guild's channel.
CREATE TABLE broadcasts (
    id uuid PRIMARY KEY,
    guild_id bigint NOT NULL REFERENCES guilds (guild_id) ON DELETE CASCADE,
    broadcast jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX broadcasts_guild_created_idx ON broadcasts (guild_id, created_at DESC);

-- Catalog of personal best activities (case-insensitively unique).
CREATE TABLE personal_best_activities (
    id uuid PRIMARY KEY,
    activity_name text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX personal_best_activities_name_lower_idx
    ON personal_best_activities (lower(activity_name));
CREATE INDEX personal_best_activities_name_idx ON personal_best_activities (activity_name);

-- Best time a clan mate has for an activity. A new time only replaces the
-- stored one when it is better (lower); enforced in the DB layer upsert.
CREATE TABLE personal_best_records (
    id uuid PRIMARY KEY,
    clan_mate_id uuid NOT NULL REFERENCES clan_mates (id) ON DELETE CASCADE,
    activity_id uuid NOT NULL REFERENCES personal_best_activities (id) ON DELETE CASCADE,
    guild_id bigint NOT NULL REFERENCES guilds (guild_id) ON DELETE CASCADE,
    time_in_seconds double precision NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (clan_mate_id, activity_id)
);

CREATE INDEX personal_best_records_guild_activity_idx
    ON personal_best_records (guild_id, activity_id);

-- Latest collection log slot total a clan mate has reached.
CREATE TABLE clan_mate_collection_log_totals (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id bigint NOT NULL REFERENCES guilds (guild_id) ON DELETE CASCADE,
    player_id uuid NOT NULL REFERENCES clan_mates (id) ON DELETE CASCADE,
    total bigint NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (guild_id, player_id)
);
