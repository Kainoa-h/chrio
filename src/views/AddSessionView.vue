<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed, watch } from "vue";
import { useRoute, useRouter, onBeforeRouteLeave } from "vue-router";
import { commands, type CreateSessionDto, type Client, type UpdateSessionDto} from "@/bindings";
import { ArrowLeft, Camera, Trash2 } from "lucide-vue-next";
import CameraModal from "@/components/CameraModal.vue";
import RatioImage from "@/components/RatioImage.vue";
import ImageCropper from "@/components/ImageCropper.vue";
import { useToast } from "@/composables/useToast";
import { getCurrentWindow } from "@tauri-apps/api/window";

const route = useRoute();
const router = useRouter();
const clientId = Number(route.params.id);
const sessionIdParam = route.params.sessionId ? Number(route.params.sessionId) : null;
// Mutable sessionId so we can capture the id when a new session is autosaved
const sessionIdRef = ref<number | null>(sessionIdParam);
const isEditing = computed(() => !!sessionIdRef.value);
const currentSessionNumber = ref<number | null>(null);

const newSession = ref<CreateSessionDto>({
  client_id: clientId,
  height: null,
  weight: null,
  anterior: null,
  posterior: null,
  right_lateral: null,
  left_lateral: null,
  notes: null,
  anterior_crop: null,
  posterior_crop: null,
  right_lateral_crop: null,
  left_lateral_crop: null,
});

const saving = ref(false);
const lastAutosaveAt = ref<number | null>(null);
const error = ref<string | null>(null);
const client = ref<Client | null>(null);
const nextSessionNumber = ref(1);

// Dirty state & navigation guard
const initialSnapshot = ref<string | null>(null);
const savedSuccessfully = ref(false);
const showLeaveConfirm = ref(false);
let pendingRoute: any = null;

const isDirty = computed(() => {
  if (initialSnapshot.value !== null) {
    // Editing mode (or after first autosave): compare against snapshot
    const current = JSON.stringify({
      session: newSession.value,
      hasImages: Object.fromEntries(
        Object.entries(imagePreviews.value).map(([k, v]) => [k, !!v])
      ),
    });
    return current !== initialSnapshot.value;
  }
  // New session mode, before any autosave: any image or any field filled
  const hasImage = Object.values(imagePreviews.value).some(v => !!v);
  const hasField = !!(newSession.value.height || newSession.value.weight || newSession.value.notes);
  return hasImage || hasField;
});

onBeforeRouteLeave((to) => {
  if (isDirty.value && !savedSuccessfully.value) {
    pendingRoute = to;
    showLeaveConfirm.value = true;
    return false;
  }
});

const { showToast: showToastMsg } = useToast();

function confirmLeave() {
  showLeaveConfirm.value = false;
  savedSuccessfully.value = true;
  showToastMsg('Changes discarded', 'warn');
  if (pendingRoute) {
    router.push(pendingRoute);
  }
}

const showCamera = ref(false);
const showCropper = ref(false);
const activeImageType = ref<string | null>(null);
const imagePreviews = ref<Record<string, string>>({
  anterior: "",
  posterior: "",
  right_lateral: "",
  left_lateral: "",
});
const cropData = ref<Record<string, { x: number, y: number, width: number } | null>>({
  anterior: null,
  posterior: null,
  right_lateral: null,
  left_lateral: null,
});

const imageTypes = ['anterior', 'posterior', 'right_lateral', 'left_lateral'] as const;

function captureSnapshot() {
  initialSnapshot.value = JSON.stringify({
    session: { ...newSession.value },
    hasImages: Object.fromEntries(
      Object.entries(imagePreviews.value).map(([k, v]) => [k, !!v])
    ),
  });
}

async function fetchClientAndSessionInfo() {
  try {
    const clientsResult = await commands.getClients();
    if (clientsResult.status === "ok") {
      client.value = clientsResult.data.find(c => c.id === clientId) || null;
    }

    if (isEditing.value && sessionIdRef.value) {
      const sessionResult = await commands.getSession(sessionIdRef.value);
      if (sessionResult.status === "ok") {
        const s = sessionResult.data;
        currentSessionNumber.value = s.session_number;
        
        newSession.value = {
            client_id: s.client_id,
            height: s.height,
            weight: s.weight,
            anterior: s.anterior,
            posterior: s.posterior,
            right_lateral: s.right_lateral,
            left_lateral: s.left_lateral,
            notes: s.notes,
            anterior_crop: s.anterior_crop,
            posterior_crop: s.posterior_crop,
            right_lateral_crop: s.right_lateral_crop,
            left_lateral_crop: s.left_lateral_crop,
        };

        for (const type of imageTypes) {
            const path = (s as any)[type];
            if (path) {
                const imgResult = await commands.readImageBase64(path);
                if (imgResult.status === "ok") {
                    imagePreviews.value[type] = imgResult.data;
                }
            }
            const crop = (s as any)[`${type}_crop`];
            if (crop) {
                try {
                    cropData.value[type] = JSON.parse(crop);
                } catch (e) {
                    console.error("Failed to parse crop data", e);
                }
            }
        }

        // Capture snapshot after loading so we can detect actual changes
        captureSnapshot();
      }
    } else {
      const nextSessionResult = await commands.getNextSessionNumber(clientId);
      if (nextSessionResult.status === "ok") {
          nextSessionNumber.value = nextSessionResult.data;
      }
    }
  } catch (e) {
    console.error("Failed to fetch client info", e);
  }
}

function openCamera(type: string) {
  activeImageType.value = type;
  showCamera.value = true;
}

function openCropper(type: string) {
  if (imagePreviews.value[type]) {
    activeImageType.value = type;
    showCropper.value = true;
  }
}

function handlePhotoTaken(photoData: string) {
  if (!activeImageType.value) return;
  // Only update the preview in memory
  imagePreviews.value[activeImageType.value] = photoData;
  // Reset crop when new photo is taken
  cropData.value[activeImageType.value] = null;
  // Autosave right away - photos are large operations, save them immediately
  autoSave();
}

function handleCropSave(data: { x: number, y: number, width: number }) {
  if (!activeImageType.value) return;
  cropData.value[activeImageType.value] = data;
  // Autosave - the image file is unchanged but crop metadata should persist
  autoSave();
}

function discardPhoto(type: string) {
  imagePreviews.value[type] = "";
  cropData.value[type] = null;
  (newSession.value as any)[type] = null;
  (newSession.value as any)[`${type}_crop`] = null;
  // Autosave the deletion
  autoSave();
}

// Core save routine used by both manual save and autosave
let pendingSave: { resolve: (ok: boolean) => void } | null = null;

async function performSave(): Promise<boolean> {
  if (!client.value) return false;

  if (saving.value) {
    return new Promise<boolean>((resolve) => {
      pendingSave = { resolve };
    });
  }

  saving.value = true;
  error.value = null;

  try {
    const sessionNo = sessionIdRef.value ? currentSessionNumber.value! : nextSessionNumber.value;

    // Save images first
    for (const type of imageTypes) {
      if (imagePreviews.value[type]) {
        const result = await commands.saveImage(
          clientId,
          client.value.firstname,
          sessionNo,
          type,
          imagePreviews.value[type]
        );

        if (result.status === "ok") {
          (newSession.value as any)[type] = result.data;
          // Save crop data if exists
          if (cropData.value[type]) {
             (newSession.value as any)[`${type}_crop`] = JSON.stringify(cropData.value[type]);
          }
        } else {
          throw new Error(`Failed to save ${type} image: ${result.error}`);
        }
      }
    }

    if (sessionIdRef.value) {
        const updateDto: UpdateSessionDto = {
            id: sessionIdRef.value,
            ...newSession.value
        };
        const result = await commands.updateSession(updateDto);
        if (result.status === "ok") {
          lastAutosaveAt.value = Date.now();
          // Refresh snapshot so isDirty becomes false until the next change
          captureSnapshot();
          return true;
        } else {
          error.value = result.error;
          return false;
        }
    } else {
        const result = await commands.addSession(newSession.value);
        if (result.status === "ok") {
          // Capture the newly-created session id so future saves update it
          sessionIdRef.value = result.data;
          currentSessionNumber.value = nextSessionNumber.value;
          lastAutosaveAt.value = Date.now();
          captureSnapshot();
          return true;
        } else {
          error.value = result.error;
          return false;
        }
    }
  } catch (e: any) {
    error.value = e.message || "An unknown error occurred";
    return false;
  } finally {
    saving.value = false;
    if (pendingSave) {
      const p = pendingSave;
      pendingSave = null;
      performSave().then(p.resolve);
    }
  }
}

let autosaveTimer: ReturnType<typeof setTimeout> | null = null;
function autoSave() {
  if (autosaveTimer) clearTimeout(autosaveTimer);
  autosaveTimer = setTimeout(async () => {
    autosaveTimer = null;
    // Don't autosave if there's nothing meaningful to save yet
    if (!isDirty.value) return;
    const ok = await performSave();
    if (!ok && error.value) {
      showToastMsg(`Autosave failed: ${error.value}`, 'error');
    }
  }, 600);
}

// Watch notes + numeric fields and debounce-autosave
watch(
  () => [newSession.value.notes, newSession.value.height, newSession.value.weight],
  () => {
    if (!client.value) return;
    autoSave();
  }
);

async function handleAddSession() {
  if (autosaveTimer) {
    clearTimeout(autosaveTimer);
    autosaveTimer = null;
  }
  const ok = await performSave();
  if (ok) {
    showToastMsg('Session saved!', 'success');
    savedSuccessfully.value = true;
    router.push({ name: 'client-sessions', params: { id: clientId } });
  } else if (error.value) {
    showToastMsg(`Error: ${error.value}`, 'error');
  }
}

// --- Window close-requested guard ---
let unlistenClose: (() => void) | null = null;
const showCloseConfirm = ref(false);

async function installCloseGuard() {
  try {
    const win = getCurrentWindow();
    unlistenClose = await win.onCloseRequested(async (event) => {
      if (savedSuccessfully.value || !isDirty.value) {
        return; // allow close
      }
      event.preventDefault();
      showCloseConfirm.value = true;
    });
  } catch (e) {
    console.error("Failed to install close guard", e);
  }
}

async function saveAndClose() {
  if (autosaveTimer) {
    clearTimeout(autosaveTimer);
    autosaveTimer = null;
  }
  const ok = await performSave();
  showCloseConfirm.value = false;
  if (ok) {
    showToastMsg('Session saved!', 'success');
    savedSuccessfully.value = true;
    // Re-emit the close event so the window closes
    try {
      await getCurrentWindow().close();
    } catch (e) {
      console.error("Failed to close window", e);
    }
  } else if (error.value) {
    showToastMsg(`Error: ${error.value}`, 'error');
  }
}

async function discardAndClose() {
  showCloseConfirm.value = false;
  savedSuccessfully.value = true;
  try {
    await getCurrentWindow().close();
  } catch (e) {
    console.error("Failed to close window", e);
  }
}

function cancelClose() {
  showCloseConfirm.value = false;
}

onMounted(() => {
  fetchClientAndSessionInfo();
  installCloseGuard();
});

onBeforeUnmount(() => {
  if (unlistenClose) {
    unlistenClose();
    unlistenClose = null;
  }
  if (autosaveTimer) {
    clearTimeout(autosaveTimer);
    autosaveTimer = null;
  }
});
</script>

<template>
  <!-- Leave confirmation dialog -->
  <div v-if="showLeaveConfirm" class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center">
    <div class="bg-white rounded-xl shadow-xl p-6 max-w-sm w-full mx-4 space-y-4">
      <h2 class="text-lg font-semibold text-gray-900">Unsaved Changes</h2>
      <p class="text-sm text-gray-600">You have unsaved changes. Would you like to save before leaving?</p>
      <div class="flex flex-col gap-2">
        <button
          @click="() => { showLeaveConfirm = false; handleAddSession(); }"
          class="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
        >
          Save Session
        </button>
        <button
          @click="confirmLeave"
          class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
        >
          Discard Changes
        </button>
        <button
          @click="showLeaveConfirm = false"
          class="px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700 transition-colors"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>

  <!-- Window close confirmation dialog -->
  <div v-if="showCloseConfirm" class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center">
    <div class="bg-white rounded-xl shadow-xl p-6 max-w-sm w-full mx-4 space-y-4">
      <h2 class="text-lg font-semibold text-gray-900">Unsaved Changes</h2>
      <p class="text-sm text-gray-600">You have unsaved changes. What would you like to do before closing?</p>
      <div class="flex flex-col gap-2">
        <button
          @click="saveAndClose"
          class="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
        >
          Save &amp; Close
        </button>
        <button
          @click="discardAndClose"
          class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
        >
          Discard &amp; Close
        </button>
        <button
          @click="cancelClose"
          class="px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700 transition-colors"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>

  <div class="p-8 max-w-6xl mx-auto">
    <div class="flex items-center mb-6">
      <button 
        @click="router.push({ name: 'client-sessions', params: { id: clientId } })" 
        class="mr-4 p-2 rounded-full hover:bg-gray-100 transition-colors"
      >
        <ArrowLeft class="h-6 w-6 text-gray-600" />
      </button>
      <h1 class="text-3xl font-bold text-gray-900">{{ isEditing ? 'Edit Session #' + currentSessionNumber : 'Add New Session' }} for {{ client?.firstname || 'Client' }}</h1>
      <span class="ml-4 text-xs text-gray-500 self-center" v-if="saving">Saving…</span>
      <span class="ml-4 text-xs text-gray-400 self-center" v-else-if="lastAutosaveAt">All changes saved</span>
    </div>

    <form @submit.prevent="handleAddSession" class="bg-white p-6 rounded-md shadow-sm space-y-4 border border-gray-200">

      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Anterior -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">Anterior</label>
          <RatioImage 
            :src="imagePreviews.anterior" 
            :crop="cropData.anterior"
            empty-text="No photo" 
            container-class="group w-full cursor-pointer"
            @click="openCropper('anterior')"
          >
            <template #overlay>
              <div v-if="imagePreviews.anterior" class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity z-10 flex gap-2">
                 <button 
                  type="button"
                  @click.stop="openCamera('anterior')"
                  class="p-2 bg-gray-800 bg-opacity-75 rounded-full hover:bg-gray-700 text-white"
                  title="Retake Photo"
                >
                  <Camera class="h-5 w-5" />
                </button>
                 <button 
                  type="button"
                  @click.stop="discardPhoto('anterior')"
                  class="p-2 bg-red-800 bg-opacity-75 rounded-full hover:bg-red-700 text-white"
                  title="Discard Photo"
                >
                  <Trash2 class="h-5 w-5" />
                </button>
              </div>
              <div v-else class="absolute inset-0 flex items-center justify-center">
                 <button 
                  type="button"
                  @click.stop="openCamera('anterior')"
                  class="px-4 py-2 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 text-gray-700 flex items-center gap-2 shadow-sm"
                >
                  <Camera class="h-5 w-5" /> Take Photo
                </button>
              </div>
            </template>
          </RatioImage>
        </div>

        <!-- Posterior -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">Posterior</label>
          <RatioImage 
            :src="imagePreviews.posterior" 
            :crop="cropData.posterior"
            empty-text="No photo" 
            container-class="group w-full cursor-pointer"
            @click="openCropper('posterior')"
          >
            <template #overlay>
              <div v-if="imagePreviews.posterior" class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity z-10 flex gap-2">
                 <button 
                  type="button"
                  @click.stop="openCamera('posterior')"
                  class="p-2 bg-gray-800 bg-opacity-75 rounded-full hover:bg-gray-700 text-white"
                  title="Retake Photo"
                >
                  <Camera class="h-5 w-5" />
                </button>
                 <button 
                  type="button"
                  @click.stop="discardPhoto('posterior')"
                  class="p-2 bg-red-800 bg-opacity-75 rounded-full hover:bg-red-700 text-white"
                  title="Discard Photo"
                >
                  <Trash2 class="h-5 w-5" />
                </button>
              </div>
              <div v-else class="absolute inset-0 flex items-center justify-center">
                 <button 
                  type="button"
                  @click.stop="openCamera('posterior')"
                  class="px-4 py-2 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 text-gray-700 flex items-center gap-2 shadow-sm"
                >
                  <Camera class="h-5 w-5" /> Take Photo
                </button>
              </div>
            </template>
          </RatioImage>
        </div>

        <!-- Right Lateral -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">Right Lateral</label>
          <RatioImage 
            :src="imagePreviews.right_lateral" 
            :crop="cropData.right_lateral"
            empty-text="No photo" 
            container-class="group w-full cursor-pointer"
            @click="openCropper('right_lateral')"
          >
            <template #overlay>
              <div v-if="imagePreviews.right_lateral" class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity z-10 flex gap-2">
                 <button 
                  type="button"
                  @click.stop="openCamera('right_lateral')"
                  class="p-2 bg-gray-800 bg-opacity-75 rounded-full hover:bg-gray-700 text-white"
                  title="Retake Photo"
                >
                  <Camera class="h-5 w-5" />
                </button>
                 <button 
                  type="button"
                  @click.stop="discardPhoto('right_lateral')"
                  class="p-2 bg-red-800 bg-opacity-75 rounded-full hover:bg-red-700 text-white"
                  title="Discard Photo"
                >
                  <Trash2 class="h-5 w-5" />
                </button>
              </div>
              <div v-else class="absolute inset-0 flex items-center justify-center">
                 <button 
                  type="button"
                  @click.stop="openCamera('right_lateral')"
                  class="px-4 py-2 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 text-gray-700 flex items-center gap-2 shadow-sm"
                >
                  <Camera class="h-5 w-5" /> Take Photo
                </button>
              </div>
            </template>
          </RatioImage>
        </div>

        <!-- Left Lateral -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">Left Lateral</label>
          <RatioImage 
            :src="imagePreviews.left_lateral" 
            :crop="cropData.left_lateral"
            empty-text="No photo" 
            container-class="group w-full cursor-pointer"
            @click="openCropper('left_lateral')"
          >
            <template #overlay>
              <div v-if="imagePreviews.left_lateral" class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity z-10 flex gap-2">
                 <button 
                  type="button"
                  @click.stop="openCamera('left_lateral')"
                  class="p-2 bg-gray-800 bg-opacity-75 rounded-full hover:bg-gray-700 text-white"
                  title="Retake Photo"
                >
                  <Camera class="h-5 w-5" />
                </button>
                 <button 
                  type="button"
                  @click.stop="discardPhoto('left_lateral')"
                  class="p-2 bg-red-800 bg-opacity-75 rounded-full hover:bg-red-700 text-white"
                  title="Discard Photo"
                >
                  <Trash2 class="h-5 w-5" />
                </button>
              </div>
              <div v-else class="absolute inset-0 flex items-center justify-center">
                 <button 
                  type="button"
                  @click.stop="openCamera('left_lateral')"
                  class="px-4 py-2 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 text-gray-700 flex items-center gap-2 shadow-sm"
                >
                  <Camera class="h-5 w-5" /> Take Photo
                </button>
              </div>
            </template>
          </RatioImage>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label for="height" class="block text-sm font-medium text-gray-700 mb-1">Height (cm)</label>
          <input 
            id="height"
            v-model.number="newSession.height" 
            type="number"
            step="0.01"
            class="w-full rounded-lg border border-gray-300 bg-white px-3 py-2 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-gray-900 shadow-sm"
          />
        </div>
        <div>
          <label for="weight" class="block text-sm font-medium text-gray-700 mb-1">Weight (kg)</label>
          <input 
            id="weight"
            v-model.number="newSession.weight" 
            type="number"
            step="0.01"
            class="w-full rounded-lg border border-gray-300 bg-white px-3 py-2 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-gray-900 shadow-sm"
          />
        </div>
      </div>

      <div>
        <label for="notes" class="block text-sm font-medium text-gray-700 mb-1">Notes</label>
        <textarea 
          id="notes"
          v-model="newSession.notes" 
          rows="4"
          class="w-full rounded-lg border border-gray-300 bg-white px-3 py-2 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-gray-900 shadow-sm"
        ></textarea>
      </div>

      <div v-if="error" class="text-red-500 text-sm">
        Error: {{ error }}
      </div>

      <div class="mt-6 flex justify-end gap-3">
        <button 
          type="button" 
          @click="router.push({ name: 'client-sessions', params: { id: clientId } })"
          class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors shadow-sm"
        >
          Cancel
        </button>
        <button 
          type="submit"
          :disabled="saving"
          class="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ saving ? 'Saving...' : (isEditing ? 'Update Session' : 'Save Session') }}
        </button>
      </div>
    </form>

    <CameraModal 
      :show="showCamera" 
      @close="showCamera = false"
      @photo-taken="handlePhotoTaken"
    />

    <ImageCropper 
      :show="showCropper"
      :image-src="activeImageType ? imagePreviews[activeImageType] : null"
      :initial-crop="activeImageType ? cropData[activeImageType] : null"
      :image-type="activeImageType"
      @close="showCropper = false"
      @save="handleCropSave"
    />
  </div>
</template>
