import { defineStore } from 'pinia'
import type { PbActivity } from '@/services/TrackscapeApiTypes'

export const usePbStore = defineStore('pb', {
  state: () => {
    return { selectedActivity: 'Select an Activity' }
  },
  actions: {
    setSelectedActivity(activity: PbActivity) {
      this.selectedActivity = activity._id
    },
  },
  getters:{
    getSelectedActivity: (state) =>{
      return state.selectedActivity
    }
  }
})
