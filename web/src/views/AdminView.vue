<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import api from '@/api'

const router = useRouter()
const authStore = useAuthStore()

const clients = ref<any[]>([])
const loading = ref(true)

async function fetchClients() {
  try {
    const response = await api.get('/api/admin/clients')
    clients.value = response.data || []
  } catch (e) {
    console.error('Failed to fetch clients', e)
  } finally {
    loading.value = false
  }
}

async function handleLogout() {
  await authStore.logout()
  router.push('/login')
}

async function addClient() {
  const name = prompt('Enter server name:')
  if (!name) return

  try {
    await api.post('/api/admin/clients', { name })
    fetchClients()
  } catch (e) {
    alert('Failed to add client')
  }
}

async function getToken(id: string) {
  try {
    const response = await api.get(`/api/admin/clients/${id}/token`)
    const data = response.data
    alert(`UUID: ${data.uuid}\nToken: ${data.token}`)
  } catch (e) {
    alert('Failed to get token')
  }
}

async function deleteClient(id: string, name: string) {
  if (!confirm(`Delete server "${name}"?`)) return

  try {
    await api.delete(`/api/admin/clients/${id}`)
    fetchClients()
  } catch (e) {
    alert('Failed to delete client')
  }
}

onMounted(() => {
  authStore.fetchUser()
  fetchClients()
})
</script>

<template>
  <div class="admin-page page">
    <!-- Header -->
    <header class="header">
      <div class="container header-content">
        <h1 class="logo">Vanmoi</h1>
        <nav class="nav">
          <router-link to="/">Dashboard</router-link>
          <router-link to="/admin" class="active">Admin</router-link>
          <button @click="handleLogout" class="btn btn-secondary">Logout</button>
        </nav>
      </div>
    </header>

    <main class="container">
      <section class="admin-section fade-in">
        <div class="section-header">
          <h2 class="section-title">Server Management</h2>
          <button @click="addClient" class="btn btn-primary">+ Add Server</button>
        </div>

        <div v-if="loading" class="loading-container">
          <div class="loading">Loading...</div>
        </div>

        <div v-else-if="clients.length === 0" class="empty-state card">
          <p>No servers configured.</p>
          <button @click="addClient" class="btn btn-primary">Add your first server</button>
        </div>

        <div v-else class="clients-table">
          <table>
            <thead>
              <tr>
                <th>Status</th>
                <th>Name</th>
                <th>OS</th>
                <th>Region</th>
                <th>Last Seen</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="client in clients" :key="client.id">
                <td>
                  <span class="status-dot" :class="client.online ? 'online' : 'offline'"></span>
                </td>
                <td>{{ client.name }}</td>
                <td>{{ client.os || '-' }}</td>
                <td>{{ client.region || '-' }}</td>
                <td>{{ client.last_seen_at ? new Date(client.last_seen_at).toLocaleString() : 'Never' }}</td>
                <td class="actions">
                  <button @click="getToken(client.id)" class="btn btn-secondary btn-sm">Token</button>
                  <button @click="deleteClient(client.id, client.name)" class="btn btn-danger btn-sm">Delete</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </main>
  </div>
</template>

<style scoped>
.admin-section {
  margin-top: 2rem;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
}

.section-title {
  font-size: 1.25rem;
  font-weight: 600;
}

.loading-container, .empty-state {
  text-align: center;
  padding: 3rem 2rem;
  color: var(--text-secondary);
}

.empty-state .btn {
  margin-top: 1rem;
}

.clients-table {
  background: var(--bg-secondary);
  border-radius: var(--radius);
  overflow: hidden;
  border: 1px solid var(--border);
}

table {
  width: 100%;
  border-collapse: collapse;
}

th, td {
  padding: 1rem;
  text-align: left;
  border-bottom: 1px solid var(--border);
}

th {
  background: var(--bg-card);
  font-weight: 500;
  color: var(--text-secondary);
  font-size: 0.875rem;
}

td {
  font-size: 0.875rem;
}

tr:last-child td {
  border-bottom: none;
}

tr:hover {
  background: rgba(255, 255, 255, 0.02);
}

.actions {
  display: flex;
  gap: 0.5rem;
}

.btn-sm {
  padding: 0.375rem 0.75rem;
  font-size: 0.75rem;
}

.btn-danger {
  background: var(--danger);
  color: white;
}

.btn-danger:hover {
  background: #dc2626;
}
</style>
