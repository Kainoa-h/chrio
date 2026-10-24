<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRoute } from "vue-router";
import { Info, Settings, Archive } from "lucide-vue-next";
import CameraModal from "@/components/CameraModal.vue";
import AboutModal from "@/components/AboutModal.vue";
import DataTransferModal from "@/components/DataTransferModal.vue";
import { dataTransferActive } from "@/lib/dataTransfer";
import { commands } from "@/bindings";
import Toast from "@/components/Toast.vue";
import { useToast } from "@/composables/useToast";

const showTransfer = ref(false);
const route = useRoute();
const startupMessage = ref<string | null>(null);
onMounted(async () => {
  try { startupMessage.value = await commands.backupStartupMessage(); } catch { /* No recovery message available. */ }
});
const showAbout = ref(false);
const showSettings = ref(false);
const showBetaWarning = ref(true);
const { show, message, type, closeToast } = useToast();
</script>

<template>
  <Toast :show="show" :message="message" :type="type" @close="closeToast" />

  <div v-if="route.name === 'home'" :inert="dataTransferActive" class="relative z-40 flex justify-end gap-2 px-4 pt-4">
    <button @click="showSettings = true" class="p-2 text-gray-500 hover:text-blue-600 bg-white/80 rounded-full shadow-sm" title="Settings" aria-label="Settings">
      <Settings class="w-6 h-6" />
    </button>
    <button @click="showTransfer = true" class="p-2 text-gray-500 hover:text-blue-600 bg-white/80 rounded-full shadow-sm" title="Import / Export" aria-label="Import / Export">
      <Archive class="w-6 h-6" />
    </button>
    <button 
      @click="showAbout = true"
      class="p-2 text-gray-500 hover:text-blue-600 bg-white/80 backdrop-blur-sm rounded-full shadow-sm hover:shadow transition-all"
      title="About"
    >
      <Info class="w-6 h-6" />
    </button>
  </div>

  <div :inert="dataTransferActive">
    <div v-if="startupMessage" role="status" class="mx-auto mt-4 max-w-3xl rounded-lg border border-blue-200 bg-blue-50 p-4 text-sm text-gray-800">
      {{ startupMessage }}
      <button class="ml-3 underline" @click="startupMessage = null">Dismiss</button>
    </div>
    <router-view />
  
    <AboutModal v-if="showAbout" @close="showAbout = false" />
    <CameraModal :show="showSettings" settings-only @close="showSettings = false" />
  </div>
  <DataTransferModal v-if="showTransfer" @close="showTransfer = false" />

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
