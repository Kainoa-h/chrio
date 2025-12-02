<script setup lang="ts">
import {
  useVueTable,
  FlexRender,
  getCoreRowModel,
  getFilteredRowModel,
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
import type { Client } from "@/bindings";
import { h, ref } from "vue";
import { Search, ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight, Edit } from "lucide-vue-next";
import { RouterLink } from "vue-router";

const props = defineProps<{
  clients: Client[];
}>();

const emit = defineEmits<{
  (e: 'edit', client: Client): void
}>();

const globalFilter = ref("");

const columnHelper = createColumnHelper<Client>();

function formatDate(dateString: string) {
  if (!dateString) return "-";
  return new Date(dateString).toLocaleDateString();
}

const columns = [
  columnHelper.accessor("id", {
    header: "ID",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("firstname", {
    header: "First Name",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("lastname", {
    header: "Last Name",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("dob", {
    header: "Date of Birth",
    cell: (info) => formatDate(info.getValue()),
  }),
  columnHelper.accessor("sex", {
    header: "Sex",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("registration_date", {
    header: "Registration Date",
    cell: (info) => formatDate(info.getValue()),
  }),
  columnHelper.display({
    id: "actions",
    header: "Actions",
    cell: (info) => h('div', { class: 'flex items-center gap-4' }, [
      h('button', {
        onClick: () => emit('edit', info.row.original),
        class: 'text-gray-500 hover:text-blue-600 transition-colors',
        title: 'Edit Client'
      }, h(Edit, { class: 'w-4 h-4' })),
      h(RouterLink, { 
        to: { name: 'client-sessions', params: { id: info.row.original.id } },
        class: 'font-medium text-blue-600 hover:underline' 
      }, () => 'Sessions')
    ]),
  }),
];

const table = useVueTable({
  get data() {
    return props.clients;
  },
  columns,
  getCoreRowModel: getCoreRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  getPaginationRowModel: getPaginationRowModel(),
  initialState: {
    pagination: {
      pageSize: 20,
    },
  },
  state: {
    get globalFilter() {
      return globalFilter.value;
    },
  },
  onGlobalFilterChange: (updaterOrValue) => {
    if (typeof updaterOrValue === 'function') {
      globalFilter.value = updaterOrValue(globalFilter.value);
    } else {
      globalFilter.value = updaterOrValue;
    }
  },
});
</script>

<template>
  <div class="space-y-4">
    <!-- Search Input -->
    <div class="flex items-center py-4">
      <div class="relative max-w-sm w-full">
        <Search class="absolute left-2 top-2.5 h-4 w-4 text-gray-400" />
        <input
          v-model="globalFilter"
          placeholder="Search clients..."
          class="flex h-9 w-full rounded-md border border-gray-300 bg-white px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-gray-500 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-blue-500 disabled:cursor-not-allowed disabled:opacity-50 pl-8 text-gray-900"
        />
      </div>
    </div>

    <!-- Table -->
    <div class="rounded-md border border-gray-200 bg-white shadow-sm overflow-hidden">
      <div class="relative w-full overflow-auto"> <!-- Added this div to handle scrolling -->
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
                  No results found.
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
</template>