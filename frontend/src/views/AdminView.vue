<script setup>
import { ref, onMounted } from 'vue'
import PlayerSlot from '../components/PlayerSlot.vue'
import HistoryTable from '../components/HistoryTable.vue'

const currentGame = ref(null)
const history = ref([])
const loading = ref(true)

const BASE_URL = window.location.origin

async function fetchCurrentGame() {
  try {
    const res = await fetch(`${BASE_URL}/api/admin/current_game`)
    currentGame.value = await res.json()
  } catch (e) {
    console.error('Failed to fetch current game', e)
  }
}

async function fetchHistory() {
  try {
    const res = await fetch(`${BASE_URL}/api/admin/history`)
    const data = await res.json()
    history.value = data.games
  } catch (e) {
    console.error('Failed to fetch history', e)
  }
}

async function newGame() {
  if (!confirm('确定开始新一局？当前还未就绪的玩家信息将丢失。')) return
  try {
    await fetch(`${BASE_URL}/api/new_game`, { method: 'POST' })
    currentGame.value = null
    await fetchHistory()
  } catch (e) {
    console.error('Failed to start new game', e)
  }
}

onMounted(async () => {
  await Promise.all([fetchCurrentGame(), fetchHistory()])
  loading.value = false
})
</script>

<template>
  <div class="admin">
    <h1>管理后台</h1>

    <div v-if="loading">加载中...</div>

    <template v-else>
      <section class="current-game">
        <h2>当前局</h2>
        <div v-if="!currentGame || !currentGame.game_over" class="not-over">
          <p v-if="currentGame">
            游戏进行中，{{ currentGame.unready_numbers.length }} 名玩家未就绪：
            {{ currentGame.unready_numbers.join('、') }} 号
          </p>
          <p v-else>无进行中的游戏</p>
        </div>
        <div v-else class="game-result">
          <h3>全员就绪！</h3>
          <div class="players">
            <PlayerSlot
              v-for="player in currentGame.players"
              :key="player.number"
              :player="player"
            />
          </div>
        </div>
      </section>

      <section class="history">
        <h2>历史记录</h2>
        <HistoryTable :games="history" />
      </section>

      <div class="actions">
        <button class="new-game-btn" @click="newGame">开始新一局</button>
        <router-link to="/" class="back-link">返回首页</router-link>
      </div>
    </template>
  </div>
</template>

<style scoped>
.admin {
  padding: 2rem;
  max-width: 800px;
  margin: 0 auto;
}

h1 {
  text-align: center;
  margin-bottom: 2rem;
}

section {
  margin-bottom: 2rem;
}

section h2 {
  border-bottom: 2px solid var(--color-border);
  padding-bottom: 0.5rem;
  margin-bottom: 1rem;
}

.not-over {
  text-align: center;
  padding: 2rem;
  color: var(--color-text);
  opacity: 0.7;
}

.game-result h3 {
  text-align: center;
  margin-bottom: 1rem;
}

.actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
  margin-top: 2rem;
}

.new-game-btn {
  padding: 0.75rem 2rem;
  font-size: 1.1rem;
  border: none;
  border-radius: 8px;
  background: #e74c3c;
  color: white;
  cursor: pointer;
}

.new-game-btn:hover {
  opacity: 0.9;
}

.back-link {
  display: inline-flex;
  align-items: center;
  padding: 0.75rem 2rem;
  font-size: 1.1rem;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  text-decoration: none;
  color: var(--color-text);
}
</style>
