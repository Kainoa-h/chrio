<script setup lang="ts">
import { ref } from "vue";
import { Info, Settings } from "lucide-vue-next";
import CameraModal from "@/components/CameraModal.vue";
import AboutModal from "@/components/AboutModal.vue";
import Toast from "@/components/Toast.vue";
import { useToast } from "@/composables/useToast";

const showAbout = ref(false);
const showSettings = ref(false);
const showBetaWarning = ref(true);
const { show, message, type, closeToast } = useToast();
</script>

<template>
  <Toast :show="show" :message="message" :type="type" @close="closeToast" />

  <div class="absolute top-4 right-4 z-40 flex gap-2">
    <button @click="showSettings = true" class="p-2 text-gray-500 hover:text-blue-600 bg-white/80 rounded-full shadow-sm" title="Settings" aria-label="Settings">
      <Settings class="w-6 h-6" />
    </button>
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
  <CameraModal :show="showSettings" settings-only @close="showSettings = false" />

  <button
    v-if="showBetaWarning"
    type="button"
    class="fixed bottom-4 left-1/2 -translate-x-1/2 z-50 p-2 bg-yellow-400 text-yellow-900 rounded-lg shadow-lg text-sm font-medium cursor-pointer"
    title="Click to dismiss"
    aria-label="Beta build: use with caution. Dismiss warning."
    @click="showBetaWarning = false"
  >
    ⚠️ This is a beta build. Use with caution.
  </button>
</template>
