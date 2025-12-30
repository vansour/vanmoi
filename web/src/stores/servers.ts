import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api'

interface ClientStatus {
    cpu: number
    ram: number
    ram_total: number
    disk: number
    disk_total: number
    net_in: number
    net_out: number
    load: number
    uptime: number
}

interface Client {
    id: string
    name: string
    cpu_name: string
    arch: string
    cpu_cores: number
    os: string
    region: string
    public_remark: string
    mem_total: number
    disk_total: number
    group_name: string
    online: boolean
    last_seen_at: string | null
    status?: ClientStatus
}

export const useServersStore = defineStore('servers', () => {
    const clients = ref<Client[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)

    async function fetchClients() {
        loading.value = true
        error.value = null

        try {
            const response = await api.get('/api/clients')
            clients.value = response.data.clients || []
        } catch (e: any) {
            error.value = e.message || 'Failed to fetch clients'
        } finally {
            loading.value = false
        }
    }

    function getClient(id: string) {
        return clients.value.find(c => c.id === id)
    }

    // Auto-refresh every 5 seconds
    let refreshInterval: number | null = null

    function startAutoRefresh() {
        if (refreshInterval) return
        refreshInterval = window.setInterval(fetchClients, 5000)
    }

    function stopAutoRefresh() {
        if (refreshInterval) {
            clearInterval(refreshInterval)
            refreshInterval = null
        }
    }

    return {
        clients,
        loading,
        error,
        fetchClients,
        getClient,
        startAutoRefresh,
        stopAutoRefresh
    }
})
