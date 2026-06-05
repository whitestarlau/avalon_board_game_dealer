<script setup>
import { ref, onUnmounted } from 'vue'
import RoleCard from '../components/RoleCard.vue'

const selectedNumber = ref(null)
const isReady = ref(false)
const waiting = ref(false)
const gameOver = ref(false)
const roleData = ref(null)
let pollTimer = null

const BASE_URL = window.location.origin

async function ready() {
  if (!selectedNumber.value) return
  waiting.value = true

  try {
    const res = await fetch(`${BASE_URL}/api/ready?number=${selectedNumber.value}`, {
      method: 'GET',
    })
    if (!res.ok) {
      const text = await res.text()
      alert(text)
      waiting.value = false
      return
    }
    isReady.value = true
    startPolling()
  } catch (e) {
    alert('Connection failed')
    waiting.value = false
  }
}

function startPolling() {
  pollTimer = setInterval(async () => {
    try {
      const res = await fetch(`${BASE_URL}/api/poll_player_role?number=${selectedNumber.value}`, {
        method: 'POST',
      })
      const data = await res.json()
      if (data.ready) {
        gameOver.value = true
        roleData.value = data
        stopPolling()
      }
    } catch (e) {
      // ignore polling errors
    }
  }, 3000)
}

function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

onUnmounted(() => {
  stopPolling()
})
</script>

<template>
  <div class="home">
    <h1>阿瓦隆发牌工具</h1>

    <div v-if="!isReady" class="setup">
      <h2>选择你的玩家编号</h2>
      <div class="number-grid">
        <button
          v-for="n in 7"
          :key="n"
          :class="['num-btn', { selected: selectedNumber === n }]"
          @click="selectedNumber = n"
        >
          {{ n }} 号
        </button>
      </div>
      <button
        class="ready-btn"
        :disabled="!selectedNumber || waiting"
        @click="ready"
      >
        {{ waiting ? '准备中...' : '准备' }}
      </button>
    </div>

    <div v-else-if="!gameOver" class="waiting">
      <h2>等待其他玩家...</h2>
      <p>已就绪，请等待所有 7 名玩家准备</p>
      <div class="spinner"></div>
    </div>

    <div v-else class="result">
      <h2>你的角色</h2>
      <RoleCard v-if="roleData" :role="roleData" />
    </div>

    <div class="footer-links">
      <router-link to="/admin">管理后台</router-link>
    </div>
  </div>
</template>

<style scoped>
.home {
  text-align: center;
  padding: 2rem;
}

.number-grid {
  display: flex;
  justify-content: center;
  gap: 1rem;
  margin: 2rem 0;
  flex-wrap: wrap;
}

.num-btn {
  width: 80px;
  height: 80px;
  font-size: 1.2rem;
  border: 2px solid var(--color-border);
  border-radius: 8px;
  cursor: pointer;
  background: var(--color-background);
  color: var(--color-text);
  transition: all 0.2s;
}

.num-btn:hover {
  border-color: var(--color-heading);
  transform: scale(1.05);
}

.num-btn.selected {
  border-color: #42b883;
  background: #42b883;
  color: white;
}

.ready-btn {
  display: block;
  margin: 2rem auto;
  padding: 1rem 3rem;
  font-size: 1.3rem;
  border: none;
  border-radius: 8px;
  background: #42b883;
  color: white;
  cursor: pointer;
}

.ready-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.waiting {
  margin-top: 3rem;
}

.spinner {
  width: 40px;
  height: 40px;
  margin: 2rem auto;
  border: 4px solid var(--color-border);
  border-top-color: #42b883;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.footer-links {
  margin-top: 3rem;
}

.footer-links a {
  color: var(--color-text);
  text-decoration: underline;
}

.result {
  margin-top: 2rem;
}
</style>
