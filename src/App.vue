<script setup lang="ts">
</script>

<template>
  <nav class="p-8 text-center">
    <router-link to="/" class="font-bold text-[#2c3e50] dark:text-[#b0b0b0] mx-2.5 no-underline hover:text-[#535bf2] dark:hover:text-[#24c8db]" active-class="text-[#42b983] dark:text-[#42b983]">Home</router-link> |
    <router-link to="/about" class="font-bold text-[#2c3e50] dark:text-[#b0b0b0] mx-2.5 no-underline hover:text-[#535bf2] dark:hover:text-[#24c8db]" active-class="text-[#42b983] dark:text-[#42b983]">About</router-link>
  </nav>
  <router-view />
</template>