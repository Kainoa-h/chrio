<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { commands, type Session } from "@/bindings";
import {
  useVueTable,
  FlexRender,
  getCoreRowModel,
  createColumnHelper,
} from "@tanstack/vue-table";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ArrowLeft, Plus } from "lucide-vue-next";

const route = useRoute();
const router = useRouter();
const clientId = Number(route.params.id);
const sessions = ref<Session[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);

const columnHelper = createColumnHelper<Session>();

function formatDate(dateString: string) {
  if (!dateString) return "-";
  const date = new Date(dateString);
  return new Intl.DateTimeFormat('en-GB', { 
    day: '2-digit', 
    month: 'long', 
    year: 'numeric' 
  }).format(date);
}

const columns = [
  columnHelper.accessor("session_number", {
    header: "Session #",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("datetime", {
    header: "Date",
    cell: (info) => formatDate(info.getValue()),
  }),
  columnHelper.accessor("notes", {
    header: "Notes",
    cell: (info) => {
      const notes = info.getValue();
      if (!notes) return "-";
      return notes.length > 50 ? notes.substring(0, 50) + "..." : notes;
    },
  }),
];

const table = useVueTable({
  get data() {
    return sessions.value;
  },
  columns,
  getCoreRowModel: getCoreRowModel(),
});

onMounted(async () => {
  try {
    const result = await commands.getClientSessions(clientId);
    if (result.status === "ok") {
      sessions.value = result.data;
    } else {
      error.value = result.error;
    }
  } catch (e: any) {
    error.value = e.message || "An unknown error occurred";
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="p-8 max-w-6xl mx-auto">
    <div class="flex justify-between items-center mb-6">
      <div class="flex items-center">
        <button 
          @click="router.push({ name: 'home' })" 
          class="mr-4 p-2 rounded-full hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
        >
          <ArrowLeft class="h-6 w-6 text-gray-600 dark:text-gray-300" />
        </button>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Client Sessions</h1>
      </div>
      <button 
        @click="router.push({ name: 'add-session', params: { id: clientId } })"
        class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg shadow-sm transition-colors flex items-center gap-2"
      >
        <Plus class="h-5 w-5" /> Add New Session
      </button>
    </div>

    <div v-if="loading" class="text-center py-8 text-gray-500">
      Loading sessions...
    </div>

    <div v-else-if="error" class="text-center py-8 text-red-500">
      Error: {{ error }}
    </div>

    <div v-else class="rounded-md border border-gray-700 bg-gray-950 shadow-sm overflow-hidden">
       <div class="relative w-full overflow-auto">
        <Table>
          <TableHeader>
            <TableRow v-for="headerGroup in table.getHeaderGroups()" :key="headerGroup.id" class="bg-gray-800">
              <TableHead v-for="header in headerGroup.headers" :key="header.id" class="font-bold text-gray-200">
                <FlexRender
                  v-if="!header.isPlaceholder"
                  :render="header.column.columnDef.header"
                  :props="header.getContext()"
                />
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <template v-if="table.getRowModel().rows?.length">
              <TableRow v-for="row in table.getRowModel().rows" :key="row.id" class="transition-colors hover:bg-gray-800">
                <TableCell v-for="cell in row.getVisibleCells()" :key="cell.id" class="text-gray-300">
                  <FlexRender
                    :render="cell.column.columnDef.cell"
                    :props="cell.getContext()"
                  />
                </TableCell>
              </TableRow>
            </template>
            <template v-else>
              <TableRow>
                <TableCell :colspan="columns.length" class="h-24 text-center text-gray-400">
                  No sessions found for this client.
                </TableCell>
              </TableRow>
            </template>
          </TableBody>
        </Table>
      </div>
    </div>
  </div>
</template>
