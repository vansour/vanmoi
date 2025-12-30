<script setup lang="ts">
import { computed } from 'vue'

interface ClientStatus {
  cpu: number
  ram: number
  ram_total: number
  disk: number
  disk_total: number
  net_in: number
  net_out: number
}

interface Client {
  id: string
  name: string
  os: string
  region: string
  group_name: string
  online: boolean
  status?: ClientStatus
}

const props = defineProps<{
  client: Client
}>()

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

const cpuPercent = computed(() => props.client.status?.cpu || 0)
const ramPercent = computed(() => {
  if (!props.client.status) return 0
  return (props.client.status.ram / props.client.status.ram_total) * 100
})
const diskPercent = computed(() => {
  if (!props.client.status) return 0
  return (props.client.status.disk / props.client.status.disk_total) * 100
})
</script>

<template>
  <router-link :to="`/server/${client.id}`" class="server-card card">
    <div class="card-header">
      <span class="status-dot" :class="client.online ? 'online' : 'offline'"></span>
      <h3 class="server-name">{{ client.name }}</h3>
    </div>

    <div class="card-meta">
      <span class="os">{{ client.os || 'Unknown OS' }}</span>
      <span class="region">{{ client.region || 'Unknown' }}</span>
    </div>

    <div v-if="client.online && client.status" class="card-stats">
      <div class="stat-row">
        <span class="stat-label">CPU</span>
        <div class="progress-bar flex-1">
          <div 
            class="progress" 
            :class="{ low: cpuPercent < 50, medium: cpuPercent >= 50 && cpuPercent < 80, high: cpuPercent >= 80 }"
            :style="{ width: cpuPercent + '%' }"
          ></div>
        </div>
        <span class="stat-value">{{ cpuPercent.toFixed(0) }}%</span>
      </div>

      <div class="stat-row">
        <span class="stat-label">RAM</span>
        <div class="progress-bar flex-1">
          <div 
            class="progress"
            :class="{ low: ramPercent < 50, medium: ramPercent >= 50 && ramPercent < 80, high: ramPercent >= 80 }"
            :style="{ width: ramPercent + '%' }"
          ></div>
        </div>
        <span class="stat-value">{{ ramPercent.toFixed(0) }}%</span>
      </div>

      <div class="stat-row">
        <span class="stat-label">Disk</span>
        <div class="progress-bar flex-1">
          <div 
            class="progress"
            :class="{ low: diskPercent < 50, medium: diskPercent >= 50 && diskPercent < 80, high: diskPercent >= 80 }"
            :style="{ width: diskPercent + '%' }"
          ></div>
        </div>
        <span class="stat-value">{{ diskPercent.toFixed(0) }}%</span>
      </div>

      <div class="network-row">
        <span>↓ {{ formatBytes(client.status.net_in) }}/s</span>
        <span>↑ {{ formatBytes(client.status.net_out) }}/s</span>
      </div>
    </div>

    <div v-else class="card-offline">
      <span>Offline</span>
    </div>
  </router-link>
</template>

<style scoped>
.server-card {
  display: block;
  padding: 1.25rem;
  transition: transform 0.2s, box-shadow 0.2s;
  cursor: pointer;
  text-decoration: none;
  color: inherit;
}

.server-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px -5px rgba(0, 0, 0, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
}

.server-name {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-meta {
  display: flex;
  gap: 1rem;
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-bottom: 1rem;
}

.card-stats {
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
}

.stat-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.stat-label {
  width: 32px;
  font-size: 0.75rem;
  color: var(--text-muted);
}

.stat-value {
  width: 40px;
  text-align: right;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.flex-1 {
  flex: 1;
}

.network-row {
  display: flex;
  justify-content: space-between;
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-top: 0.5rem;
  padding-top: 0.5rem;
  border-top: 1px solid var(--border);
}

.card-offline {
  text-align: center;
  padding: 1rem 0;
  color: var(--text-muted);
  font-size: 0.875rem;
}
</style>
