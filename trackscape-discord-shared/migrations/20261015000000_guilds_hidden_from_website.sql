-- Set by the Discord /hide_from_website command. A hidden clan is left out of
-- the website's clan list and its pages answer as if the clan does not exist.
-- The bot, the jobs and the drop log export are unaffected.
ALTER TABLE guilds ADD COLUMN hidden_from_website boolean NOT NULL DEFAULT false;
