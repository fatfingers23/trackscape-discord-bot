type BotInfo = {
    server_count: number;
    connected_users: number;
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
    rank:  string | null,
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
    player_name: string,
    total: number,
}

export type {
    BotInfo,
    Clan,
    ClanDetail,
    ClanMate,
    ClanMateCollectionLogTotalsView
};
