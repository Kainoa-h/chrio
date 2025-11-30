import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useGreetStore = defineStore('greet', () => {
  const name = ref('')
  const greetMsg = ref('')

  return { name, greetMsg }
})
