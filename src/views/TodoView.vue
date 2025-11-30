<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useDbStore } from '../stores/db';

const dbStore = useDbStore();
const todos = ref<any[]>([]);
const newTodo = ref('');

async function loadTodos() {
  const result = await dbStore.getTodos();
  if (result) {
    todos.value = result as any[];
  }
}

async function add() {
  if (!newTodo.value.trim()) return;
  await dbStore.addTodo(newTodo.value);
  newTodo.value = '';
  await loadTodos();
}

onMounted(async () => {
  await dbStore.init();
  await loadTodos();
});
</script>

<template>
  <div class="flex flex-col items-center pt-12">
    <h1 class="text-2xl font-bold mb-4">SQLite Todo List</h1>
    
    <div class="w-full max-w-md">
      <form @submit.prevent="add" class="flex gap-2 mb-4">
        <input 
          v-model="newTodo" 
          placeholder="Add a task..." 
          class="flex-1 rounded-lg border border-gray-300 px-4 py-2 focus:border-blue-500 outline-none dark:bg-gray-800 dark:border-gray-600 dark:text-white"
        />
        <button 
          type="submit"
          class="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors"
        >
          Add
        </button>
      </form>

      <ul class="space-y-2">
        <li 
          v-for="todo in todos" 
          :key="todo.id"
          class="p-3 bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 flex justify-between items-center"
        >
          <span :class="{ 'line-through text-gray-500': todo.completed }">{{ todo.title }}</span>
          <span class="text-xs text-gray-400">ID: {{ todo.id }}</span>
        </li>
      </ul>

      <div v-if="todos.length === 0" class="text-center text-gray-500 mt-4">
        No tasks yet. Add one above!
      </div>
      
      <div v-if="dbStore.error" class="text-red-500 mt-4 text-center">
        Error: {{ dbStore.error }}
      </div>
    </div>
  </div>
</template>
