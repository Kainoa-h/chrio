<script setup lang="ts">
import { X } from "lucide-vue-next";

defineEmits<{
  (e: 'close'): void
}>();
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-50 backdrop-blur-sm" @click.self="$emit('close')">
    <div class="bg-white rounded-lg shadow-xl w-full max-w-md overflow-hidden transform transition-all relative">
      <button 
        @click="$emit('close')" 
        class="absolute top-4 right-4 p-1 rounded-full hover:bg-gray-100 transition-colors"
      >
        <X class="w-5 h-5 text-gray-500" />
      </button>
      
      <div class="p-8">
        <h2 class="text-2xl font-bold text-gray-900 mb-6">About Chrio</h2>
        
        <div class="space-y-4 text-gray-600 text-left">
          <p>This application is free, open-source software dedicated to the public domain.</p>
          <p>Licensed under <a href="https://opensource.org/license/0bsd" target="_blank" class="text-blue-600 hover:underline">0BSD</a>.</p>

          <p class="text-xs italic text-gray-500 mt-6">
            Provided "as is" without warranty of any kind.<br>
            For clinical documentation purposes only.
          </p>

          <div class="pt-4">
            <p>Built with <a href="https://tauri.app/" target="_blank" class="text-blue-600 hover:underline">Tauri</a>, <a href="https://vuejs.org/" target="_blank" class="text-blue-600 hover:underline">Vue 3</a>, and <a href="https://www.rust-lang.org/" target="_blank" class="text-blue-600 hover:underline">Rust</a>.</p>
            <p class="text-sm mt-1">Version 0.1.0</p>
          </div>
          
          <div class="pt-4 text-center">
            <a href="https://github.com/Kainoa-h/chrio" target="_blank" class="text-blue-600 hover:underline">View on Github</a>
          </div>
        </div>
        
        <div class="mt-8 text-center">
          <button 
            @click="$emit('close')"
            class="px-6 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
