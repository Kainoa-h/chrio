import { defineStore } from 'pinia'
import Database from '@tauri-apps/plugin-sql'
import { ref } from 'vue'

export const useDbStore = defineStore('db', () => {
  const db = ref<Database | null>(null)
  const error = ref<string | null>(null)

  async function init() {
    try {
      db.value = await Database.load('sqlite:chrio.db')
      console.log('Database loaded successfully')
      
      // Example: Create a table if it doesn't exist
      await db.value.execute(
        'CREATE TABLE IF NOT EXISTS todos (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT, completed BOOLEAN)'
      )
      
      await db.value.execute(
        `CREATE TABLE IF NOT EXISTS clients (
          id INTEGER PRIMARY KEY AUTOINCREMENT, 
          firstname TEXT, 
          lastname TEXT, 
          dob TEXT, 
          sex TEXT, 
          registration_date TEXT
        )`
      )
    } catch (e: any) {
      error.value = e.message
      console.error('Failed to load database:', e)
    }
  }

  async function addTodo(title: string) {
    if (!db.value) return
    await db.value.execute('INSERT INTO todos (title, completed) VALUES ($1, $2)', [title, false])
  }

  async function getTodos() {
    if (!db.value) return []
    return await db.value.select('SELECT * FROM todos')
  }

  async function getClients() {
    if (!db.value) return []
    return await db.value.select('SELECT * FROM clients ORDER BY registration_date DESC')
  }

  async function addClient(client: { firstname: string, lastname: string, dob: string, sex: string }) {
    if (!db.value) return
    const registration_date = new Date().toISOString()
    await db.value.execute(
      'INSERT INTO clients (firstname, lastname, dob, sex, registration_date) VALUES ($1, $2, $3, $4, $5)',
      [client.firstname, client.lastname, client.dob, client.sex, registration_date]
    )
  }

  return { db, error, init, addTodo, getTodos, getClients, addClient }
})
