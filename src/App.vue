<script setup lang="ts">
import { ref } from "vue";
import { Info } from "lucide-vue-next";
import AboutModal from "@/components/AboutModal.vue";

const showAbout = ref(false);
</script>

<template>
  <div class="absolute top-4 right-4 z-40">
    <button 
      @click="showAbout = true"
      class="p-2 text-gray-500 hover:text-blue-600 bg-white/80 backdrop-blur-sm rounded-full shadow-sm hover:shadow transition-all"
      title="About"
    >
      <Info class="w-6 h-6" />
    </button>
  </div>

  <router-view />
  
  <AboutModal v-if="showAbout" @close="showAbout = false" />
</template>