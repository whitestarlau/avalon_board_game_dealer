<script setup>
defineProps({
  player: {
    type: Object,
    required: true
  }
})
</script>

<template>
  <div :class="['player-slot', player.faction]">
    <span class="player-number">{{ player.number }} 号</span>
    <span class="player-role">{{ player.role }}</span>
    <span :class="['faction-badge', player.faction]">
      {{ player.faction === 'good' ? '正义' : '邪恶' }}
    </span>
  </div>
</template>

<style scoped>
.player-slot {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem 1.5rem;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  margin-bottom: 0.5rem;
}

.player-slot.good {
  border-left: 4px solid #42b883;
}

.player-slot.evil {
  border-left: 4px solid #e74c3c;
}

.player-number {
  font-weight: bold;
  min-width: 60px;
}

.player-role {
  flex: 1;
}

.faction-badge {
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: 0.85rem;
  font-weight: bold;
}

.faction-badge.good {
  background: #42b883;
  color: white;
}

.faction-badge.evil {
  background: #e74c3c;
  color: white;
}
</style>
