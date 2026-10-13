<script setup lang="ts">
import { computed, ref } from 'vue';
import type { SelectItem } from '@/components/General/GeneralTypes';


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
});

const emit = defineEmits(['update:modelValue']);

const modelValue = computed({
  get () {
    return props.modelValue;
  },
  set (value) {
    emit('update:modelValue', value);
  }
});

// Unique per instance: popover id and CSS anchor name.
const uid = `text-search-select-${Math.random().toString(36).slice(2, 9)}`;
const inputId = `${uid}-input`;
const optionsId = `${uid}-options`;
const anchorName = `--${uid}`;

const optionsPopover = ref<HTMLElement | null>(null);

const openOptions = () => {
  const el = optionsPopover.value;
  if (el && typeof el.showPopover === 'function') {
    el.showPopover();
  }
};

const setModel = (item: SelectItem) => {
  if (item.value === 0) {
    return;
  }
  modelValue.value = item;
  optionsPopover.value?.hidePopover();
};
</script>

<template>
  <div>
    <label class="label"
           :for="inputId">{{ props.label }}</label>

    <input :id="inputId"
           type="text"
           class="input w-full input-primary"
           placeholder="Type something..."
           v-model="modelValue.label"
           :style="`anchor-name: ${anchorName}`"
           @focus="openOptions"
           @click="openOptions" />

    <ul ref="optionsPopover"
        :id="optionsId"
        popover
        class="dropdown dropdown-end menu bg-base-100 rounded-box p-2 shadow-sm max-h-96 overflow-y-auto"
        :style="`position-anchor: ${anchorName}; width: anchor-size(width)`">
      <li v-for="(item, index) in options"
          :key="index">
        <button type="button"
                @click="setModel(item)">{{ item.label }}</button>
      </li>
    </ul>
  </div>
</template>
