<script setup lang="ts">
import { computed } from "vue";
import { type Session } from "@/bindings";
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight, Crop, X } from "lucide-vue-next";
import RatioImage from "@/components/RatioImage.vue";

const props = defineProps<{
  session: Session;
  allSessions: Session[]; 
  olderLimit: number | null;
  newerLimit: number | null;
  images: Record<string, string>;
  canRemove: boolean;
  isSelectMode: boolean;
  selectedKeys: string[];
}>();

const emit = defineEmits<{
  (e: 'update:session', newSession: Session): void;
  (e: 'view-image', type: string): void;
  (e: 'open-cropper', type: string): void;
  (e: 'select-image', type: string): void;
  (e: 'remove'): void;
}>();

// Helper to format date
function formatDate(dateString: string | undefined) {
  if (!dateString) return "-";
  const date = new Date(dateString);
  return new Intl.DateTimeFormat('en-GB', { 
    day: '2-digit', 
    month: 'short', 
    year: '2-digit' 
  }).format(date);
}

// Navigation candidates
// We need sessions that are strictly between olderLimit and newerLimit.
// And exclude the current session itself.
const availableCandidates = computed(() => {
  return props.allSessions.filter(s => {
    const n = s.session_number;
    // If olderLimit exists, s must be > olderLimit
    if (props.olderLimit !== null && n <= props.olderLimit) return false;
    // If newerLimit exists, s must be < newerLimit
    if (props.newerLimit !== null && n >= props.newerLimit) return false;
    return true;
  }).sort((a, b) => a.session_number - b.session_number); // Sort ASC
});

const currentIndex = computed(() => {
  return availableCandidates.value.findIndex(s => s.id === props.session.id);
});

function changeSession(s: Session) {
  emit('update:session', s);
}

function jumpToOldest() {
  if (availableCandidates.value.length > 0) {
    changeSession(availableCandidates.value[0]);
  }
}

function movePrev() { // To an older session within constraints
  if (currentIndex.value > 0) {
    changeSession(availableCandidates.value[currentIndex.value - 1]);
  }
}

function moveNext() { // To a newer session within constraints
  if (currentIndex.value < availableCandidates.value.length - 1) {
    changeSession(availableCandidates.value[currentIndex.value + 1]);
  }
}

function jumpToNewest() {
  if (availableCandidates.value.length > 0) {
    changeSession(availableCandidates.value[availableCandidates.value.length - 1]);
  }
}

function getParsedCrop(type: string) {
  const key = `${type}_crop` as keyof Session;
  const cropString = props.session[key];
  if (typeof cropString === 'string') {
    try {
      return JSON.parse(cropString);
    } catch (e) {
      console.error("Failed to parse crop data", e);
    }
  }
  return null;
}

const imageTypes = [
  { key: 'anterior', label: 'Anterior' },
  { key: 'posterior', label: 'Posterior' },
  { key: 'right_lateral', label: 'Right Lateral' },
  { key: 'left_lateral', label: 'Left Lateral' },
];
</script>

<template>
  <div class="flex-shrink-0 w-80 flex flex-col bg-white border border-gray-200 rounded-lg shadow-sm">
    <!-- Sticky Header -->
    <div class="sticky top-0 z-20 bg-gray-50 border-b border-gray-200 p-3 text-center shadow-sm">
        <!-- Remove Button -->
        <div v-if="canRemove" class="absolute top-1 right-1">
             <button 
                @click="emit('remove')"
                class="p-1.5 rounded-full bg-gray-100 text-red-500 hover:bg-red-100 transition-colors"
                title="Remove Column"
            >
                <X class="w-3 h-3" />
            </button>
        </div>

        <div class="flex items-center justify-between mb-2 pt-1">
            <div class="flex gap-0.5">
                 <button 
                  @click="jumpToOldest" 
                  :disabled="currentIndex <= 0"
                  class="p-1 text-gray-500 hover:text-blue-600 disabled:opacity-30 disabled:hover:text-gray-500 transition-colors"
                  title="Oldest allowed"
                >
                  <ChevronsLeft class="w-4 h-4" />
                </button>
                <button 
                  @click="movePrev" 
                  :disabled="currentIndex <= 0"
                  class="p-1 text-gray-500 hover:text-blue-600 disabled:opacity-30 disabled:hover:text-gray-500 transition-colors"
                  title="Previous allowed"
                >
                  <ChevronLeft class="w-4 h-4" />
                </button>
            </div>

            <div class="flex flex-col">
                <span class="font-bold text-gray-900">Session #{{ session.session_number }}</span>
                <span class="text-xs text-gray-500">{{ formatDate(session.datetime) }}</span>
            </div>

            <div class="flex gap-0.5">
                <button 
                  @click="moveNext" 
                  :disabled="currentIndex >= availableCandidates.length - 1"
                  class="p-1 text-gray-500 hover:text-blue-600 disabled:opacity-30 disabled:hover:text-gray-500 transition-colors"
                  title="Next allowed"
                >
                  <ChevronRight class="w-4 h-4" />
                </button>
                <button 
                  @click="jumpToNewest" 
                  :disabled="currentIndex >= availableCandidates.length - 1"
                  class="p-1 text-gray-500 hover:text-blue-600 disabled:opacity-30 disabled:hover:text-gray-500 transition-colors"
                  title="Newest allowed"
                >
                  <ChevronsRight class="w-4 h-4" />
                </button>
            </div>
        </div>
        
        <!-- Dropdown for quick jump -->
        <select 
            :value="session.id"
            @change="e => changeSession(availableCandidates.find(s => s.id === Number((e.target as HTMLSelectElement).value))!)"
            class="w-full text-xs bg-white border border-gray-300 text-gray-700 rounded px-2 py-1 focus:ring-1 focus:ring-blue-500 outline-none"
        >
            <option 
            v-for="s in availableCandidates" 
            :key="s.id" 
            :value="s.id"
            >
            #{{ s.session_number }} ({{ formatDate(s.datetime) }})
            </option>
        </select>
    </div>

    <!-- Content (Scrollable relative to column usually, but here the whole page scrolls vertically?) -->
    <!-- If we want the header sticky, the container needs to be tall. -->
    <div class="p-3 space-y-6">
        <div v-for="type in imageTypes" :key="type.key" class="space-y-1">
            <div class="flex justify-between items-center px-1">
                <span class="text-xs font-semibold text-gray-500 uppercase tracking-wider">{{ type.label }}</span>
                <button 
                    @click="emit('open-cropper', type.key)"
                    class="text-gray-400 hover:text-blue-500 transition-colors"
                    title="Crop Image"
                >
                    <Crop class="w-3 h-3" />
                </button>
            </div>
            <RatioImage 
                :src="images[type.key]" 
                :crop="getParsedCrop(type.key)"
                empty-text="No Image" 
                :container-class="[
                    'w-full rounded border transition-all duration-200 cursor-pointer',
                    isSelectMode 
                        ? (selectedKeys.includes(`${session.id}-${type.key}`) 
                            ? 'border-blue-500 ring-4 ring-blue-500 ring-opacity-50' 
                            : 'border-gray-200 hover:border-blue-300 hover:bg-blue-50')
                        : 'border-gray-200 hover:border-blue-400'
                ].join(' ')"
                @click="isSelectMode ? emit('select-image', type.key) : emit('view-image', type.key)"
            />
        </div>

        <!-- Data -->
        <div class="border-t border-gray-200 pt-4 space-y-3">
             <div class="flex justify-between items-center text-sm">
               <span class="text-gray-600">Height</span>
               <span class="font-medium text-gray-900">{{ session.height ? session.height + ' cm' : '-' }}</span>
             </div>
             <div class="flex justify-between items-center text-sm">
               <span class="text-gray-600">Weight</span>
               <span class="font-medium text-gray-900">{{ session.weight ? session.weight + ' kg' : '-' }}</span>
             </div>
             <div class="pt-1">
               <span class="text-xs text-gray-500 uppercase font-semibold block mb-1">Notes</span>
               <div class="text-sm text-gray-800 bg-gray-50 p-2 rounded border border-gray-100 min-h-[2rem] whitespace-pre-wrap">{{ session.notes || '-' }}</div>
             </div>
        </div>
    </div>
  </div>
</template>
