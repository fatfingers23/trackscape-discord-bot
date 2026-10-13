-- Player names are unique per guild ignoring case. The chat job and the
-- WOM sync compare names case-insensitively, but the original constraint
-- was exact, so rows differing only by case crept in. Merge them first
-- (one loser at a time, so three or more duplicates cannot collide on the
-- child tables' unique keys), then swap the constraint for a functional
-- unique index.
DO $$
DECLARE
    dup RECORD;
BEGIN
    FOR dup IN
        SELECT id, keeper_id
        FROM (
            SELECT id,
                   first_value(id) OVER (
                       PARTITION BY guild_id, lower(player_name)
                       ORDER BY (wom_player_id IS NOT NULL) DESC,
                                last_seen_in_chat_at DESC NULLS LAST,
                                created_at ASC,
                                id ASC
                   ) AS keeper_id
            FROM clan_mates
        ) ranked
        WHERE id <> keeper_id
    LOOP
        -- Personal bests: where both rows have the activity the keeper takes
        -- the lower time, then the loser's redundant rows go, then the rest
        -- move across.
        UPDATE personal_best_records kept
        SET time_in_seconds = loser.time_in_seconds,
            updated_at = loser.updated_at
        FROM personal_best_records loser
        WHERE kept.clan_mate_id = dup.keeper_id
          AND loser.clan_mate_id = dup.id
          AND kept.activity_id = loser.activity_id
          AND loser.time_in_seconds < kept.time_in_seconds;

        DELETE FROM personal_best_records loser
        USING personal_best_records kept
        WHERE loser.clan_mate_id = dup.id
          AND kept.clan_mate_id = dup.keeper_id
          AND kept.activity_id = loser.activity_id;

        UPDATE personal_best_records
        SET clan_mate_id = dup.keeper_id
        WHERE clan_mate_id = dup.id;

        -- Collection log totals: higher total wins, earliest created_at kept.
        UPDATE clan_mate_collection_log_totals kept
        SET total = GREATEST(kept.total, loser.total),
            created_at = LEAST(kept.created_at, loser.created_at)
        FROM clan_mate_collection_log_totals loser
        WHERE kept.player_id = dup.keeper_id
          AND loser.player_id = dup.id
          AND kept.guild_id = loser.guild_id;

        DELETE FROM clan_mate_collection_log_totals loser
        USING clan_mate_collection_log_totals kept
        WHERE loser.player_id = dup.id
          AND kept.player_id = dup.keeper_id
          AND kept.guild_id = loser.guild_id;

        UPDATE clan_mate_collection_log_totals
        SET player_id = dup.keeper_id
        WHERE player_id = dup.id;

        -- The PB/clog steps mirror merge_clan_mates in clan_mates.rs, but this identity fold (previous_names, wom_player_id, "rank", created_at, in_wom, left_at, last_seen_in_chat_at) has no Rust counterpart: the WOM planner does that merge in plan_wom_sync.
        -- Fold the loser's identity into the keeper.
        UPDATE clan_mates keeper
        SET previous_names = (
                SELECT COALESCE(array_agg(DISTINCT n), '{}')
                FROM unnest(keeper.previous_names || loser.previous_names || ARRAY[loser.player_name]) AS n
                WHERE n <> keeper.player_name
            ),
            wom_player_id = COALESCE(keeper.wom_player_id, loser.wom_player_id),
            -- A merged row the WOM sync has confirmed is never labelled
            -- Guest: a Guest rank from a chat-created twin predates their
            -- joining. (in_wom here is the pre-fold value of each row.)
            "rank" = CASE
                WHEN keeper.in_wom OR loser.in_wom
                    THEN COALESCE(NULLIF(keeper."rank", 'Guest'), NULLIF(loser."rank", 'Guest'))
                ELSE COALESCE(keeper."rank", loser."rank")
            END,
            created_at = LEAST(keeper.created_at, loser.created_at),
            in_wom = keeper.in_wom OR loser.in_wom,
            left_at = CASE WHEN keeper.left_at IS NULL OR loser.left_at IS NULL THEN NULL ELSE keeper.left_at END,
            last_seen_in_chat_at = GREATEST(keeper.last_seen_in_chat_at, loser.last_seen_in_chat_at)
        FROM clan_mates loser
        WHERE keeper.id = dup.keeper_id
          AND loser.id = dup.id;

        DELETE FROM clan_mates WHERE id = dup.id;
    END LOOP;
END $$;

ALTER TABLE clan_mates DROP CONSTRAINT clan_mates_guild_id_player_name_key;

CREATE UNIQUE INDEX clan_mates_guild_id_lower_player_name_idx
    ON clan_mates (guild_id, lower(player_name));
