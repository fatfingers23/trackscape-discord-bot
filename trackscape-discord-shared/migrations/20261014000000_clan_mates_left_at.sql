-- left_at is the single "no longer a member" marker. NULL means member.
-- It replaces wom_left_at: the WOM sync still sets it, and from now on so
-- do the in-game leave/expel broadcasts and the Discord /expel command.
-- in_wom and last_seen_in_chat_at remain as informational columns only.
ALTER TABLE clan_mates RENAME COLUMN wom_left_at TO left_at;

-- Backfill rule 1: WOM-governed clans. A clan is WOM-governed only when it has
-- a wom_id AND the sync has confirmed at least one member (some row has
-- in_wom = true); a wom_id the sync never managed to use is not evidence.
-- In those clans every row the last sync did not confirm is marked as left,
-- except rows with a non-guest chat line in the last 90 days: chatting in the
-- clan channel as a non-guest is proof of membership the WOM group has not
-- caught up with yet. (One-time rule; the ongoing predicate has no window.)
UPDATE clan_mates cm
SET left_at = COALESCE(cm.left_at, now())
FROM guilds g
WHERE g.guild_id = cm.guild_id
  AND g.wom_id IS NOT NULL
  AND EXISTS (SELECT 1 FROM clan_mates x WHERE x.guild_id = g.guild_id AND x.in_wom)
  AND cm.in_wom = false
  AND (cm."rank" = 'Guest' OR cm.last_seen_in_chat_at IS NULL
       OR cm.last_seen_in_chat_at <= now() - interval '90 days');

-- Backfill rule 2: clans WOM does not govern (no wom_id, or a wom_id the sync
-- never confirmed a member for). Nothing ever recorded these players leaving,
-- so years of chat-created rows accumulate (some clans exceed the in-game cap
-- of 500). One-time cleanup: a row with no evidence in the last 180 days is
-- marked as left. Evidence is the latest of first sighting, last non-guest
-- chat line, last personal best update and last collection log update. Any
-- row cut here comes back on the player's next chat line or broadcast.
UPDATE clan_mates cm
SET left_at = now()
FROM guilds g
WHERE g.guild_id = cm.guild_id
  AND (g.wom_id IS NULL
       OR NOT EXISTS (SELECT 1 FROM clan_mates x WHERE x.guild_id = g.guild_id AND x.in_wom))
  AND cm.left_at IS NULL
  AND GREATEST(
        cm.created_at,
        cm.last_seen_in_chat_at,
        (SELECT max(r.updated_at) FROM personal_best_records r WHERE r.clan_mate_id = cm.id),
        (SELECT max(t.created_at) FROM clan_mate_collection_log_totals t
         WHERE t.guild_id = cm.guild_id AND t.player_id = cm.id)
      ) < now() - interval '180 days';
