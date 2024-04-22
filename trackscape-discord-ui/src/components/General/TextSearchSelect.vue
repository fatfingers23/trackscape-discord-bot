<script setup lang="ts">
import { computed } from 'vue'
import type { SelectItem } from '@/components/General/GeneralTypes'


const props = defineProps({
  modelValue: {
    type: Object as () => SelectItem,
    required: true,
  },
  label: {
    type: String,
    required: false,
    default: ''
  },
  options: {
    type: Array<SelectItem>,
    required: false,
    default: () => [
      {
        label: 'Type to search...',
        value: 0
      }
    ]
  }
})

const emit = defineEmits(['update:modelValue'])

const modelValue = computed({
  get () {
    return props.modelValue
  },
  set (value) {
    emit('update:modelValue', value)
  }
})

const setModel = (item: SelectItem) => {
  if (item.value === 0) {
    return
  }
  modelValue.value = item;
}


// https://reacthustle.com/blog/how-to-implement-a-react-autocomplete-input-using-daisyui
</script>

<template>
  <div>
    <div class="">

      <label class="label">
        <span class="text-base label-text">{{ props.label }}</span>
      </label>

      <div class="dropdown dropdown-end w-full">
        <input
          type="text"
          class="input input-bordered w-full input-primary"
          placeholder="Type something..."
          v-model="modelValue.label"
        >
        <div
          class="bg-base-200 top-14 overflow-auto flex-col rounded-md w-full"
        >
          <ul
            tabindex="0"
            class="p-2 shadow menu dropdown-content bg-base-100 rounded-box w-full"
          >
            <li
              v-for="(item, index) in options"
              :key="index"
              class="z-200"
            >
              <div @click="setModel(item)">
                {{ item.label }}
              </div>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>
<style scoped></style>
