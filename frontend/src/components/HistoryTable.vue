<script setup>
import { ref } from 'vue'

defineProps({
  games: {
    type: Array,
    required: true
  }
})

const expandedGames = ref({})
</script>

<template>
  <div v-if="games.length === 0" class="empty">暂无游戏记录</div>
  <div v-else v-for="game in games" :key="game.id" class="game-entry">
    <div class="game-header" @click="expandedGames[game.id] = !expandedGames[game.id]">
      <h3>第 {{ game.id }} 局 — {{ new Date(game.timestamp).toLocaleString() }}</h3>
      <span class="expand-icon">{{ expandedGames[game.id] ? '▼' : '▶' }}</span>
    </div>
    <div v-if="expandedGames[game.id]" class="player-list">
      <div
        v-for="player in game.players"
        :key="player.number"
        :class="['history-player', player.faction]"
      >
        <span>{{ player.number }} 号</span>
        <span class="role-name">{{ player.role }}</span>
        <span :class="['mini-badge', player.faction]">
          {{ player.faction === 'good' ? '正' : '邪' }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.empty {
  text-align: center;
  padding: 2rem;
  color: var(--color-text);
  opacity: 0.6;
}

.game-entry {
  margin-bottom: 1.5rem;
  padding: 1rem;
  border: 1px solid var(--color-border);
  border-radius: 8px;
}

.game-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  user-select: none;
}

.game-header h3 {
  margin-bottom: 0;
}

.expand-icon {
  font-size: 0.8rem;
  opacity: 0.5;
}

.game-entry h3 {
  margin-bottom: 0.5rem;
}

.player-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.history-player {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 0.9rem;
}

.history-player.good {
  border-left: 3px solid #42b883;
}

.history-player.evil {
  border-left: 3px solid #e74c3c;
}

.mini-badge {
  padding: 0.1rem 0.4rem;
  border-radius: 8px;
  font-size: 0.75rem;
  font-weight: bold;
  color: white;
}

.mini-badge.good { background: #42b883; }
.mini-badge.evil { background: #e74c3c; }

.role-name {
  font-weight: bold;
}
</style>
