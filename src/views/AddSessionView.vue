<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { commands, type CreateSessionDto, type Client, type Session } from "@/bindings";
import { ArrowLeft, Camera } from "lucide-vue-next";
import CameraModal from "@/components/CameraModal.vue";

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
});

const saving = ref(false);
const error = ref<string | null>(null);
const client = ref<Client | null>(null);
const nextSessionNumber = ref(1);

const showCamera = ref(false);
const activeImageType = ref<string | null>(null);

async function fetchClientAndSessionInfo() {
  try {
    const clientsResult = await commands.getClients();
    if (clientsResult.status === "ok") {
      client.value = clientsResult.data.find(c => c.id === clientId) || null;
    }

    const sessionsResult = await commands.getClientSessions(clientId);
    if (sessionsResult.status === "ok") {
        const sessions = sessionsResult.data;
        const maxSession = sessions.reduce((max, s) => (s.session_number > max ? s.session_number : max), 0);
        nextSessionNumber.value = maxSession + 1;
    }
  } catch (e) {
    console.error("Failed to fetch client info", e);
  }
}

function openCamera(type: string) {
  activeImageType.value = type;
  showCamera.value = true;
}

async function handlePhotoTaken(photoData: string) {
  if (!activeImageType.value || !client.value) return;

  try {
    const result = await commands.saveImage(
      clientId,
      client.value.firstname,
      nextSessionNumber.value,
      activeImageType.value,
      photoData
    );

    if (result.status === "ok") {
      // Update the specific field with the file path
      if (activeImageType.value === 'anterior') newSession.value.anterior = result.data;
      if (activeImageType.value === 'posterior') newSession.value.posterior = result.data;
      if (activeImageType.value === 'right_lateral') newSession.value.right_lateral = result.data;
      if (activeImageType.value === 'left_lateral') newSession.value.left_lateral = result.data;
    } else {
      error.value = "Failed to save image: " + result.error;
    }
  } catch (e: any) {
    error.value = "Error saving image: " + e.message;
  }
}

async function handleAddSession() {
  saving.value = true;
  error.value = null;
  try {
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

      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Anterior -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Anterior</label>
          <div class="flex gap-2">
             <input 
              v-model="newSession.anterior" 
              type="text"
              placeholder="Image Path"
              class="flex-1 rounded-lg border border-gray-700 bg-gray-900 px-3 py-2 text-gray-200 text-sm"
              readonly
            />
            <button 
              type="button"
              @click="openCamera('anterior')"
              class="p-2 bg-gray-800 rounded-lg border border-gray-700 hover:bg-gray-700 text-gray-300"
            >
              <Camera class="h-5 w-5" />
            </button>
          </div>
        </div>

        <!-- Posterior -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Posterior</label>
          <div class="flex gap-2">
             <input 
              v-model="newSession.posterior" 
              type="text"
              placeholder="Image Path"
              class="flex-1 rounded-lg border border-gray-700 bg-gray-900 px-3 py-2 text-gray-200 text-sm"
              readonly
            />
            <button 
              type="button"
              @click="openCamera('posterior')"
              class="p-2 bg-gray-800 rounded-lg border border-gray-700 hover:bg-gray-700 text-gray-300"
            >
              <Camera class="h-5 w-5" />
            </button>
          </div>
        </div>

        <!-- Right Lateral -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Right Lateral</label>
          <div class="flex gap-2">
             <input 
              v-model="newSession.right_lateral" 
              type="text"
              placeholder="Image Path"
              class="flex-1 rounded-lg border border-gray-700 bg-gray-900 px-3 py-2 text-gray-200 text-sm"
              readonly
            />
            <button 
              type="button"
              @click="openCamera('right_lateral')"
              class="p-2 bg-gray-800 rounded-lg border border-gray-700 hover:bg-gray-700 text-gray-300"
            >
              <Camera class="h-5 w-5" />
            </button>
          </div>
        </div>

        <!-- Left Lateral -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Left Lateral</label>
          <div class="flex gap-2">
             <input 
              v-model="newSession.left_lateral" 
              type="text"
              placeholder="Image Path"
              class="flex-1 rounded-lg border border-gray-700 bg-gray-900 px-3 py-2 text-gray-200 text-sm"
              readonly
            />
            <button 
              type="button"
              @click="openCamera('left_lateral')"
              class="p-2 bg-gray-800 rounded-lg border border-gray-700 hover:bg-gray-700 text-gray-300"
            >
              <Camera class="h-5 w-5" />
            </button>
          </div>
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
  </div>
</template>
