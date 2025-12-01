<script setup lang="ts">
import { ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { commands, type CreateSessionDto } from "@/bindings";
import { ArrowLeft } from "lucide-vue-next";

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
      <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Add New Session for Client {{ clientId }}</h1>
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

      <div>
        <label for="anterior" class="block text-sm font-medium text-gray-300 mb-1">Anterior Image URL</label>
        <input 
          id="anterior"
          v-model="newSession.anterior" 
          type="text"
          class="w-full rounded-lg border border-gray-700 bg-gray-900 px-3 py-2 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-gray-200"
        />
      </div>
      <div>
        <label for="posterior" class="block text-sm font-medium text-gray-300 mb-1">Posterior Image URL</label>
        <input 
          id="posterior"
          v-model="newSession.posterior" 
          type="text"
          class="w-full rounded-lg border border-gray-700 bg-gray-900 px-3 py-2 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-gray-200"
        />
      </div>
      <div>
        <label for="right_lateral" class="block text-sm font-medium text-gray-300 mb-1">Right Lateral Image URL</label>
        <input 
          id="right_lateral"
          v-model="newSession.right_lateral" 
          type="text"
          class="w-full rounded-lg border border-gray-700 bg-gray-900 px-3 py-2 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-gray-200"
        />
      </div>
      <div>
        <label for="left_lateral" class="block text-sm font-medium text-gray-300 mb-1">Left Lateral Image URL</label>
        <input 
          id="left_lateral"
          v-model="newSession.left_lateral" 
          type="text"
          class="w-full rounded-lg border border-gray-700 bg-gray-900 px-3 py-2 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-gray-200"
        />
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
  </div>
</template>
