<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { useClientStore } from "../stores/client";
import type { Client } from "@/bindings";

const props = defineProps<{
  client?: Client;
}>();

const emit = defineEmits<{
  (e: 'close'): void
}>();

const clientStore = useClientStore();

const isEditing = computed(() => !!props.client);
const showConfirmChanges = ref(false); // New state for confirmation

const newClient = ref({
  firstname: "",
  lastname: "",
  dob: "",
  sex: "M"
});

// Initialize form with client data if in edit mode
watch(() => props.client, (newVal) => {
  if (newVal) {
    newClient.value = {
      firstname: newVal.firstname,
      lastname: newVal.lastname,
      dob: newVal.dob,
      sex: newVal.sex
    };
  } else {
    newClient.value = {
      firstname: "",
      lastname: "",
      dob: "",
      sex: "M"
    };
  }
  showConfirmChanges.value = false; // Reset confirmation state on client change
}, { immediate: true });

async function handleSubmit() {
  if (!newClient.value.firstname || !newClient.value.lastname) return;

  if (isEditing.value) {
    showConfirmChanges.value = true; // Show confirmation dialog
  } else {
    // Add new client
    await clientStore.addClient(newClient.value);
    emit('close');
  }
}

async function commitUpdate() {
  if (!newClient.value.firstname || !newClient.value.lastname || !props.client) return;
  await clientStore.updateClient({ ...newClient.value, id: props.client.id });
  emit('close');
}

function cancelConfirm() {
  showConfirmChanges.value = false;
}

function closeModal() {
  showConfirmChanges.value = false; // Ensure confirmation is reset
  emit('close');
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-50 backdrop-blur-sm" @click.self="closeModal">
    <div class="bg-white rounded-lg shadow-xl w-full max-w-md overflow-hidden transform transition-all">
      <div class="px-6 py-4 border-b">
        <h3 class="text-lg font-medium leading-6 text-gray-900">{{ isEditing ? 'Edit Client' : 'Add New Client' }}</h3>
      </div>
      <form @submit.prevent="handleSubmit" class="p-6 space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">First Name</label>
            <input
              id="firstName"
              v-model="newClient.firstname"
              type="text"
              class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              :disabled="showConfirmChanges"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Last Name</label>
            <input
              id="lastName"
              v-model="newClient.lastname"
              type="text"
              class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              :disabled="showConfirmChanges"
            />
          </div>
        </div>
        
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Date of Birth</label>
          <input 
            v-model="newClient.dob" 
            type="date"
            class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            :disabled="showConfirmChanges"
          />
        </div>
        
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Sex</label>
          <select
            id="sex"
            v-model="newClient.sex"
            class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            :disabled="showConfirmChanges"
          >
            <option value="M">Male</option>
            <option value="F">Female</option>
            <option value="O">Other</option>
          </select>
        </div>

        <div class="mt-6 flex justify-end gap-3">
          <button 
            type="button" 
            @click="closeModal"
            class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50"
            :disabled="showConfirmChanges"
            :class="{ 'opacity-50 cursor-not-allowed': showConfirmChanges }"
          >
            Cancel
          </button>
          <button 
            type="submit"
            class="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            :disabled="showConfirmChanges && isEditing"
            :class="{ 'opacity-50 cursor-not-allowed': showConfirmChanges && isEditing }"
          >
            {{ isEditing ? 'Update Client' : 'Save Client' }}
          </button>
        </div>
      </form>

      <div v-if="showConfirmChanges" class="p-6 bg-gray-50 border-t border-gray-200 text-center">
        <p class="text-sm text-gray-700 mb-4">Are you sure you want to update these client details?</p>
        <div class="flex justify-center gap-3">
          <button 
            type="button" 
            @click="cancelConfirm"
            class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50"
          >
            Go Back
          </button>
          <button 
            type="button" 
            @click="commitUpdate"
            class="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            Confirm Update
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
