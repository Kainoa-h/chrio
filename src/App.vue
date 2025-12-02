<script setup lang="ts">
</script>

<template>
  <nav class="p-8 text-center">
    <router-link to="/" class="font-bold text-[#2c3e50] mx-2.5 no-underline hover:text-[#535bf2]" active-class="text-[#42b983]">Home</router-link> |
    <router-link to="/about" class="font-bold text-[#2c3e50] mx-2.5 no-underline hover:text-[#535bf2]" active-class="text-[#42b983]">About</router-link>
  </nav>
  <router-view />
</template>