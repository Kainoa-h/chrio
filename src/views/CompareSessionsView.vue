<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { commands, type Session } from "@/bindings";
import { ArrowLeft, ChevronsLeft, ChevronLeft, ChevronRight, ChevronsRight } from "lucide-vue-next";
import RatioImage from "@/components/RatioImage.vue";
import ImageCropper from "@/components/ImageCropper.vue";

const route = useRoute();
const router = useRouter();
const clientId = Number(route.params.clientId);
const session1Id = Number(route.query.s1);
const session2Id = Number(route.query.s2);

const allSessions = ref<Session[]>([]);
const session1 = ref<Session | null>(null);
const session2 = ref<Session | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);

// Store base64 images: images[sessionId][type] = base64String
const images = ref<Record<number, Record<string, string>>>({});

const showCropper = ref(false);
const activeCropSessionId = ref<number | null>(null);
const activeCropType = ref<string | null>(null);

// Available sessions for the left side (must be older than session 2)
const availableLeftSessions = computed(() => {
  if (!session2.value) return [];
  // sessions are sorted DESC by backend (newest first)
  // We want sessions strictly older than session2 (lower session number or index > session2 index)
  return allSessions.value.filter(s => s.session_number < session2.value!.session_number);
});

// Current index in the available list (0 is newest/closest to session 2, length-1 is oldest)
const currentLeftIndex = computed(() => {
  if (!session1.value) return -1;
  return availableLeftSessions.value.findIndex(s => s.id === session1.value!.id);
});

async function updateSession1(newSession: Session) {
  session1.value = newSession;
  await loadSessionImages(newSession);
  // Update URL without reloading
  router.replace({ 
    query: { ...route.query, s1: newSession.id } 
  });
}

function jumpToOldest() {
  const sessions = availableLeftSessions.value;
  if (sessions.length > 0) {
    updateSession1(sessions[sessions.length - 1]);
  }
}

function moveBack() {
  const index = currentLeftIndex.value;
  const sessions = availableLeftSessions.value;
  if (index < sessions.length - 1) {
    updateSession1(sessions[index + 1]);
  }
}

function moveForward() {
  const index = currentLeftIndex.value;
  const sessions = availableLeftSessions.value;
  if (index > 0) {
    updateSession1(sessions[index - 1]);
  }
}

function jumpToNewestPossible() {
  const sessions = availableLeftSessions.value;
  if (sessions.length > 0) {
    updateSession1(sessions[0]);
  }
}

async function loadImage(sessionId: number, type: string, path: string | null) {
  if (!path) return;
  // Don't reload if we have it
  if (images.value[sessionId]?.[type]) return;

  try {
    const result = await commands.readImageBase64(path);
    if (result.status === "ok") {
      if (!images.value[sessionId]) images.value[sessionId] = {};
      images.value[sessionId][type] = result.data;
    }
  } catch (e) {
    console.error(`Failed to load image ${type} for session ${sessionId}`, e);
  }
}

async function loadSessionImages(session: Session) {
  await Promise.all([
    loadImage(session.id, 'anterior', session.anterior),
    loadImage(session.id, 'posterior', session.posterior),
    loadImage(session.id, 'right_lateral', session.right_lateral),
    loadImage(session.id, 'left_lateral', session.left_lateral),
  ]);
}

onMounted(async () => {
  try {
    const result = await commands.getClientSessions(clientId);
    if (result.status === "ok") {
      allSessions.value = result.data;
      session1.value = allSessions.value.find(s => s.id === session1Id) || null;
      session2.value = allSessions.value.find(s => s.id === session2Id) || null;

      if (!session1.value || !session2.value) {
        error.value = "Could not find one or both sessions.";
        return;
      }

      await Promise.all([
        loadSessionImages(session1.value),
        loadSessionImages(session2.value)
      ]);
    } else {
      error.value = result.error;
    }
  } catch (e: any) {
    error.value = e.message || "An unknown error occurred";
  } finally {
    loading.value = false;
  }
});

function getParsedCrop(session: Session | null, type: string) {
  if (!session) return null;
  const key = `${type}_crop` as keyof Session;
  const cropString = session[key];
  if (typeof cropString === 'string') {
    try {
      return JSON.parse(cropString);
    } catch (e) {
      console.error("Failed to parse crop data", e);
    }
  }
  return null;
}

function formatDate(dateString: string | undefined) {
  if (!dateString) return "-";
  const date = new Date(dateString);
  return new Intl.DateTimeFormat('en-GB', { 
    day: '2-digit', 
    month: 'long', 
    year: 'numeric' 
  }).format(date);
}

function openCropper(sessionId: number, type: string) {
  if (images.value[sessionId]?.[type]) {
    activeCropSessionId.value = sessionId;
    activeCropType.value = type;
    showCropper.value = true;
  }
}

async function handleCropSave(cropData: { x: number, y: number, width: number }) {
  if (!activeCropSessionId.value || !activeCropType.value) return;

  const cropString = JSON.stringify(cropData);
  const sessionId = activeCropSessionId.value;
  const type = activeCropType.value;

  try {
    const result = await commands.updateSessionCrop(sessionId, type, cropString);
    if (result.status === "ok") {
      // Update local session state to reflect change immediately
      const targetSession = [session1.value, session2.value].find(s => s?.id === sessionId);
      if (targetSession) {
        (targetSession as any)[`${type}_crop`] = cropString;
      }
    } else {
      console.error("Failed to save crop", result.error);
    }
  } catch (e) {
    console.error("Error saving crop", e);
  }
}

const activeInitialCrop = computed(() => {
  if (!activeCropSessionId.value || !activeCropType.value) return null;
  const session = [session1.value, session2.value].find(s => s?.id === activeCropSessionId.value);
  return getParsedCrop(session || null, activeCropType.value);
});

const imageTypes = [
  { key: 'anterior', label: 'Anterior' },
  { key: 'posterior', label: 'Posterior' },
  { key: 'right_lateral', label: 'Right Lateral' },
  { key: 'left_lateral', label: 'Left Lateral' },
];
</script>

<template>
  <div class="p-8 max-w-6xl mx-auto pb-20">
    <div class="flex items-center mb-6">
      <button 
        @click="router.back()" 
        class="mr-4 p-2 rounded-full hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
      >
        <ArrowLeft class="h-6 w-6 text-gray-600 dark:text-gray-300" />
      </button>
      <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Compare Sessions</h1>
    </div>

    <div v-if="loading" class="text-center py-8 text-gray-500">
      Loading comparison...
    </div>

    <div v-else-if="error" class="text-center py-8 text-red-500">
      Error: {{ error }}
    </div>

    <div v-else class="grid grid-cols-2 gap-8">
      <!-- Sticky Header Container -->
      <div class="sticky top-0 z-10 col-span-2 bg-transparent pb-4 -mx-8 px-8"> <!-- Negative margin to extend full width of original container -->
        <div class="grid grid-cols-2 gap-8">
          <!-- Session 1 Header -->
          <div class="bg-gray-900 p-4 border border-gray-700">
            <div class="flex items-center justify-between gap-2">
              <div class="flex gap-1">
                <button 
                  @click="jumpToOldest" 
                  :disabled="currentLeftIndex >= availableLeftSessions.length - 1"
                  class="p-1 text-gray-400 hover:text-white disabled:opacity-30 disabled:hover:text-gray-400"
                  title="Jump to First Session"
                >
                  <ChevronsLeft class="w-5 h-5" />
                </button>
                <button 
                  @click="moveBack" 
                  :disabled="currentLeftIndex >= availableLeftSessions.length - 1"
                  class="p-1 text-gray-400 hover:text-white disabled:opacity-30 disabled:hover:text-gray-400"
                  title="Previous Session"
                >
                  <ChevronLeft class="w-5 h-5" />
                </button>
              </div>

              <div class="flex-1 mx-2">
                <select 
                  v-if="session1"
                  :value="session1.id"
                  @change="e => updateSession1(allSessions.find(s => s.id === Number((e.target as HTMLSelectElement).value))!)"
                  class="w-full bg-gray-800 border border-gray-600 text-white text-sm rounded px-2 py-1 focus:ring-1 focus:ring-blue-500 outline-none"
                >
                  <option 
                    v-for="s in availableLeftSessions" 
                    :key="s.id" 
                    :value="s.id"
                  >
                    Session #{{ s.session_number }} - {{ formatDate(s.datetime) }}
                  </option>
                </select>
              </div>

              <div class="flex gap-1">
                <button 
                  @click="moveForward" 
                  :disabled="currentLeftIndex <= 0"
                  class="p-1 text-gray-400 hover:text-white disabled:opacity-30 disabled:hover:text-gray-400"
                  title="Next Session"
                >
                  <ChevronRight class="w-5 h-5" />
                </button>
                <button 
                  @click="jumpToNewestPossible" 
                  :disabled="currentLeftIndex <= 0"
                  class="p-1 text-gray-400 hover:text-white disabled:opacity-30 disabled:hover:text-gray-400"
                  title="Jump to -1 of Right Session"
                >
                  <ChevronsRight class="w-5 h-5" />
                </button>
              </div>
            </div>
          </div>

          <!-- Session 2 Header -->
          <div class="bg-gray-900 p-4 border border-gray-700 text-center flex flex-col justify-center h-[62px]">
            <h2 class="text-xl font-bold text-white leading-tight">Session #{{ session2?.session_number }}</h2>
            <p class="text-gray-400 text-xs leading-tight">{{ formatDate(session2?.datetime) }}</p>
          </div>
        </div>
      </div>
      <!-- End Sticky Header Container -->

      <!-- Images Rows -->
      <template v-for="type in imageTypes" :key="type.key">
        <!-- Label Row -->
        <div class="col-span-2 text-center py-2 border-b border-gray-700">
          <h3 class="text-lg font-medium text-gray-300">{{ type.label }}</h3>
        </div>

        <!-- Image 1 -->
        <RatioImage 
          :src="session1 && images[session1.id]?.[type.key] ? images[session1.id][type.key] : null" 
          :crop="getParsedCrop(session1, type.key)"
          empty-text="No Image" 
          container-class="w-full cursor-pointer hover:ring-2 hover:ring-blue-500 transition-all" 
          @click="session1 && openCropper(session1.id, type.key)"
        />

        <!-- Image 2 -->
        <RatioImage 
          :src="session2 && images[session2.id]?.[type.key] ? images[session2.id][type.key] : null" 
          :crop="getParsedCrop(session2, type.key)"
          empty-text="No Image" 
          container-class="w-full cursor-pointer hover:ring-2 hover:ring-blue-500 transition-all" 
          @click="session2 && openCropper(session2.id, type.key)"
        />
      </template>

      <!-- Data Comparison -->
      <div class="col-span-2 mt-8">
        <h3 class="text-xl font-bold text-gray-200 mb-4 border-b border-gray-700 pb-2">Data Comparison</h3>
        
        <div class="grid grid-cols-2 gap-8">
          <!-- Session 1 Data -->
          <div class="bg-gray-900 rounded-lg border border-gray-700 p-4 space-y-3">
             <div class="flex justify-between items-center border-b border-gray-800 pb-2">
               <span class="text-gray-400 font-medium">Height</span>
               <span class="text-gray-200 font-bold">{{ session1?.height ? session1.height + ' cm' : '-' }}</span>
             </div>
             <div class="flex justify-between items-center border-b border-gray-800 pb-2">
               <span class="text-gray-400 font-medium">Weight</span>
               <span class="text-gray-200 font-bold">{{ session1?.weight ? session1.weight + ' kg' : '-' }}</span>
             </div>
             <div class="pt-1">
               <span class="text-gray-400 font-medium block mb-1">Notes</span>
               <div class="text-gray-300 text-sm bg-gray-950 p-2 rounded border border-gray-800 min-h-[3rem] whitespace-pre-wrap">{{ session1?.notes || '-' }}</div>
             </div>
          </div>

          <!-- Session 2 Data -->
          <div class="bg-gray-900 rounded-lg border border-gray-700 p-4 space-y-3">
             <div class="flex justify-between items-center border-b border-gray-800 pb-2">
               <span class="text-gray-400 font-medium">Height</span>
               <span class="text-gray-200 font-bold">{{ session2?.height ? session2.height + ' cm' : '-' }}</span>
             </div>
             <div class="flex justify-between items-center border-b border-gray-800 pb-2">
               <span class="text-gray-400 font-medium">Weight</span>
               <span class="text-gray-200 font-bold">{{ session2?.weight ? session2.weight + ' kg' : '-' }}</span>
             </div>
             <div class="pt-1">
               <span class="text-gray-400 font-medium block mb-1">Notes</span>
               <div class="text-gray-300 text-sm bg-gray-950 p-2 rounded border border-gray-800 min-h-[3rem] whitespace-pre-wrap">{{ session2?.notes || '-' }}</div>
             </div>
          </div>
        </div>
      </div>
    </div>

    <ImageCropper 
      :show="showCropper"
      :image-src="(activeCropSessionId && activeCropType) ? images[activeCropSessionId]?.[activeCropType] : null"
      :initial-crop="activeInitialCrop"
      :image-type="activeCropType"
      @close="showCropper = false"
      @save="handleCropSave"
    />
  </div>
</template>