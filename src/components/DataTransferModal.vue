<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { commands, type DataSummary, type Result } from '@/bindings';
import { dataTransferActive, flushTransferSaves } from '@/lib/dataTransfer';

const emit = defineEmits<{ close: [] }>();
const current = ref<DataSummary | null>(null);
const incoming = ref<DataSummary | null>(null);
const phase = ref<'home' | 'choice' | 'confirm'>('home');
const busy = ref(false);
const progress = ref('');
const error = ref('');
const notice = ref('');
const opened = ref(false);
const dialog = ref<HTMLElement | null>(null);
const hasData = computed(() => !!current.value && (current.value.clients > 0 || current.value.sessions > 0 || current.value.photos > 0));

function unwrap<T>(result: Result<T, string>): T {
  if (result.status === 'error') throw new Error(result.error);
  return result.data;
}
async function run(message: string, action: () => Promise<void>) {
  if (busy.value) return;
  busy.value = true;
  progress.value = message;
  error.value = '';
  notice.value = '';
  try { await action(); }
  catch (e) { error.value = e instanceof Error ? e.message : String(e); }
  finally { busy.value = false; progress.value = ''; }
}
async function start() {
  await run('Finishing saves…', async () => {
    await flushTransferSaves();
    current.value = unwrap(await commands.beginBackup());
    opened.value = true;
  });
}
async function exportData(beforeImport = false) {
  await run('Creating and verifying your backup. Large photo libraries may take several minutes…', async () => {
    const path = unwrap(await commands.exportBackup());
    if (path === null) return;
    notice.value = `Backup saved to ${path}`;
    if (beforeImport) phase.value = 'confirm';
  });
}
async function selectImport() {
  await run('Checking the archive and preparing all photos. Your current data is unchanged…', async () => {
    const result = unwrap(await commands.validateBackup());
    if (result === null) return;
    incoming.value = result;
    phase.value = hasData.value ? 'choice' : 'confirm';
  });
}
async function importData() {
  await run('Preparing the restore and restarting Chrio. Please keep the app open…', async () => {
    unwrap(await commands.importBackup());
  });
}
async function close() {
  if (busy.value) return;
  await run('Closing…', async () => {
    // Also releases a stale session after a webview reload or failed initialization.
    unwrap(await commands.endBackup());
    dataTransferActive.value = false;
    emit('close');
  });
}
function trapFocus(event: KeyboardEvent) {
  const buttons = [...(dialog.value?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)') ?? [])];
  const index = buttons.indexOf(document.activeElement as HTMLButtonElement);
  event.preventDefault();
  if (buttons.length) buttons[(index + (event.shiftKey ? -1 : 1) + buttons.length) % buttons.length].focus();
  else dialog.value?.focus();
}
onMounted(() => { dialog.value?.focus(); start(); });
</script>

<template>
  <div class="fixed inset-0 z-[80] flex items-center justify-center bg-black/50 p-4" @keydown.esc.prevent.stop="close">
    <section ref="dialog" tabindex="-1" @keydown.tab="trapFocus" role="dialog" aria-modal="true" aria-labelledby="transfer-title" :aria-busy="busy" class="w-full max-w-lg rounded-xl bg-white p-6 shadow-xl space-y-4">
      <h2 id="transfer-title" class="text-xl font-semibold text-gray-900">Import / Export</h2>
      <p class="text-sm text-gray-600">Back up or replace all clients, sessions, notes, measurements, crop information, and original photos. Camera and app settings are excluded.</p>
      <p v-if="current" class="text-sm text-gray-700">Current data: {{ current.clients }} clients · {{ current.sessions }} sessions · {{ current.photos }} photos</p>
      <p v-if="incoming" class="text-sm text-gray-700">Selected backup: {{ incoming.clients }} clients · {{ incoming.sessions }} sessions · {{ incoming.photos }} photos</p>
      <p v-if="progress" role="status" class="text-sm text-blue-700">{{ progress }}</p>
      <p v-if="notice" role="status" class="break-words text-sm text-green-700">{{ notice }}</p>
      <p v-if="error" role="alert" class="break-words text-sm text-red-700">{{ error }}</p>

      <div v-if="opened && phase === 'home'" class="flex flex-col gap-2">
        <button :disabled="busy" @click="exportData()" class="transfer-primary">Export all data…</button>
        <button :disabled="busy" @click="selectImport" class="transfer-secondary">Import from ZIP…</button>
      </div>
      <div v-else-if="opened && phase === 'choice'" class="space-y-3">
        <p class="text-sm font-medium text-red-700">Importing will overwrite all current client data and photos. Export your current data first so you can recover it later.</p>
        <div class="flex flex-col gap-2">
          <button :disabled="busy" @click="exportData(true)" class="transfer-primary">Export current data first…</button>
          <button :disabled="busy" @click="phase = 'confirm'; error = ''; notice = ''" class="transfer-danger">Replace without exporting</button>
        </div>
      </div>
      <div v-else-if="opened && phase === 'confirm'" class="space-y-3">
        <p class="text-sm font-medium text-red-700">{{ hasData ? 'All current client data and photos will be replaced by this backup.' : 'This backup will become your client data.' }} Chrio will restart to complete the import.</p>
        <button :disabled="busy" @click="importData" class="transfer-danger w-full">Import and restart</button>
      </div>
      <button v-if="!opened && !busy" @click="start" class="transfer-primary w-full">Retry</button>
      <div class="flex justify-end gap-3">
        <button v-if="phase !== 'home'" :disabled="busy" @click="phase = 'home'; incoming = null; error = ''; notice = ''" class="text-sm text-gray-600 disabled:opacity-50">Back</button>
        <button :disabled="busy" @click="close" class="transfer-secondary">{{ phase === 'home' ? 'Close' : 'Cancel' }}</button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.transfer-primary, .transfer-secondary, .transfer-danger { @apply rounded-lg px-4 py-2 text-sm font-medium disabled:opacity-50 disabled:cursor-wait; }
.transfer-primary { @apply bg-blue-600 text-white hover:bg-blue-700; }
.transfer-secondary { @apply border border-gray-300 bg-white text-gray-700 hover:bg-gray-50; }
.transfer-danger { @apply bg-red-600 text-white hover:bg-red-700; }
</style>
