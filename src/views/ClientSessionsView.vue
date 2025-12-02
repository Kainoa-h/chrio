<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { commands, type Session, type Client } from "@/bindings";
import {
  useVueTable,
  FlexRender,
  getCoreRowModel,
  getPaginationRowModel,
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
import { ArrowLeft, Plus, GitCompare, ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight, Edit, Check } from "lucide-vue-next";
import { h } from "vue";

const route = useRoute();
const router = useRouter();
const clientId = Number(route.params.id);
const sessions = ref<Session[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const selectedSessionIds = ref<Array<number>>(new Array());
const client = ref<Client | null>(null);

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
       query: { ids: `${prevSession.id},${currentSession.id}` } 
     });
  }
}

function compareSelected() {
  if (selectedSessionIds.value.length < 2) return;
  const selected = sessions.value.filter(s => selectedSessionIds.value.includes(s.id));
  // Sort by session_number asc (older first)
  selected.sort((a, b) => a.session_number - b.session_number);
  
  // Pass selected IDs as query parameters. Since we can have multiple, we can pass s1, s2, s3... or maybe pass a list if the router supports it?
  // The current CompareSessionsView expects s1 and s2 initially. 
  // But the new CompareSessionsView can handle multiple if we update it to read from a list or just assume 2 initially.
  // Wait, the NEW CompareSessionsView handles dynamic columns. We should update it to accept an array of IDs or handle multiple query params.
  // Let's pass them as a comma-separated list in a new 'ids' query param, or multiple 's' params?
  // Standard way: ids=1,2,3
  
  router.push({ 
     name: 'compare-sessions', 
     params: { clientId }, 
     query: { ids: selected.map(s => s.id).join(',') } 
  });
}

const columns = [
  columnHelper.display({
    id: "select",
    header: () => "Select",
    cell: (info) => {
      const isSelected = selectedSessionIds.value.includes(info.row.original.id);
      return h('div', {
        class: 'relative flex items-center justify-center h-5 w-5 rounded border border-gray-300 bg-white cursor-pointer',
        onClick: (e) => {
          e.stopPropagation(); // Prevent row click from firing
          toggleSelection(info.row.original.id);
        },
      }, [
        isSelected
          ? h(Check, { class: 'h-4 w-4 text-blue-600' })
          : null
      ]);
    },
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

        if (hasPrevious) {
             buttons.push(h('button', { 
                onClick: (e) => {
                  e.stopPropagation();
                  compareSingle(info.row.original);
                },
                class: 'text-blue-400 hover:text-blue-300 flex items-center gap-1 text-xs font-medium mr-3',
                title: 'Compare with previous'
            }, [h(GitCompare, { class: "w-4 h-4" }), 'Compare Prev']));
        }

        buttons.push(h('button', { 
            onClick: (e) => {
              e.stopPropagation();
              router.push({ name: 'edit-session', params: { id: clientId, sessionId: currentId } });
            },
            class: 'text-gray-600 hover:text-gray-900',
            title: 'Edit Session'
        }, [h(Edit, { class: "w-4 h-4" })]));

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
  getPaginationRowModel: getPaginationRowModel(),
  initialState: {
    pagination: {
      pageSize: 10,
    },
  },
});

onMounted(async () => {
  try {
    const clientResult = await commands.getClients();
    if (clientResult.status === "ok") {
      client.value = clientResult.data.find(c => c.id === clientId) || null;
    }

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
              <h1 class="text-3xl font-bold text-gray-900">{{ client?.firstname }} {{ client?.lastname }}'s Sessions</h1>
        </div>
        
        <button 
            v-if="selectedSessionIds.length >= 2"
            @click="compareSelected"
            class="bg-indigo-600 hover:bg-indigo-700 text-white px-3 py-1.5 rounded-lg shadow-sm transition-colors flex items-center gap-2 text-sm"
        >
            <GitCompare class="h-4 w-4" /> Compare Selected ({{ selectedSessionIds.length }})
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

    <div v-else class="space-y-4">
      <div class="rounded-md border border-gray-200 bg-white shadow-sm overflow-hidden">
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
                <TableRow 
                  v-for="row in table.getRowModel().rows" 
                  :key="row.id" 
                  class="transition-colors hover:bg-gray-50 cursor-pointer"
                  @click="toggleSelection(row.original.id)"
                >
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

      <!-- Pagination Controls -->
      <div class="flex items-center justify-between px-2">
        <div class="flex items-center gap-2 text-sm text-gray-700">
          <span>Page</span>
          <span class="font-medium">{{ table.getState().pagination.pageIndex + 1 }}</span>
          <span>of</span>
          <span class="font-medium">{{ table.getPageCount() }}</span>
        </div>
        
        <div class="flex items-center gap-2">
          <select
            :value="table.getState().pagination.pageSize"
            @change="(e) => table.setPageSize(Number((e.target as HTMLSelectElement).value))"
            class="h-8 w-[70px] rounded-md border border-gray-300 bg-white text-gray-700 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option :value="10">10</option>
            <option :value="20">20</option>
            <option :value="30">30</option>
            <option :value="40">40</option>
            <option :value="50">50</option>
          </select>

          <div class="flex items-center space-x-2">
            <button
              class="p-2 rounded-md border border-gray-300 bg-white text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              @click="table.setPageIndex(0)"
              :disabled="!table.getCanPreviousPage()"
            >
              <ChevronsLeft class="h-4 w-4" />
            </button>
            <button
              class="p-2 rounded-md border border-gray-300 bg-white text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              @click="table.previousPage()"
              :disabled="!table.getCanPreviousPage()"
            >
              <ChevronLeft class="h-4 w-4" />
            </button>
            <button
              class="p-2 rounded-md border border-gray-300 bg-white text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              @click="table.nextPage()"
              :disabled="!table.getCanNextPage()"
            >
              <ChevronRight class="h-4 w-4" />
            </button>
            <button
              class="p-2 rounded-md border border-gray-300 bg-white text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              @click="table.setPageIndex(table.getPageCount() - 1)"
              :disabled="!table.getCanNextPage()"
            >
              <ChevronsRight class="h-4 w-4" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
