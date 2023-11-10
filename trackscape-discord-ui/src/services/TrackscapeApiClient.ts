import type {BotInfo, Clan} from "@/services/TrackscapeApiTypes";

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

}

