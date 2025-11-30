<script setup lang="ts">
import { useGreetStore } from "../stores/greet";
import { storeToRefs } from "pinia";

const store = useGreetStore();
const { name, greetMsg } = storeToRefs(store);
</script>

<template>
  <div class="flex flex-col items-center pt-12">
    <h1 class="text-2xl font-bold mb-4">This is an about page</h1>
    <p class="mb-4">This application is built with Tauri and Vue 3.</p>
    <div v-if="name" class="p-4 bg-gray-200 dark:bg-gray-800 rounded-lg shadow-md">
      <p>Current user: <strong class="text-blue-600 dark:text-blue-400">{{ name }}</strong></p>
      <p>Last greeting: <em class="text-green-600 dark:text-green-400">{{ greetMsg }}</em></p>
    </div>
  </div>
</template>
