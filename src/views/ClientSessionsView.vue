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
import { ArrowLeft, Plus, GitCompare, ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight, Edit } from "lucide-vue-next";
import { h } from "vue";

const route = useRoute();
const router = useRouter();
const clientId = Number(route.params.id);
const sessions = ref<Session[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const selectedSessionIds = ref<Array<number>>(new Array());

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

function toggleSelection(id: number) {
  const arr = selectedSessionIds.value;
  const index = arr.indexOf(id);
  if (arr.includes(id)) {
    arr.splice(index, 1);
    return;
  }
  if (arr.length === 2) {
    arr.pop(1);
  }
  arr.push(id);
}

function compareSingle(currentSession: Session) {
  // Find previous session (session_number - 1) or simply the one with next lower ID/date if numbers aren't sequential? 
  // Assuming sequential or sorted by date desc.
  // sessions.value is usually sorted by session_number desc (from backend query).
  const currentIndex = sessions.value.findIndex(s => s.id === currentSession.id);
  // If sorted desc, previous session (older) is at index + 1
  const prevSession = sessions.value[currentIndex + 1];
  
  if (prevSession) {
     router.push({ 
       name: 'compare-sessions', 
       params: { clientId }, 
       query: { s1: prevSession.id, s2: currentSession.id } 
     });
  }
}

function compareSelected() {
  if (selectedSessionIds.value.length !== 2) return;
  const selected = sessions.value.filter(s => selectedSessionIds.value.includes(s.id));
  // Sort by session_number asc (older first)
  selected.sort((a, b) => a.session_number - b.session_number);
  
  router.push({ 
     name: 'compare-sessions', 
     params: { clientId }, 
     query: { s1: selected[0].id, s2: selected[1].id } 
  });
}

const columns = [
  columnHelper.display({
    id: "select",
    header: () => "Select",
    cell: (info) => h('input', { 
      type: 'checkbox', 
      checked: selectedSessionIds.value.includes(info.row.original.id),
      onChange: () => toggleSelection(info.row.original.id),
      class: "rounded border-gray-300 bg-white text-blue-600 focus:ring-blue-500"
    }),
  }),
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
  columnHelper.display({
    id: "actions",
    header: "Actions",
    cell: (info) => {
        const currentId = info.row.original.id;
        const currentIndex = sessions.value.findIndex(s => s.id === currentId);
        const hasPrevious = currentIndex < sessions.value.length - 1; // Assuming desc sort
        
        const buttons = [];

        buttons.push(h('button', { 
            onClick: () => router.push({ name: 'edit-session', params: { id: clientId, sessionId: currentId } }),
            class: 'text-gray-600 hover:text-gray-900 mr-3',
            title: 'Edit Session'
        }, [h(Edit, { class: "w-4 h-4" })]));

        if (hasPrevious) {
             buttons.push(h('button', { 
                onClick: () => compareSingle(info.row.original),
                class: 'text-blue-400 hover:text-blue-300 flex items-center gap-1 text-xs font-medium',
                title: 'Compare with previous'
            }, [h(GitCompare, { class: "w-4 h-4" }), 'Compare Prev']));
        }

        return h('div', { class: 'flex items-center' }, buttons);
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
      <div class="flex items-center gap-4">
        <div class="flex items-center">
            <button 
            @click="router.push({ name: 'home' })" 
            class="mr-4 p-2 rounded-full hover:bg-gray-100 transition-colors"
            >
            <ArrowLeft class="h-6 w-6 text-gray-600" />
            </button>
            <h1 class="text-3xl font-bold text-gray-900">Client Sessions</h1>
        </div>
        
        <button 
            v-if="selectedSessionIds.length === 2"
            @click="compareSelected"
            class="bg-indigo-600 hover:bg-indigo-700 text-white px-3 py-1.5 rounded-lg shadow-sm transition-colors flex items-center gap-2 text-sm"
        >
            <GitCompare class="h-4 w-4" /> Compare Selected (2)
        </button>
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

    <div v-else class="rounded-md border border-gray-200 bg-white shadow-sm overflow-hidden">
       <div class="relative w-full overflow-auto">
        <Table>
          <TableHeader>
            <TableRow v-for="headerGroup in table.getHeaderGroups()" :key="headerGroup.id" class="bg-gray-100">
              <TableHead v-for="header in headerGroup.headers" :key="header.id" class="font-bold text-gray-700">
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
              <TableRow v-for="row in table.getRowModel().rows" :key="row.id" class="transition-colors hover:bg-gray-50">
                <TableCell v-for="cell in row.getVisibleCells()" :key="cell.id" class="text-gray-800">
                  <FlexRender
                    :render="cell.column.columnDef.cell"
                    :props="cell.getContext()"
                  />
                </TableCell>
              </TableRow>
            </template>
            <template v-else>
              <TableRow>
                <TableCell :colspan="columns.length" class="h-24 text-center text-gray-500">
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
