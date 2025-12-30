<script setup lang="ts">
import { onMounted, onUnmounted, computed } from 'vue'
import { useServersStore } from '@/stores/servers'
import { useAuthStore } from '@/stores/auth'
import ServerCard from '@/components/ServerCard.vue'

const serversStore = useServersStore()
const authStore = useAuthStore()

const onlineCount = computed(() => 
  serversStore.clients.filter(c => c.online).length
)

const totalCount = computed(() => serversStore.clients.length)

onMounted(() => {
  serversStore.fetchClients()
  serversStore.startAutoRefresh()
  authStore.fetchUser()
})

onUnmounted(() => {
  serversStore.stopAutoRefresh()
})
</script>

<template>
  <div class="dashboard-page page">
    <!-- Header -->
    <header class="header">
      <div class="container header-content">
        <h1 class="logo">Vanmoi</h1>
        <nav class="nav">
          <router-link to="/" class="active">Dashboard</router-link>
          <router-link v-if="authStore.isAuthenticated" to="/admin">Admin</router-link>
          <router-link v-else to="/login">Login</router-link>
        </nav>
      </div>
    </header>

    <!-- Main Content -->
    <main class="container">
      <!-- Stats Section -->
      <section class="stats-section fade-in">
        <div class="stats-grid grid grid-4">
          <div class="stat-card card">
            <div class="stat-label">Total Servers</div>
            <div class="stat-value">{{ totalCount }}</div>
          </div>
          <div class="stat-card card">
            <div class="stat-label">Online</div>
            <div class="stat-value status-online">{{ onlineCount }}</div>
          </div>
          <div class="stat-card card">
            <div class="stat-label">Offline</div>
            <div class="stat-value status-offline">{{ totalCount - onlineCount }}</div>
          </div>
          <div class="stat-card card">
            <div class="stat-label">Uptime Rate</div>
            <div class="stat-value">
              {{ totalCount > 0 ? Math.round(onlineCount / totalCount * 100) : 0 }}%
            </div>
          </div>
        </div>
      </section>

      <!-- Servers Grid -->
      <section class="servers-section">
        <h2 class="section-title">Servers</h2>
        
        <div v-if="serversStore.loading && serversStore.clients.length === 0" class="loading-container">
          <div class="loading">Loading servers...</div>
        </div>

        <div v-else-if="serversStore.clients.length === 0" class="empty-state">
          <p>No servers found.</p>
          <p class="text-muted">Add a server from the admin panel.</p>
        </div>

        <div v-else class="servers-grid grid grid-3">
          <ServerCard 
            v-for="client in serversStore.clients" 
            :key="client.id" 
            :client="client"
            class="fade-in"
          />
        </div>
      </section>
    </main>
  </div>
</template>

<style scoped>
.stats-section {
  margin: 2rem 0;
}

.stat-card {
  text-align: center;
}

.stat-label {
  font-size: 0.875rem;
  color: var(--text-secondary);
  margin-bottom: 0.5rem;
}

.stat-value {
  font-size: 2rem;
  font-weight: 700;
}

.servers-section {
  margin-top: 2rem;
}

.section-title {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 1.5rem;
  color: var(--text-primary);
}

.loading-container, .empty-state {
  text-align: center;
  padding: 4rem 2rem;
  color: var(--text-secondary);
}

.text-muted {
  color: var(--text-muted);
  margin-top: 0.5rem;
}

.servers-grid {
  gap: 1rem;
}
</style>
