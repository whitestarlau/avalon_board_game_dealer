<script setup>
import { ref, onMounted } from 'vue'
import PlayerSlot from '../components/PlayerSlot.vue'
import HistoryTable from '../components/HistoryTable.vue'

const currentGame = ref(null)
const history = ref([])
const loading = ref(true)
const showCurrentGame = ref(false)
const setupPlayerCount = ref(7)
const showSkillInfo = ref(true)

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

async function startGame() {
  try {
    const res = await fetch(`${BASE_URL}/api/new_game?count=${setupPlayerCount.value}`, {
      method: 'POST',
    })
    if (!res.ok) return
    await fetchCurrentGame()
  } catch (e) {
    console.error('Failed to start game', e)
  }
}

function newGame() {
  if (!confirm('确定开始新一局？当前游戏将结束。')) return
  startGame()
}

async function fetchSkillInfoStatus() {
  try {
    const res = await fetch(`${BASE_URL}/api/admin/skill_info_status`)
    const data = await res.json()
    showSkillInfo.value = data.show_skill_info
  } catch (e) {
    console.error('Failed to fetch skill info status', e)
  }
}

async function toggleSkillInfo() {
  try {
    const res = await fetch(`${BASE_URL}/api/admin/toggle_skill_info`, {
      method: 'POST',
    })
    const data = await res.json()
    showSkillInfo.value = data.show_skill_info
  } catch (e) {
    console.error('Failed to toggle skill info', e)
  }
}

onMounted(async () => {
  await Promise.all([fetchCurrentGame(), fetchHistory(), fetchSkillInfoStatus()])
  loading.value = false
})
</script>

<template>
  <div class="admin">
    <h1>管理后台</h1>

    <div v-if="loading">加载中...</div>

    <template v-else>
      <section v-if="!currentGame || currentGame.player_count === 0" class="setup-game">
        <h2>开始新游戏</h2>
        <p>选择玩家人数：</p>
        <div class="setup-count-grid">
          <button
            v-for="n in [5, 6, 7, 8, 9, 10]"
            :key="n"
            :class="['setup-count-btn', { selected: setupPlayerCount === n }]"
            @click="setupPlayerCount = n"
          >
            {{ n }} 人
          </button>
        </div>
        <button class="start-game-btn" @click="startGame">开始游戏</button>
      </section>

      <section v-else class="current-game">
        <div class="section-header">
          <h2>当前局（{{ currentGame.player_count }} 人）</h2>
        </div>
        <div class="switch-row">
          <label class="switch">
            <input type="checkbox" :checked="showCurrentGame" @change="showCurrentGame = !showCurrentGame">
            <span class="slider"></span>
          </label>
          <span>显示详情（开启后可查看各玩家角色分配）</span>
        </div>
        <div class="switch-row">
          <label class="switch">
            <input type="checkbox" :checked="showSkillInfo" @change="toggleSkillInfo">
            <span class="slider"></span>
          </label>
          <span>技能信息（关闭后玩家只看到角色名，不展示技能详情，如梅林不再获知邪恶方号码）</span>
        </div>
        <div v-if="!currentGame.game_over" class="not-over">
          <p>
            游戏进行中，{{ currentGame.unready_numbers.length }} 名玩家未就绪：
            {{ currentGame.unready_numbers.join('、') }} 号
          </p>
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

.switch-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.switch-row span {
  font-size: 0.9rem;
  color: var(--color-text);
  opacity: 0.85;
}

.switch {
  position: relative;
  display: inline-block;
  width: 36px;
  height: 20px;
  flex-shrink: 0;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #ccc;
  transition: 0.3s;
  border-radius: 20px;
}

.slider::before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 2px;
  bottom: 2px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

.switch input:checked + .slider {
  background-color: #42b883;
}

.switch input:checked + .slider::before {
  transform: translateX(16px);
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

.setup-game {
  text-align: center;
  padding: 2rem;
  border: 2px dashed var(--color-border);
  border-radius: 12px;
}

.setup-game h2 {
  border: none;
}

.setup-count-grid {
  display: flex;
  justify-content: center;
  gap: 1rem;
  margin: 1.5rem 0;
  flex-wrap: wrap;
}

.setup-count-btn {
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

.setup-count-btn:hover {
  border-color: var(--color-heading);
  transform: scale(1.05);
}

.setup-count-btn.selected {
  border-color: #e67e22;
  background: #e67e22;
  color: white;
}

.start-game-btn {
  display: block;
  margin: 1.5rem auto;
  padding: 1rem 3rem;
  font-size: 1.2rem;
  border: none;
  border-radius: 8px;
  background: #e67e22;
  color: white;
  cursor: pointer;
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
