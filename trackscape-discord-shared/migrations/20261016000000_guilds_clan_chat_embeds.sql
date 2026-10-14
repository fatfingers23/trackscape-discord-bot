-- Set by the Discord /clan_chat_embeds command. When false, clan chat lines are
-- posted to the clan chat channel as plain text instead of one embed per line.
-- Broadcast and leagues channels always use embeds.
ALTER TABLE guilds ADD COLUMN clan_chat_embeds boolean NOT NULL DEFAULT true;
