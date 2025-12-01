import { ref } from 'vue'
import { defineStore } from 'pinia'
import { commands, type Client, type CreateClientDto } from '@/bindings'

export const useClientStore = defineStore('client', () => {
  const clients = ref<Client[]>([])
  const error = ref<string | null>(null)

  async function loadClients() {
    try {
      const result = await commands.getClients()
      if (result.status === "ok") {
         clients.value = result.data
      } else {
         error.value = result.error
         console.error('Failed to load clients:', result.error)
      }
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to load clients:', e)
    }
  }

  async function addClient(client: CreateClientDto) {
    try {
      const result = await commands.addClient(client)
      if (result.status === "ok") {
        await loadClients()
      } else {
        error.value = result.error
        console.error('Failed to add client:', result.error)
      }
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to add client:', e)
    }
  }

  return { clients, error, loadClients, addClient }
})
