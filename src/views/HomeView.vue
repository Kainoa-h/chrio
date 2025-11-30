<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { useGreetStore } from "../stores/greet";
import { storeToRefs } from "pinia";

const store = useGreetStore();
const { name, greetMsg } = storeToRefs(store);

async function greet() {
  store.greetMsg = await invoke("greet", { name: store.name });
}
</script>

<template>
  <div class="flex flex-col items-center justify-center pt-[10vh] text-center">
    <h1 class="text-4xl font-bold mb-8">Welcome to Tauri + Vue</h1>

    <div class="flex justify-center gap-8 mb-4">
      <a href="https://vite.dev" target="_blank" class="hover:drop-shadow-[0_0_2em_#747bff] transition-all duration-700">
        <img src="/vite.svg" class="h-24 p-6" alt="Vite logo" />
      </a>
      <a href="https://tauri.app" target="_blank" class="hover:drop-shadow-[0_0_2em_#24c8db] transition-all duration-700">
        <img src="/tauri.svg" class="h-24 p-6" alt="Tauri logo" />
      </a>
      <a href="https://vuejs.org/" target="_blank" class="hover:drop-shadow-[0_0_2em_#249b73] transition-all duration-700">
        <img src="../assets/vue.svg" class="h-24 p-6" alt="Vue logo" />
      </a>
    </div>
    <p class="mb-8">Click on the Tauri, Vite, and Vue logos to learn more.</p>

    <form class="flex justify-center gap-2 mb-4" @submit.prevent="greet">
      <input 
        id="greet-input" 
        v-model="name" 
        placeholder="Enter a name..." 
        class="rounded-lg border border-transparent px-5 py-2.5 text-base font-medium bg-white dark:bg-[#0f0f0f98] text-[#0f0f0f] dark:text-white shadow-md transition-colors duration-250 outline-none focus:border-[#396cd8]"
      />
      <button 
        type="submit"
        class="rounded-lg border border-transparent px-5 py-2.5 text-base font-medium bg-white dark:bg-[#0f0f0f98] text-[#0f0f0f] dark:text-white shadow-md cursor-pointer transition-colors duration-250 outline-none hover:border-[#396cd8] active:bg-gray-200 dark:active:bg-[#0f0f0f69]"
      >
        Greet
      </button>
    </form>
    <p>{{ greetMsg }}</p>
  </div>
</template>