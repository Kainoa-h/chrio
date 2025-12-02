<script setup lang="ts">
import { ref } from "vue";
import { useClientStore } from "../stores/client";

const emit = defineEmits<{
  (e: 'close'): void
}>();

const clientStore = useClientStore();
const newClient = ref({
  firstname: "",
  lastname: "",
  dob: "",
  sex: "M"
});

async function handleAddClient() {
  if (!newClient.value.firstname || !newClient.value.lastname) return;
  
  await clientStore.addClient(newClient.value);
  emit('close');
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-50 backdrop-blur-sm">
    <div class="bg-white rounded-lg shadow-xl w-full max-w-md overflow-hidden transform transition-all">
      <div class="px-6 py-4 border-b">
        <h3 class="text-lg font-medium leading-6 text-gray-900">Add New Client</h3>
      </div>
      <form @submit.prevent="handleAddClient" class="p-6 space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <div>
    <label class="block text-sm font-medium text-gray-700 mb-1">First Name</label>
    <input
      id="firstName"
      v-model="newClient.firstname"
      type="text"
      class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
    />
          </div>
          <div>
    <label class="block text-sm font-medium text-gray-700 mb-1">Last Name</label>
    <input
      id="lastName"
      v-model="newClient.lastname"
      type="text"
      class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
    />
          </div>
        </div>
        
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Date of Birth</label>
          <input 
            v-model="newClient.dob" 
            type="date"
            class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>
        
        <div>
    <label class="block text-sm font-medium text-gray-700 mb-1">Sex</label>
    <select
      id="sex"
      v-model="newClient.sex"
      class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
    >
      <option value="M">Male</option>
      <option value="F">Female</option>
      <option value="O">Other</option>
    </select>
        </div>

        <div class="mt-6 flex justify-end gap-3">
          <button 
            type="button" 
            @click="$emit('close')"
            class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50"
          >
            Cancel
          </button>
          <button 
            type="submit"
            class="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            Save Client
          </button>
        </div>
      </form>
    </div>
  </div>
</template>
