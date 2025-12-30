<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import api from '@/api'
import { useServersStore } from '@/stores/servers'

const route = useRoute()
const router = useRouter()
const serversStore = useServersStore()

const serverId = computed(() => route.params.id as string)
const client = computed(() => serversStore.getClient(serverId.value))
const records = ref<any[]>([])
const loading = ref(true)

async function fetchRecords() {
  try {
    const response = await api.get(`/api/recent/${serverId.value}?limit=60`)
    records.value = response.data || []
  } catch (e) {
    console.error('Failed to fetch records', e)
  } finally {
    loading.value = false
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const mins = Math.floor((seconds % 3600) / 60)
  if (days > 0) return `${days}d ${hours}h`
  if (hours > 0) return `${hours}h ${mins}m`
  return `${mins}m`
}

onMounted(() => {
  serversStore.fetchClients()
  fetchRecords()
})
</script>

<template>
  <div class="detail-page page">
    <!-- Header -->
    <header class="header">
      <div class="container header-content">
        <h1 class="logo">Vanmoi</h1>
        <nav class="nav">
          <router-link to="/">← Dashboard</router-link>
        </nav>
      </div>
    </header>

    <main class="container">
      <div v-if="loading" class="loading-container">
        <div class="loading">Loading server details...</div>
      </div>

      <div v-else-if="!client" class="empty-state">
        <p>Server not found.</p>
        <button @click="router.push('/')" class="btn btn-primary">Back to Dashboard</button>
      </div>

      <template v-else>
        <!-- Server Info -->
        <section class="server-info fade-in">
          <div class="info-header">
            <div class="info-title">
              <span class="status-dot" :class="client.online ? 'online' : 'offline'"></span>
              <h1>{{ client.name }}</h1>
            </div>
            <span class="region-badge">{{ client.region || 'Unknown' }}</span>
          </div>

          <div class="info-grid grid grid-4">
            <div class="info-item card">
              <div class="info-label">OS</div>
              <div class="info-value">{{ client.os || '-' }}</div>
            </div>
            <div class="info-item card">
              <div class="info-label">CPU</div>
              <div class="info-value">{{ client.cpu_name || '-' }}</div>
            </div>
            <div class="info-item card">
              <div class="info-label">Cores</div>
              <div class="info-value">{{ client.cpu_cores || 0 }}</div>
            </div>
            <div class="info-item card">
              <div class="info-label">Memory</div>
              <div class="info-value">{{ formatBytes(client.mem_total) }}</div>
            </div>
          </div>
        </section>

        <!-- Current Status -->
        <section v-if="client.status" class="status-section fade-in">
          <h2 class="section-title">Current Status</h2>
          
          <div class="status-grid grid grid-4">
            <div class="status-card card">
              <div class="status-header">
                <span class="status-label">CPU</span>
                <span class="status-value">{{ client.status.cpu.toFixed(1) }}%</span>
              </div>
              <div class="progress-bar">
                <div 
                  class="progress" 
                  :class="{ low: client.status.cpu < 50, medium: client.status.cpu >= 50 && client.status.cpu < 80, high: client.status.cpu >= 80 }"
                  :style="{ width: client.status.cpu + '%' }"
                ></div>
              </div>
            </div>

            <div class="status-card card">
              <div class="status-header">
                <span class="status-label">Memory</span>
                <span class="status-value">{{ (client.status.ram / client.status.ram_total * 100).toFixed(1) }}%</span>
              </div>
              <div class="progress-bar">
                <div 
                  class="progress"
                  :class="{ low: client.status.ram / client.status.ram_total < 0.5, medium: client.status.ram / client.status.ram_total >= 0.5 && client.status.ram / client.status.ram_total < 0.8, high: client.status.ram / client.status.ram_total >= 0.8 }"
                  :style="{ width: (client.status.ram / client.status.ram_total * 100) + '%' }"
                ></div>
              </div>
              <div class="status-detail">{{ formatBytes(client.status.ram) }} / {{ formatBytes(client.status.ram_total) }}</div>
            </div>

            <div class="status-card card">
              <div class="status-header">
                <span class="status-label">Disk</span>
                <span class="status-value">{{ (client.status.disk / client.status.disk_total * 100).toFixed(1) }}%</span>
              </div>
              <div class="progress-bar">
                <div 
                  class="progress"
                  :class="{ low: client.status.disk / client.status.disk_total < 0.5, medium: client.status.disk / client.status.disk_total >= 0.5 && client.status.disk / client.status.disk_total < 0.8, high: client.status.disk / client.status.disk_total >= 0.8 }"
                  :style="{ width: (client.status.disk / client.status.disk_total * 100) + '%' }"
                ></div>
              </div>
              <div class="status-detail">{{ formatBytes(client.status.disk) }} / {{ formatBytes(client.status.disk_total) }}</div>
            </div>

            <div class="status-card card">
              <div class="status-header">
                <span class="status-label">Uptime</span>
                <span class="status-value">{{ formatUptime(client.status.uptime) }}</span>
              </div>
              <div class="status-detail">
                Load: {{ client.status.load.toFixed(2) }}
              </div>
            </div>
          </div>

          <div class="network-stats grid grid-2">
            <div class="card">
              <div class="status-label">Network In</div>
              <div class="status-value">{{ formatBytes(client.status.net_in) }}/s</div>
            </div>
            <div class="card">
              <div class="status-label">Network Out</div>
              <div class="status-value">{{ formatBytes(client.status.net_out) }}/s</div>
            </div>
          </div>
        </section>

        <section v-else class="offline-notice card">
          <p>This server is currently offline. No real-time data available.</p>
        </section>
      </template>
    </main>
  </div>
</template>

<style scoped>
.loading-container, .empty-state {
  text-align: center;
  padding: 4rem 2rem;
  color: var(--text-secondary);
}

.server-info {
  margin-top: 2rem;
}

.info-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1.5rem;
}

.info-title {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.info-title h1 {
  font-size: 1.75rem;
  font-weight: 700;
}

.region-badge {
  background: var(--bg-card);
  padding: 0.375rem 0.75rem;
  border-radius: 9999px;
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.info-item {
  padding: 1rem;
}

.info-label {
  font-size: 0.75rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 0.25rem;
}

.info-value {
  font-size: 0.875rem;
  color: var(--text-primary);
  word-break: break-word;
}

.status-section {
  margin-top: 2rem;
}

.section-title {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 1.5rem;
}

.status-card {
  padding: 1.25rem;
}

.status-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
}

.status-label {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.status-value {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
}

.status-detail {
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-top: 0.5rem;
}

.network-stats {
  margin-top: 1rem;
}

.network-stats .card {
  padding: 1.25rem;
  text-align: center;
}

.offline-notice {
  margin-top: 2rem;
  text-align: center;
  color: var(--text-secondary);
}
</style>
