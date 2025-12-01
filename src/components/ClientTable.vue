<script setup lang="ts">
import {
  useVueTable,
  FlexRender,
  getCoreRowModel,
  getFilteredRowModel,
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
import { Search } from "lucide-vue-next";

const props = defineProps<{
  clients: Client[];
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
    cell: () => h('a', { href: '#', class: 'font-medium text-blue-600 dark:text-blue-500 hover:underline' }, 'Edit'),
  }),
];

const table = useVueTable({
  get data() {
    return props.clients;
  },
  columns,
  getCoreRowModel: getCoreRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
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
          class="flex h-9 w-full rounded-md border border-gray-700 bg-gray-900 px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-gray-500 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-blue-500 disabled:cursor-not-allowed disabled:opacity-50 pl-8 text-gray-200"
        />
      </div>
    </div>

    <!-- Table -->
    <div class="rounded-md border border-gray-700 bg-gray-950 shadow-sm overflow-hidden">
      <div class="relative w-full overflow-auto"> <!-- Added this div to handle scrolling -->
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
                  No results found.
                </TableCell>
              </TableRow>
            </template>
          </TableBody>
        </Table>
      </div>
    </div>
  </div>
</template>