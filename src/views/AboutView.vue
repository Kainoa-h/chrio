<script setup lang="ts">
import { useGreetStore } from "../stores/greet";
import { storeToRefs } from "pinia";

const store = useGreetStore();
const { name, greetMsg } = storeToRefs(store);
</script>

<template>
  <div class="about">
    <h1>This is an about page</h1>
    <p>This application is built with Tauri and Vue 3.</p>
    <div v-if="name">
      <p>Current user: <strong>{{ name }}</strong></p>
      <p>Last greeting: <em>{{ greetMsg }}</em></p>
    </div>
  </div>
</template>

<style scoped>
.about {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 50px;
}
</style>