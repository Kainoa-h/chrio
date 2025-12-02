<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { commands, type Session } from "@/bindings";
import { ArrowLeft, Plus, CheckSquare, Check, X } from "lucide-vue-next";
import ImageCropper from "@/components/ImageCropper.vue";
import CompareSessionColumn from "@/components/CompareSessionColumn.vue";
import ImageModal from "@/components/ImageModal.vue";
import ImageComparisonModal from "@/components/ImageComparisonModal.vue";

const route = useRoute();
const router = useRouter();
const clientId = Number(route.params.clientId);

// Initial sessions from query params
const initialSessionIds = (route.query.ids as string)?.split(',').map(Number) || [];
// Fallback to old params if ids is missing
if (initialSessionIds.length === 0) {
    if (route.query.s1) initialSessionIds.push(Number(route.query.s1));
    if (route.query.s2) initialSessionIds.push(Number(route.query.s2));
}

const allSessions = ref<Session[]>([]);
// Visible session IDs in order (Left to Right)
const visibleSessionIds = ref<number[]>([]);

const loading = ref(true);
const error = ref<string | null>(null);

// Cache for images: images[sessionId][type] = base64String
const imageCache = ref<Record<number, Record<string, string>>>({});

const showCropper = ref(false);
const showImageModal = ref(false);
const showComparisonModal = ref(false);
const activeCropSessionId = ref<number | null>(null);
const activeCropType = ref<string | null>(null);
const activeComparisonType = ref<string | null>(null);

// Helper refs for legacy modal compatibility (derived from visible/selected)
// Actually the modal now uses custom props mostly, but `getParsedCrop` needs updating
const session1 = computed(() => visibleSessions.value[0] || null);
const session2 = computed(() => visibleSessions.value[1] || null);

// Selection Mode State
const isSelectMode = ref(false);
const selectedImages = ref<{
    key: string;
    sessionId: number;
    type: string;
    src: string;
    crop: any;
    label: string;
}[]>([]);

const selectedKeys = computed(() => selectedImages.value.map(i => i.key));

function toggleSelectMode() {
    isSelectMode.value = !isSelectMode.value;
    if (!isSelectMode.value) {
        selectedImages.value = [];
    }
}

function handleSelectImage(sessionId: number, type: string) {
    if (!isSelectMode.value) return;

    const key = `${sessionId}-${type}`;
    const existingIndex = selectedImages.value.findIndex(i => i.key === key);

    if (existingIndex !== -1) {
        // Deselect if already selected
        selectedImages.value.splice(existingIndex, 1);
        return;
    }

    const session = allSessions.value.find(s => s.id === sessionId);
    const src = imageCache.value[sessionId]?.[type];
    
    if (!session || !src) return;

    const cropKey = `${type}_crop` as keyof Session;
    let crop = null;
    try {
        const cropStr = session[cropKey];
        if (typeof cropStr === 'string') crop = JSON.parse(cropStr);
    } catch {}

    const newItem = {
        key,
        sessionId,
        type,
        src,
        crop,
        label: `Session #${session.session_number} - ${type.replace('_', ' ')}`
    };

    // Add to list (FIFO max 2)
    if (selectedImages.value.length >= 2) {
        selectedImages.value.shift();
    }
    selectedImages.value.push(newItem);
}

// Custom Comparison State to override the default row-based comparison
const customComparison = ref<{
    img1: { src: string, crop: any, label: string } | null,
    img2: { src: string, crop: any, label: string } | null,
} | null>(null);

function handleCompareSelected() {
    if (selectedImages.value.length !== 2) return;
    customComparison.value = {
        img1: { 
            src: selectedImages.value[0].src, 
            crop: selectedImages.value[0].crop, 
            label: selectedImages.value[0].label 
        },
        img2: { 
            src: selectedImages.value[1].src, 
            crop: selectedImages.value[1].crop, 
            label: selectedImages.value[1].label 
        }
    };
    showComparisonModal.value = true;
}

function exitSelectMode() {
    isSelectMode.value = false;
    selectedImages.value = [];
    customComparison.value = null;
}

// Computed visible sessions objects
const visibleSessions = computed(() => {
  return visibleSessionIds.value
    .map(id => allSessions.value.find(s => s.id === id))
    .filter((s): s is Session => !!s);
});

async function loadImage(sessionId: number, type: string, path: string | null) {
  if (!path) return;
  if (imageCache.value[sessionId]?.[type]) return;

  try {
    const result = await commands.readImageBase64(path);
    if (result.status === "ok") {
      if (!imageCache.value[sessionId]) imageCache.value[sessionId] = {};
      imageCache.value[sessionId][type] = result.data;
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

async function addSessionToView(session: Session, position: 'start' | 'end') {
    if (position === 'start') {
        visibleSessionIds.value.unshift(session.id);
    } else {
        visibleSessionIds.value.push(session.id);
    }
    await loadSessionImages(session);
}

function addOlder() {
    if (visibleSessions.value.length === 0) return;
    const oldestVisible = visibleSessions.value[0];
    // Find candidate: Closest session number < oldestVisible
    const candidates = allSessions.value.filter(s => s.session_number < oldestVisible.session_number);
    // Sort by session number desc (closest to oldestVisible)
    candidates.sort((a, b) => b.session_number - a.session_number);
    
    if (candidates.length > 0) {
        addSessionToView(candidates[0], 'start');
    }
}

function addNewer() {
    if (visibleSessions.value.length === 0) return;
    const newestVisible = visibleSessions.value[visibleSessions.value.length - 1];
    // Find candidate: Closest session number > newestVisible
    const candidates = allSessions.value.filter(s => s.session_number > newestVisible.session_number);
    // Sort by session number asc (closest to newestVisible)
    candidates.sort((a, b) => a.session_number - b.session_number);
    
    if (candidates.length > 0) {
        addSessionToView(candidates[0], 'end');
    }
}

// Disable logic
const canAddOlder = computed(() => {
    if (visibleSessions.value.length === 0) return false;
    const oldestVisible = visibleSessions.value[0];
    return allSessions.value.some(s => s.session_number < oldestVisible.session_number);
});

const canAddNewer = computed(() => {
    if (visibleSessions.value.length === 0) return false;
    const newestVisible = visibleSessions.value[visibleSessions.value.length - 1];
    return allSessions.value.some(s => s.session_number > newestVisible.session_number);
});

async function handleUpdateSession(index: number, newSession: Session) {
    visibleSessionIds.value[index] = newSession.id;
    await loadSessionImages(newSession);
}

function handleRemoveSession(index: number) {
    if (visibleSessionIds.value.length > 1) {
        visibleSessionIds.value.splice(index, 1);
    }
}

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

// Crop Handling
const activeInitialCrop = computed(() => {
  if (!activeCropSessionId.value || !activeCropType.value) return null;
  const session = allSessions.value.find(s => s.id === activeCropSessionId.value);
  if (!session) return null;
  
  const key = `${activeCropType.value}_crop` as keyof Session;
  const cropString = session[key];
  if (typeof cropString === 'string') {
    try {
      return JSON.parse(cropString);
    } catch {
        return null;
    }
  }
  return null;
});

function openImageModal(sessionId: number, type: string) {
    // Only open if image exists
    if (imageCache.value[sessionId]?.[type]) {
        activeCropSessionId.value = sessionId;
        activeCropType.value = type;
        showImageModal.value = true;
    }
}

function handleOpenCropper(sessionId: number, type: string) {
     if (imageCache.value[sessionId]?.[type]) {
        activeCropSessionId.value = sessionId;
        activeCropType.value = type;
        showCropper.value = true;
    }
}

function handleCropFromModal() {
    showImageModal.value = false;
    // Wait a bit? No need.
    showCropper.value = true;
}

async function handleCropSave(cropData: { x: number, y: number, width: number }) {
  if (!activeCropSessionId.value || !activeCropType.value) return;

  const cropString = JSON.stringify(cropData);
  const sessionId = activeCropSessionId.value;
  const type = activeCropType.value;

  try {
    const result = await commands.updateSessionCrop(sessionId, type, cropString);
    if (result.status === "ok") {
      // Update local session state
      const sessionIndex = allSessions.value.findIndex(s => s.id === sessionId);
      if (sessionIndex !== -1) {
        (allSessions.value[sessionIndex] as any)[`${type}_crop`] = cropString;
      }
    }
  } catch (e) {
    console.error("Error saving crop", e);
  }
}

const imageTypes = [
  { key: 'anterior', label: 'Anterior' },
  { key: 'posterior', label: 'Posterior' },
  { key: 'right_lateral', label: 'Right Lateral' },
  { key: 'left_lateral', label: 'Left Lateral' },
];

onMounted(async () => {
  try {
    const result = await commands.getClientSessions(clientId);
    if (result.status === "ok") {
      allSessions.value = result.data;
      // Sort all sessions by session_number ascending for easier logic?
      // Actually the backend sends DESC. Let's keep that in mind or sort it locally to be consistent.
      // Let's sort ASC locally for "Timeline" logic (Left=Old -> Right=New)
      // Actually `CompareSessionColumn` expects them sorted? 
      // The sort in `CompareSessionColumn` handles it.

      // Init visible sessions
      const initialList = initialSessionIds
        .map(id => allSessions.value.find(s => s.id === id))
        .filter((s): s is Session => !!s);
      
      // Ensure sorted by session_number ASC
      initialList.sort((a, b) => a.session_number - b.session_number);

      for (const s of initialList) {
          await addSessionToView(s, 'end');
      }

      if (initialList.length === 0 && allSessions.value.length > 0) {
          // Fallback: load newest session if nothing valid passed
          await addSessionToView(allSessions.value[0], 'end');
      }

    } else {
      error.value = result.error;
    }
  } catch (e: any) {
    error.value = e.message || "An unknown error occurred";
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div 
    class="h-screen flex flex-col"
    :style="isSelectMode 
      ? { 
          backgroundImage: 'radial-gradient(rgb(0, 0, 0) 1px, transparent 1px)', 
          backgroundSize: '20px 20px',
          backgroundColor: 'rgb(243, 244, 246)' // Tailwind gray-100 color
        } 
      : { backgroundColor: 'rgb(249, 250, 251)' } // Tailwind gray-50 color
    "
  >
    <!-- Header -->
    <div class="bg-white shadow-sm border-b border-gray-200 p-4 flex items-center z-30 flex-shrink-0">
      <button 
        @click="router.push({ name: 'client-sessions', params: { id: clientId } })" 
        class="mr-4 p-2 rounded-full hover:bg-gray-100 transition-colors"
      >
        <ArrowLeft class="h-6 w-6 text-gray-600" />
      </button>
      <h1 class="text-2xl font-bold text-gray-900">Compare Sessions</h1>
    </div>

    <!-- Main Content Area -->
    <div class="flex-1 overflow-hidden relative flex items-center">
        <div v-if="loading" class="w-full text-center text-gray-500">Loading...</div>
        <div v-else-if="error" class="w-full text-center text-red-500">{{ error }}</div>
        
        <div v-else class="w-full h-full overflow-auto">
            <div class="min-w-full min-h-full w-fit flex justify-center items-start p-8 gap-6">
            
            <!-- Add Older Button -->
            <div class="h-full flex items-center flex-shrink-0 sticky left-0 z-10 top-1/2 -translate-y-1/2">
                 <button 
                    @click="addOlder"
                    :disabled="!canAddOlder"
                    class="h-12 w-12 rounded-full bg-white border border-gray-200 shadow-sm flex items-center justify-center text-gray-600 hover:text-blue-600 hover:border-blue-300 transition-all disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:border-gray-200 disabled:hover:text-gray-600"
                    title="Add Previous Session"
                >
                    <Plus class="h-6 w-6" />
                </button>
            </div>

            <!-- Columns -->
            <CompareSessionColumn 
                v-for="(session, index) in visibleSessions" 
                :key="session.id"
                :session="session"
                :all-sessions="allSessions"
                :older-limit="index > 0 ? visibleSessions[index - 1].session_number : null"
                :newer-limit="index < visibleSessions.length - 1 ? visibleSessions[index + 1].session_number : null"
                :images="imageCache[session.id] || {}"
                :can-remove="visibleSessions.length > 1"
                :is-select-mode="isSelectMode"
                :selected-keys="selectedKeys"
                @update:session="s => handleUpdateSession(index, s)"
                @view-image="type => openImageModal(session.id, type)"
                @open-cropper="type => handleOpenCropper(session.id, type)"
                @select-image="type => handleSelectImage(session.id, type)"
                @remove="handleRemoveSession(index)"
            />

            <!-- Add Newer Button -->
            <div class="h-full flex items-center flex-shrink-0 sticky right-0 z-10 top-1/2 -translate-y-1/2">
                <button 
                    @click="addNewer"
                    :disabled="!canAddNewer"
                    class="h-12 w-12 rounded-full bg-white border border-gray-200 shadow-sm flex items-center justify-center text-gray-600 hover:text-blue-600 hover:border-blue-300 transition-all disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:border-gray-200 disabled:hover:text-gray-600"
                    title="Add Next Session"
                >
                    <Plus class="h-6 w-6" />
                </button>
            </div>

            </div>
        </div>
    </div>

    <ImageCropper 
      :show="showCropper"
      :image-src="(activeCropSessionId && activeCropType) ? imageCache[activeCropSessionId]?.[activeCropType] : null"
      :initial-crop="activeInitialCrop"
      :image-type="activeCropType"
      @close="showCropper = false"
      @save="handleCropSave"
    />

    <ImageModal
        :show="showImageModal"
        :image-src="(activeCropSessionId && activeCropType) ? imageCache[activeCropSessionId]?.[activeCropType] : null"
        :crop="activeInitialCrop"
        :can-crop="true"
        @close="showImageModal = false"
        @crop="handleCropFromModal"
    />
    <ImageComparisonModal
      :show="showComparisonModal"
      :title="customComparison ? 'Custom Comparison' : (imageTypes.find(t => t.key === activeComparisonType)?.label || '')"
      :image1-src="customComparison ? (customComparison.img1?.src ?? null) : ((session1 && activeComparisonType) ? (imageCache[session1.id]?.[activeComparisonType] ?? null) : null)"
      :image2-src="customComparison ? (customComparison.img2?.src ?? null) : ((session2 && activeComparisonType) ? (imageCache[session2.id]?.[activeComparisonType] ?? null) : null)"
      :crop1="customComparison ? customComparison.img1?.crop : getParsedCrop(session1, activeComparisonType || '')"
      :crop2="customComparison ? customComparison.img2?.crop : getParsedCrop(session2, activeComparisonType || '')"
      :label1="customComparison ? customComparison.img1?.label : (session1 ? `Session #${session1.session_number}` : 'Before')"
      :label2="customComparison ? customComparison.img2?.label : (session2 ? `Session #${session2.session_number}` : 'After')"
      @close="() => { showComparisonModal = false; customComparison = null; exitSelectMode() }"
    />

    <!-- Floating Action Button for Select Mode -->
    <div class="fixed bottom-8 left-8 z-40">
        <button 
            @click="toggleSelectMode"
            class="h-14 w-14 rounded-full shadow-lg flex items-center justify-center transition-all duration-300 transform hover:scale-105 active:scale-95"
            :class="isSelectMode ? 'bg-blue-600 text-white' : 'bg-white text-gray-700 hover:bg-gray-50'"
            title="Toggle Selection Mode"
        >
            <X v-if="isSelectMode" class="h-6 w-6" />
            <CheckSquare v-else class="h-6 w-6" />
        </button>
    </div>

    <!-- Floating Selection Card -->
    <div v-if="isSelectMode" class="fixed bottom-8 left-1/2 -translate-x-1/2 z-40 bg-white rounded-xl shadow-xl border border-gray-200 p-3 flex items-center gap-4 animate-in slide-in-from-bottom-4 fade-in duration-300">
        <div class="flex gap-2">
            <div v-for="img in selectedImages" :key="img.key" class="relative h-12 w-12 rounded overflow-hidden border border-gray-200 group">
                <img :src="img.src" class="h-full w-full object-cover" />
                <div class="absolute inset-0 bg-black/20 group-hover:bg-black/0 transition-colors"></div>
            </div>
            <!-- Placeholders -->
            <div v-if="selectedImages.length < 2" class="h-12 w-12 rounded border-2 border-dashed border-gray-300 flex items-center justify-center text-gray-400 bg-gray-50">
                <span class="text-xs">{{ selectedImages.length + 1 }}</span>
            </div>
        </div>

        <div class="h-8 w-px bg-gray-200"></div>

        <div class="flex items-center gap-2">
            <button 
                @click="handleCompareSelected"
                :disabled="selectedImages.length !== 2"
                class="px-4 py-2 rounded-lg bg-blue-600 text-white text-sm font-medium hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            >
                <Check class="w-4 h-4" />
                Compare
            </button>
            <button 
                @click="exitSelectMode"
                class="px-3 py-2 rounded-lg hover:bg-gray-100 text-gray-600 text-sm font-medium transition-colors"
            >
                Cancel
            </button>
        </div>
    </div>
  </div>
</template>
