<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useClientStore } from "@/stores/client";
import { storeToRefs } from "pinia";
import ClientTable from "@/components/ClientTable.vue";
import AddClientModal from "@/components/AddClientModal.vue";

const clientStore = useClientStore();
const { clients } = storeToRefs(clientStore);
const showAddModal = ref(false);

onMounted(async () => {
  await clientStore.loadClients();
});
</script>

<template>
  <div class="p-8 max-w-6xl mx-auto">
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-3xl font-bold text-gray-900">Clients</h1>
      <button 
        @click="showAddModal = true"
        class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg shadow-sm transition-colors flex items-center gap-2"
      >
        <span class="text-xl font-bold">+</span> Add Client
      </button>
    </div>

    <!-- Data Table -->
    <ClientTable :clients="clients" />

    <!-- Add Client Modal -->
    <AddClientModal v-if="showAddModal" @close="showAddModal = false" />
  </div>
</template>
