<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { commands, type CreateSessionDto, type Client, type Session } from "@/bindings";
import { ArrowLeft, Camera } from "lucide-vue-next";
import CameraModal from "@/components/CameraModal.vue";
import RatioImage from "@/components/RatioImage.vue";
import ImageCropper from "@/components/ImageCropper.vue";

const route = useRoute();
const router = useRouter();
const clientId = Number(route.params.id);

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
const error = ref<string | null>(null);
const client = ref<Client | null>(null);
const nextSessionNumber = ref(1);

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

async function fetchClientAndSessionInfo() {
  try {
    const clientsResult = await commands.getClients();
    if (clientsResult.status === "ok") {
      client.value = clientsResult.data.find(c => c.id === clientId) || null;
    }

    const nextSessionResult = await commands.getNextSessionNumber(clientId);
    if (nextSessionResult.status === "ok") {
        nextSessionNumber.value = nextSessionResult.data;
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
}

function handleCropSave(data: { x: number, y: number, width: number }) {
  if (!activeImageType.value) return;
  cropData.value[activeImageType.value] = data;
}

async function handleAddSession() {
  if (!client.value) return;
  saving.value = true;
  error.value = null;

  try {
    // Save images first
    const imageTypes = ['anterior', 'posterior', 'right_lateral', 'left_lateral'];
    for (const type of imageTypes) {
      if (imagePreviews.value[type]) {
        const result = await commands.saveImage(
          clientId,
          client.value.firstname,
          nextSessionNumber.value,
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

    // Save session to DB
    const result = await commands.addSession(newSession.value);
    if (result.status === "ok") {
      router.push({ name: 'client-sessions', params: { id: clientId } });
    } else {
      error.value = result.error;
    }
  } catch (e: any) {
    error.value = e.message || "An unknown error occurred";
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  fetchClientAndSessionInfo();
});
</script>

<template>
  <div class="p-8 max-w-6xl mx-auto">
    <div class="flex items-center mb-6">
      <button 
        @click="router.push({ name: 'client-sessions', params: { id: clientId } })" 
        class="mr-4 p-2 rounded-full hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
      >
        <ArrowLeft class="h-6 w-6 text-gray-600 dark:text-gray-300" />
      </button>
      <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Add New Session for {{ client?.firstname || 'Client' }}</h1>
    </div>

    <form @submit.prevent="handleAddSession" class="bg-gray-950 p-6 rounded-md shadow-sm space-y-4 border border-gray-700">

      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Anterior -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Anterior</label>
          <RatioImage 
            :src="imagePreviews.anterior" 
            :crop="cropData.anterior"
            empty-text="No photo" 
            container-class="group w-full cursor-pointer"
            @click="openCropper('anterior')"
          >
            <template #overlay>
              <div v-if="imagePreviews.anterior" class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity z-10">
                 <button 
                  type="button"
                  @click.stop="openCamera('anterior')"
                  class="p-2 bg-gray-800 bg-opacity-75 rounded-full hover:bg-gray-700 text-white"
                  title="Retake Photo"
                >
                  <Camera class="h-5 w-5" />
                </button>
              </div>
              <div v-else class="absolute inset-0 flex items-center justify-center">
                 <button 
                  type="button"
                  @click.stop="openCamera('anterior')"
                  class="px-4 py-2 bg-gray-800 border border-gray-600 rounded-lg hover:bg-gray-700 text-gray-300 flex items-center gap-2"
                >
                  <Camera class="h-5 w-5" /> Take Photo
                </button>
              </div>
            </template>
          </RatioImage>
        </div>

        <!-- Posterior -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Posterior</label>
          <RatioImage 
            :src="imagePreviews.posterior" 
            :crop="cropData.posterior"
            empty-text="No photo" 
            container-class="group w-full cursor-pointer"
            @click="openCropper('posterior')"
          >
            <template #overlay>
              <div v-if="imagePreviews.posterior" class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity z-10">
                 <button 
                  type="button"
                  @click.stop="openCamera('posterior')"
                  class="p-2 bg-gray-800 bg-opacity-75 rounded-full hover:bg-gray-700 text-white"
                  title="Retake Photo"
                >
                  <Camera class="h-5 w-5" />
                </button>
              </div>
              <div v-else class="absolute inset-0 flex items-center justify-center">
                 <button 
                  type="button"
                  @click.stop="openCamera('posterior')"
                  class="px-4 py-2 bg-gray-800 border border-gray-600 rounded-lg hover:bg-gray-700 text-gray-300 flex items-center gap-2"
                >
                  <Camera class="h-5 w-5" /> Take Photo
                </button>
              </div>
            </template>
          </RatioImage>
        </div>

        <!-- Right Lateral -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Right Lateral</label>
          <RatioImage 
            :src="imagePreviews.right_lateral" 
            :crop="cropData.right_lateral"
            empty-text="No photo" 
            container-class="group w-full cursor-pointer"
            @click="openCropper('right_lateral')"
          >
            <template #overlay>
              <div v-if="imagePreviews.right_lateral" class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity z-10">
                 <button 
                  type="button"
                  @click.stop="openCamera('right_lateral')"
                  class="p-2 bg-gray-800 bg-opacity-75 rounded-full hover:bg-gray-700 text-white"
                  title="Retake Photo"
                >
                  <Camera class="h-5 w-5" />
                </button>
              </div>
              <div v-else class="absolute inset-0 flex items-center justify-center">
                 <button 
                  type="button"
                  @click.stop="openCamera('right_lateral')"
                  class="px-4 py-2 bg-gray-800 border border-gray-600 rounded-lg hover:bg-gray-700 text-gray-300 flex items-center gap-2"
                >
                  <Camera class="h-5 w-5" /> Take Photo
                </button>
              </div>
            </template>
          </RatioImage>
        </div>

        <!-- Left Lateral -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Left Lateral</label>
          <RatioImage 
            :src="imagePreviews.left_lateral" 
            :crop="cropData.left_lateral"
            empty-text="No photo" 
            container-class="group w-full cursor-pointer"
            @click="openCropper('left_lateral')"
          >
            <template #overlay>
              <div v-if="imagePreviews.left_lateral" class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity z-10">
                 <button 
                  type="button"
                  @click.stop="openCamera('left_lateral')"
                  class="p-2 bg-gray-800 bg-opacity-75 rounded-full hover:bg-gray-700 text-white"
                  title="Retake Photo"
                >
                  <Camera class="h-5 w-5" />
                </button>
              </div>
              <div v-else class="absolute inset-0 flex items-center justify-center">
                 <button 
                  type="button"
                  @click.stop="openCamera('left_lateral')"
                  class="px-4 py-2 bg-gray-800 border border-gray-600 rounded-lg hover:bg-gray-700 text-gray-300 flex items-center gap-2"
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
          <label for="height" class="block text-sm font-medium text-gray-300 mb-1">Height (cm)</label>
          <input 
            id="height"
            v-model.number="newSession.height" 
            type="number"
            step="0.01"
            class="w-full rounded-lg border border-gray-700 bg-gray-900 px-3 py-2 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-gray-200"
          />
        </div>
        <div>
          <label for="weight" class="block text-sm font-medium text-gray-300 mb-1">Weight (kg)</label>
          <input 
            id="weight"
            v-model.number="newSession.weight" 
            type="number"
            step="0.01"
            class="w-full rounded-lg border border-gray-700 bg-gray-900 px-3 py-2 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-gray-200"
          />
        </div>
      </div>

      <div>
        <label for="notes" class="block text-sm font-medium text-gray-300 mb-1">Notes</label>
        <textarea 
          id="notes"
          v-model="newSession.notes" 
          rows="4"
          class="w-full rounded-lg border border-gray-700 bg-gray-900 px-3 py-2 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-gray-200"
        ></textarea>
      </div>

      <div v-if="error" class="text-red-500 text-sm">
        Error: {{ error }}
      </div>

      <div class="mt-6 flex justify-end gap-3">
        <button 
          type="button" 
          @click="router.push({ name: 'client-sessions', params: { id: clientId } })"
          class="px-4 py-2 text-sm font-medium text-gray-300 bg-gray-800 border border-gray-700 rounded-lg hover:bg-gray-700 transition-colors"
        >
          Cancel
        </button>
        <button 
          type="submit"
          :disabled="saving"
          class="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ saving ? 'Saving...' : 'Save Session' }}
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
