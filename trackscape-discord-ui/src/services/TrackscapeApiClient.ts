import type {
    BotInfo,
    Broadcast,
    Clan,
    ClanDetail,
    ClanMateCollectionLogTotalsView
} from '@/services/TrackscapeApiTypes'

export default class TrackscapeApiClient {

    protected baseUrl: string;

    constructor(baseUrl: string) {
        this.baseUrl = `${baseUrl ?? ""}/api`;
    }

    public async get<T>(path: string): Promise<T> {
        const response = await fetch(`${this.baseUrl}${path}`);
        return await response.json();
    }

    public async getBotInfo(): Promise<BotInfo>   {
        return this.get<BotInfo>("/info/landing-page-info");
    }

    public async getClans(): Promise<Clan[]> {
        return this.get<Clan[]>("/clans/list");
    }

    public async getClanDetail(clanId: string): Promise<ClanDetail> {
        return this.get<ClanDetail>(`/clans/${clanId}/detail`);
    }

    public async getCollectionLogLeaderboard(clanId: string): Promise<ClanMateCollectionLogTotalsView[]> {
        return this.get<ClanMateCollectionLogTotalsView[]>(`/clans/${clanId}/collection-log`);
    }

    public async getBroadcasts(clanID: string, limit: number): Promise<Broadcast[]> {
        return this.get<Broadcast[]>(`/clans/${clanID}/broadcasts/${limit}`);
    }

}

