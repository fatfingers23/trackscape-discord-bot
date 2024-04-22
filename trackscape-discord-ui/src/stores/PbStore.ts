import { defineStore } from 'pinia';
import type { PbActivity, PbRecord } from '@/services/TrackscapeApiTypes';
import TrackscapeApiClient from '@/services/TrackscapeApiClient';
import router from '@/router';
const client = new TrackscapeApiClient(import.meta.env.VITE_API_BASE_URL);

export const usePbStore = defineStore('pb', {
  state: () => {
    return {
      selectedActivity: 'Select an Activity',
      selectedActivityName: '',
      clanId: '',
      records: [] as PbRecord[],
    };
  },
  actions: {
    async fetchPbRecords() {
      return await client.getTrackScapePbRecords(this.clanId, this.selectedActivity).then((records: PbRecord[]) => {
        let rank = 1;
        records.forEach((record) => {
          record.rank = rank;
          rank++;

        });
        this.records = records;
        router.replace({ name: 'personal-best', params: { clanId: this.clanId, activityId: this.selectedActivity } });
      });
    },
    async setSelectedActivity(activity: PbActivity, guildId: string) {
      this.selectedActivity = activity._id;
      this.selectedActivityName = activity.activity_name;
      this.clanId = guildId;
      await this.fetchPbRecords();
    },
  },
  getters:{
    getSelectedActivity: (state) =>{
      return state.selectedActivity;
    },
    getRecords: (state) =>{
      return state.records;
    }
  }
});
