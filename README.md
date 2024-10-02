# TrackScape Discord Bot

![TrackScape Logo](images/trackscape_logo.png)


[![](https://img.shields.io/badge/Discord-%235865F2.svg?style=for-the-badge&logo=discord&logoColor=white)](https://discord.com/api/oauth2/authorize?client_id=864626697327869952&permissions=2147568704&scope=bot)


## Features
* Receive in game Clan Chat messages in Discord via the RuneLite TrackScape Connector plugin
* Send messages to the In Game Clan Chat via Discord. This uses the RuneLite TrackScape Connector plugin to send messages to the Clan Chat.
* The bot sends embed and styled Broadcast Messages to a Discord Channel.

### Getting a chat in in game from Discord
![In Game Chat](images/discord-to-clan-chat.gif)

### Clan Chat from game in Discord
![Clan Chat Broadcast](images/clan_chat_broadcast.png)
***
## Commands
By default, only Discord users with Manage Server permissions can use the majorty of the commands. This can be changed via Integrations. 

> `/info` - Get information about the bot 

> `/set_broadcast_channel channel: {channel}` - Sets the to receive emded Broadcast messages

> `/set_clan_chat_channel channel: {channel}` - Sets the channel to receive Clan Chat messages and records messages sent to Clan Chat in game

> `/get_verification_code` - Get the verification code to link your RuneLite TrackScape Connector plugin to the bot


***  
## Broadcast Types

### Diary Completed 
![Diary Completed Broadcast](images/diary_completed_broadcast.png)

### Item Drop
![Item Drop Broadcast](images/item_drop_broadcast.png)

### Raid Drop
![Raid Drop Broadcast](images/raid_drop_broadcast.png)

### Pet Drop
![Pet Drop Broadcast](images/pet_drop_broadcast.png)

### Quest Completed
![Quest Completed Broadcast](images/quest_completed_broadcast.png)

### and more!

### Setup your own bot
Coming soon! Everything needed is in the `.env` and `docker-compose.yml` files. But detailed instructions will come soon.

### Contributing
Coming soon!

## Future Features
All of the features below depends on data gotten from Clan Chat messages. So if it is not sent to the bot or does not broadcast in
CC will not be able to be sent to Discord and parsed. This is done by the RuneLite TrackScape Connector
* Get clan member activity. Check for last time xp gained or last time a chat was sent
* Pking
  - [ ] Check your K/D Pk ratio from Discord bot
  - [ ] Check your total kills and deaths from Discord bot
  - [ ] Check Gold earned and lost from Discord bot
  - [x] In clan leaderboards
* [x] Collection Log Leaderboards -  See where you log stands with the rest of the clan
* [x] Bossing PB Leaderboards - See where you stand with the rest of the clan for fastest kills
* Simple Team Bingo - to be determined. May not happen
  - [ ] Bingo tiles by certain drops 
    - [ ] Bingo tiles by sets. Barrows, Godsword, etc
  - [ ] Bingo tiles by certain bosses
  - [ ] Bingo tiles by certain xp gain
  - [ ] Ability to manually check/uncheck team tiles
  - [ ] Ability to auto check tiles
  - [ ] Check your bingo card from Discord bot

  # Developer setup
  This guide will help you get started with running TrackScape to make changes
  ## Requirements
  * [Rust🦀](https://www.rust-lang.org/tools/install)
  * Docker or [Docker Desktop](https://www.docker.com/products/docker-desktop/)
     * Rather not use docker? Then make sure you have a redis and mongo db instance running
  * Create a dev discord bot. [Can follow this great discord.py] guide(https://discordpy.readthedocs.io/en/stable/discord.html). Need to make sure you have MESSAGE CONTENT INTENT toggled on for access to message content for when developing any features that read discord messages and send them to the TrackScape plugin.
  * Make a copy of [.env.save](.env.save)
    * `MONGO_USERNAME` and `MONGO_PASSWORD` can be w/e you like to secure your development server
    * `DISCORD_TOKEN` This is the discord token created from setting up a discord bot
    * `MANAGEMENT_API_KEY` can be set to w/e. It is used to password protect some endpoints of the API for communication between the bot and the api
    * The bot and api are ran via [shuttle](https://github.com/shuttle-hq/shuttle). But uses an older version. So may find you need to install it via `cargo binstall cargo-shuttle -y --version 0.37.0`

## Setup
1. Make sure docker desktop is running and run `docker compose up -d` in the root of the project. Or make sure your monogodb and redis services are running
2. Run the project with `cargo shuttle run`. And that's it! The API and bot should be running.

## Further setup or questions
If you do have any issues or questions can join the bots discord and feel free to ask questions there! [TrackScape Discord](https://discord.gg/kRM6Ydf5j9)
