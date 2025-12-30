import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import api from '@/api'

interface User {
    id: string
    username: string
}

export const useAuthStore = defineStore('auth', () => {
    const user = ref<User | null>(null)
    const token = ref<string | null>(localStorage.getItem('token'))

    const isAuthenticated = computed(() => !!token.value)

    async function login(username: string, password: string) {
        const response = await api.post('/api/login', { username, password })
        const data = response.data

        token.value = data.token
        user.value = data.user
        localStorage.setItem('token', data.token)

        return data
    }

    async function logout() {
        await api.get('/api/logout')
        token.value = null
        user.value = null
        localStorage.removeItem('token')
    }

    async function fetchUser() {
        if (!token.value) return null

        try {
            const response = await api.get('/api/me')
            user.value = response.data
            return user.value
        } catch {
            token.value = null
            user.value = null
            localStorage.removeItem('token')
            return null
        }
    }

    return {
        user,
        token,
        isAuthenticated,
        login,
        logout,
        fetchUser
    }
})
