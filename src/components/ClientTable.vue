<script setup lang="ts">
import type { Client } from "../bindings";

defineProps<{
  clients: Client[];
}>();

function formatDate(dateString: string) {
  if (!dateString) return "-";
  return new Date(dateString).toLocaleDateString();
}
</script>

<template>
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
</template>
