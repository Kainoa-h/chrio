import { defineStore } from "pinia";
import { commands, type Client, type CreateClientDto, type UpdateClientDto } from "../bindings";

export const useClientStore = defineStore("client", {
  state: () => ({
    clients: [] as Client[],
    loading: false,
    error: null as string | null,
  }),
  actions: {
    async loadClients() {
      this.loading = true;
      this.error = null;
      try {
        const result = await commands.getClients();
        if (result.status === "ok") {
          this.clients = result.data;
        } else {
          this.error = result.error;
        }
      } catch (e: any) {
        this.error = e.message || "Failed to load clients";
      } finally {
        this.loading = false;
      }
    },
    async addClient(client: CreateClientDto) {
      this.loading = true;
      this.error = null;
      try {
        const result = await commands.addClient(client);
        if (result.status === "ok") {
          await this.loadClients();
        } else {
          this.error = result.error;
        }
      } catch (e: any) {
        this.error = e.message || "Failed to add client";
      } finally {
        this.loading = false;
      }
    },
    async updateClient(client: UpdateClientDto) {
      this.loading = true;
      this.error = null;
      try {
        const result = await commands.updateClient(client);
        if (result.status === "ok") {
          await this.loadClients();
        } else {
          this.error = result.error;
        }
      } catch (e: any) {
        this.error = e.message || "Failed to update client";
      } finally {
        this.loading = false;
      }
    },
  },
});
