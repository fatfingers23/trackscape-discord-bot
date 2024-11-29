type BotInfo = {
  server_count: number;
  connected_users: number;
  total_chat_messages_for_today: number | null
}

type Clan = {
  id: string,
  name: string,
  registered_members: number,
}

type ClanMate = {
  id: string,
  guild_id: string,
  player_name: string,
  wom_player_id: Number,
  previous_names: string[],
  rank: string | null,
  created_at: string,
}

type ClanDetail = {
  id: string,
  name: string,
  discord_guild_id: string,
  registered_members: number,
  members: ClanMate[]
}

type ClanMateCollectionLogTotalsView = {
  rank: number,
  total: number,
  clan_mate: ClanMate|null,
}

type BroadcastMessage = {
  player_it_happened_to: string,
  type_of_broadcast: string,
  message: string,
  icon_url: string|null,
  title: string,
  item_quantity: number|null,
}


type Broadcast = {
  id: string,
  guild_id: string,
  broadcast: BroadcastMessage,
  created_at: string
}

type PbActivity = {
  _id: string
  activity_name: string,
  created_at: string,
}

type PbRecord = {
  rank: number,
  time_in_seconds: number,
  clan_mate: ClanMate|null,
}

export type {
  BotInfo,
  Clan,
  ClanDetail,
  ClanMate,
  ClanMateCollectionLogTotalsView,
  Broadcast,
  BroadcastMessage,
  PbActivity,
  PbRecord

};
