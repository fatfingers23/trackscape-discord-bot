-- Soft removal for the WOM sync: members that leave the Wise Old Man group
-- are flagged instead of deleted, so PB records and collection log totals
-- survive (their foreign keys ON DELETE CASCADE). last_seen_in_chat_at
-- records the most recent clan-chat sighting so chat-discovered members
-- stay "active" even when they are not (yet) in the WOM group.
ALTER TABLE clan_mates
    ADD COLUMN in_wom boolean NOT NULL DEFAULT false,
    ADD COLUMN wom_left_at timestamptz NULL,
    ADD COLUMN last_seen_in_chat_at timestamptz NULL;

-- Rows that already carry a WOM player id are treated as current members.
UPDATE clan_mates SET in_wom = (wom_player_id IS NOT NULL);
