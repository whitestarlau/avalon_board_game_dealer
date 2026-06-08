<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import RoleCard from '../components/RoleCard.vue'

const playerCount = ref(0)
const loading = ref(true)
const selectedNumber = ref(null)
const isReady = ref(false)
const waiting = ref(false)
const gameOver = ref(false)
const roleData = ref(null)
let eventSource = null

const BASE_URL = window.location.origin

async function checkGameStatus() {
  try {
    const res = await fetch(`${BASE_URL}/api/admin/current_game`)
    const data = await res.json()
    playerCount.value = data.player_count || 0
    if (data.game_over) {
      gameOver.value = true
    }
  } catch (e) {
    // ignore
  }
  loading.value = false
}

async function ready() {
  if (!selectedNumber.value) return
  waiting.value = true

  try {
    const res = await fetch(`${BASE_URL}/api/ready?number=${selectedNumber.value}`, {
      method: 'GET',
    })
    if (!res.ok) {
      const data = await res.json()
      waiting.value = false
      if (data.error === 'ALREADY_READY' || data.error === 'GAME_ALREADY_OVER') {
        isReady.value = true
        startPolling()
        return
      }
      return
    }
    isReady.value = true
    startPolling()
  } catch (e) {
    waiting.value = false
  }
}

function startPolling() {
  const es = new EventSource(`${BASE_URL}/api/sse?number=${selectedNumber.value}`)
  es.onmessage = (event) => {
    const data = JSON.parse(event.data)
    if (data.ready) {
      gameOver.value = true
      roleData.value = data
      es.close()
    }
  }
  es.onerror = () => {
    // EventSource auto-reconnects on failure
  }
  eventSource = es
}

function stopPolling() {
  if (eventSource) {
    eventSource.close()
    eventSource = null
  }
}

onMounted(() => {
  checkGameStatus()
})

onUnmounted(() => {
  stopPolling()
})
</script>

<template>
  <div class="home">
    <h1>阿瓦隆发牌工具</h1>

    <div v-if="loading" class="loading">加载中...</div>

    <div v-else-if="playerCount === 0" class="waiting">
      <h2>等待管理员开始游戏</h2>
      <p>请管理员前往 <router-link to="/admin">管理后台</router-link> 设置玩家人数并开始游戏。</p>
    </div>

    <div v-else-if="!isReady" class="setup">
      <h2>选择你的玩家编号（共 {{ playerCount }} 人）</h2>
      <div class="number-grid">
        <button
          v-for="n in playerCount"
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
      <p class="selection-info">你选择了 <strong>{{ selectedNumber }} 号</strong></p>
      <p>已就绪，请等待所有 {{ playerCount }} 名玩家准备</p>
      <div class="spinner"></div>
    </div>

    <div v-else class="result">
      <h2>你的角色</h2>
      <p class="selection-info">你选择了 <strong>{{ selectedNumber }} 号</strong></p>
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

.loading {
  text-align: center;
  padding: 3rem;
  color: var(--color-text);
  opacity: 0.6;
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

.selection-info {
  font-size: 1.1rem;
  margin-bottom: 1rem;
  color: var(--color-text);
  opacity: 0.9;
}
.selection-info strong {
  color: #42b883;
  font-size: 1.3rem;
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
