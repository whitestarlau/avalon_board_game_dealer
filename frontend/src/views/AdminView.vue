<script setup>
import { ref, onMounted } from 'vue'
import PlayerSlot from '../components/PlayerSlot.vue'
import HistoryTable from '../components/HistoryTable.vue'

const currentGame = ref(null)
const history = ref([])
const loading = ref(true)
const showCurrentGame = ref(false)

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
        <div class="section-header">
          <h2>当前局</h2>
          <button class="toggle-btn" @click="showCurrentGame = !showCurrentGame">
            {{ showCurrentGame ? '隐藏详情' : '显示详情' }}
          </button>
        </div>
        <div v-if="!currentGame || !currentGame.game_over" class="not-over">
          <p v-if="currentGame">
            游戏进行中，{{ currentGame.unready_numbers.length }} 名玩家未就绪：
            {{ currentGame.unready_numbers.join('、') }} 号
          </p>
          <p v-else>无进行中的游戏</p>
        </div>
        <div v-else-if="showCurrentGame" class="game-result">
          <h3>全员就绪！</h3>
          <div class="players">
            <PlayerSlot
              v-for="player in currentGame.players"
              :key="player.number"
              :player="player"
            />
          </div>
        </div>
        <div v-else class="game-result">
          <p class="hidden-hint">所有人都已就绪，点击"显示详情"查看角色分配。</p>
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

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 2px solid var(--color-border);
  padding-bottom: 0.5rem;
  margin-bottom: 1rem;
}

.section-header h2 {
  border: none;
  padding: 0;
  margin: 0;
}

.toggle-btn {
  padding: 0.4rem 1rem;
  font-size: 0.9rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-background);
  color: var(--color-text);
  cursor: pointer;
}

.toggle-btn:hover {
  background: var(--color-background-mute);
}

.hidden-hint {
  text-align: center;
  padding: 1rem;
  color: var(--color-text);
  opacity: 0.6;
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
