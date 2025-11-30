<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useClientStore } from "../stores/client";
import { storeToRefs } from "pinia";

const clientStore = useClientStore();
const { clients } = storeToRefs(clientStore);
const showAddModal = ref(false);
const newClient = ref({
  firstname: "",
  lastname: "",
  dob: "",
  sex: "M"
});

async function handleAddClient() {
  if (!newClient.value.firstname || !newClient.value.lastname) return;
  
  await clientStore.addClient(newClient.value);
  showAddModal.value = false;
  
  // Reset form
  newClient.value = {
    firstname: "",
    lastname: "",
    dob: "",
    sex: "M"
  };
}

function formatDate(dateString: string) {
  if (!dateString) return "-";
  return new Date(dateString).toLocaleDateString();
}

onMounted(async () => {
  await clientStore.loadClients();
});
</script>

<template>
  <div class="p-8 max-w-6xl mx-auto">
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Clients</h1>
      <button 
        @click="showAddModal = true"
        class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg shadow-sm transition-colors flex items-center gap-2"
      >
        <span class="text-xl font-bold">+</span> Add Client
      </button>
    </div>

    <!-- Data Table -->
    <div class="overflow-x-auto shadow-md sm:rounded-lg">
      <table class="w-full text-sm text-left text-gray-500 dark:text-gray-400">
        <thead class="text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400">
          <tr>
            <th scope="col" class="px-6 py-3">ID</th>
            <th scope="col" class="px-6 py-3">First Name</th>
            <th scope="col" class="px-6 py-3">Last Name</th>
            <th scope="col" class="px-6 py-3">Date of Birth</th>
            <th scope="col" class="px-6 py-3">Sex</th>
            <th scope="col" class="px-6 py-3">Registration Date</th>
            <th scope="col" class="px-6 py-3">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="clients.length === 0">
            <td colspan="7" class="px-6 py-4 text-center text-gray-500 dark:text-gray-400">
              No clients found.
            </td>
          </tr>
          <tr 
            v-for="client in clients" 
            :key="client.id" 
            class="bg-white border-b dark:bg-gray-800 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600"
          >
            <td class="px-6 py-4 font-medium text-gray-900 dark:text-white whitespace-nowrap">
              {{ client.id }}
            </td>
            <td class="px-6 py-4">{{ client.firstname }}</td>
            <td class="px-6 py-4">{{ client.lastname }}</td>
            <td class="px-6 py-4">{{ formatDate(client.dob) }}</td>
            <td class="px-6 py-4">{{ client.sex }}</td>
            <td class="px-6 py-4">{{ formatDate(client.registration_date) }}</td>
            <td class="px-6 py-4">
              <a href="#" class="font-medium text-blue-600 dark:text-blue-500 hover:underline">Edit</a>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Add Client Modal -->
    <div v-if="showAddModal" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-50 backdrop-blur-sm">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md overflow-hidden transform transition-all">
        <div class="px-6 py-4 border-b dark:border-gray-700">
          <h3 class="text-lg font-medium leading-6 text-gray-900 dark:text-white">Add New Client</h3>
        </div>
        <form @submit.prevent="handleAddClient" class="p-6 space-y-4">
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">First Name</label>
              <input 
                v-model="newClient.firstname" 
                required
                class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:border-gray-600 dark:text-white"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Last Name</label>
              <input 
                v-model="newClient.lastname" 
                required
                class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:border-gray-600 dark:text-white"
              />
            </div>
          </div>
          
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Date of Birth</label>
            <input 
              v-model="newClient.dob" 
              type="date"
              class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:border-gray-600 dark:text-white"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Sex</label>
            <select 
              v-model="newClient.sex"
              class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:border-gray-600 dark:text-white"
            >
              <option value="M">Male</option>
              <option value="F">Female</option>
              <option value="O">Other</option>
            </select>
          </div>

          <div class="mt-6 flex justify-end gap-3">
            <button 
              type="button" 
              @click="showAddModal = false"
              class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 dark:bg-gray-700 dark:text-gray-300 dark:border-gray-600 dark:hover:bg-gray-600"
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
  </div>
</template>
