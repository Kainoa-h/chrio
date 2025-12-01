<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { commands, type Session } from "@/bindings";
import { ArrowLeft } from "lucide-vue-next";
import RatioImage from "@/components/RatioImage.vue";

const route = useRoute();
const router = useRouter();
const clientId = Number(route.params.clientId);
const session1Id = Number(route.query.s1);
const session2Id = Number(route.query.s2);

const session1 = ref<Session | null>(null);
const session2 = ref<Session | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);

// Store base64 images: images[sessionId][type] = base64String
const images = ref<Record<number, Record<string, string>>>({});

async function loadImage(sessionId: number, type: string, path: string | null) {
  if (!path) return;
  try {
    const result = await commands.readImageBase64(path);
    if (result.status === "ok") {
      if (!images.value[sessionId]) images.value[sessionId] = {};
      images.value[sessionId][type] = result.data;
    }
  } catch (e) {
    console.error(`Failed to load image ${type} for session ${sessionId}`, e);
  }
}

async function loadSessionImages(session: Session) {
  await Promise.all([
    loadImage(session.id, 'anterior', session.anterior),
    loadImage(session.id, 'posterior', session.posterior),
    loadImage(session.id, 'right_lateral', session.right_lateral),
    loadImage(session.id, 'left_lateral', session.left_lateral),
  ]);
}

onMounted(async () => {
  try {
    const result = await commands.getClientSessions(clientId);
    if (result.status === "ok") {
      const allSessions = result.data;
      session1.value = allSessions.find(s => s.id === session1Id) || null;
      session2.value = allSessions.find(s => s.id === session2Id) || null;

      if (!session1.value || !session2.value) {
        error.value = "Could not find one or both sessions.";
        return;
      }

      await Promise.all([
        loadSessionImages(session1.value),
        loadSessionImages(session2.value)
      ]);
    } else {
      error.value = result.error;
    }
  } catch (e: any) {
    error.value = e.message || "An unknown error occurred";
  } finally {
    loading.value = false;
  }
});

function getParsedCrop(session: Session | null, type: string) {
  if (!session) return null;
  const key = `${type}_crop` as keyof Session;
  const cropString = session[key];
  if (typeof cropString === 'string') {
    try {
      return JSON.parse(cropString);
    } catch (e) {
      console.error("Failed to parse crop data", e);
    }
  }
  return null;
}

function formatDate(dateString: string | undefined) {
  if (!dateString) return "-";
  const date = new Date(dateString);
  return new Intl.DateTimeFormat('en-GB', { 
    day: '2-digit', 
    month: 'long', 
    year: 'numeric' 
  }).format(date);
}

const imageTypes = [
  { key: 'anterior', label: 'Anterior' },
  { key: 'posterior', label: 'Posterior' },
  { key: 'right_lateral', label: 'Right Lateral' },
  { key: 'left_lateral', label: 'Left Lateral' },
];
</script>

<template>
  <div class="p-8 max-w-6xl mx-auto pb-20">
    <div class="flex items-center mb-6">
      <button 
        @click="router.back()" 
        class="mr-4 p-2 rounded-full hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
      >
        <ArrowLeft class="h-6 w-6 text-gray-600 dark:text-gray-300" />
      </button>
      <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Compare Sessions</h1>
    </div>

    <div v-if="loading" class="text-center py-8 text-gray-500">
      Loading comparison...
    </div>

    <div v-else-if="error" class="text-center py-8 text-red-500">
      Error: {{ error }}
    </div>

    <div v-else class="grid grid-cols-2 gap-8">
      <!-- Session 1 Header -->
      <div class="bg-gray-900 p-4 rounded-t-lg border border-gray-700 text-center">
        <h2 class="text-xl font-bold text-white">Session #{{ session1?.session_number }}</h2>
        <p class="text-gray-400 text-sm">{{ formatDate(session1?.datetime) }}</p>
      </div>

      <!-- Session 2 Header -->
      <div class="bg-gray-900 p-4 rounded-t-lg border border-gray-700 text-center">
        <h2 class="text-xl font-bold text-white">Session #{{ session2?.session_number }}</h2>
        <p class="text-gray-400 text-sm">{{ formatDate(session2?.datetime) }}</p>
      </div>

      <!-- Images Rows -->
      <template v-for="type in imageTypes" :key="type.key">
        <!-- Label Row -->
        <div class="col-span-2 text-center py-2 border-b border-gray-700">
          <h3 class="text-lg font-medium text-gray-300">{{ type.label }}</h3>
        </div>

        <!-- Image 1 -->
        <RatioImage 
          :src="session1 && images[session1.id]?.[type.key] ? images[session1.id][type.key] : null" 
          :crop="getParsedCrop(session1, type.key)"
          empty-text="No Image" 
          container-class="w-full" 
        />

        <!-- Image 2 -->
        <RatioImage 
          :src="session2 && images[session2.id]?.[type.key] ? images[session2.id][type.key] : null" 
          :crop="getParsedCrop(session2, type.key)"
          empty-text="No Image" 
          container-class="w-full" 
        />
      </template>

      <!-- Data Comparison -->
      <div class="col-span-2 mt-8">
        <h3 class="text-xl font-bold text-gray-200 mb-4 border-b border-gray-700 pb-2">Data Comparison</h3>
        
        <div class="grid grid-cols-3 gap-4 text-sm">
           <div class="font-bold text-gray-400 text-right pr-4">Field</div>
           <div class="font-bold text-gray-300 text-center">Session #{{ session1?.session_number }}</div>
           <div class="font-bold text-gray-300 text-center">Session #{{ session2?.session_number }}</div>

           <!-- Height -->
           <div class="text-gray-400 text-right pr-4 py-2 border-b border-gray-800">Height</div>
           <div class="text-gray-200 text-center py-2 border-b border-gray-800">{{ session1?.height ? session1.height + ' cm' : '-' }}</div>
           <div class="text-gray-200 text-center py-2 border-b border-gray-800">{{ session2?.height ? session2.height + ' cm' : '-' }}</div>

           <!-- Weight -->
           <div class="text-gray-400 text-right pr-4 py-2 border-b border-gray-800">Weight</div>
           <div class="text-gray-200 text-center py-2 border-b border-gray-800">{{ session1?.weight ? session1.weight + ' kg' : '-' }}</div>
           <div class="text-gray-200 text-center py-2 border-b border-gray-800">{{ session2?.weight ? session2.weight + ' kg' : '-' }}</div>

           <!-- Notes -->
           <div class="text-gray-400 text-right pr-4 py-2">Notes</div>
           <div class="text-gray-200 p-2 bg-gray-900 rounded border border-gray-700 text-xs whitespace-pre-wrap">{{ session1?.notes || '-' }}</div>
           <div class="text-gray-200 p-2 bg-gray-900 rounded border border-gray-700 text-xs whitespace-pre-wrap">{{ session2?.notes || '-' }}</div>
        </div>
      </div>
    </div>
  </div>
</template>
