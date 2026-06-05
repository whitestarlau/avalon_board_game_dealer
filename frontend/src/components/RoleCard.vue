<script setup>
defineProps({
  role: {
    type: Object,
    required: true
  }
})
</script>

<template>
  <div class="role-card">
    <div class="role-name">{{ role.role }}</div>
    <div class="role-desc">{{ role.role_des }}</div>
    <div v-if="role.skill_des" class="skill-desc">
      <strong>技能信息：</strong>{{ role.skill_des }}
    </div>
  </div>
</template>

<style scoped>
.role-card {
  display: inline-block;
  padding: 2rem;
  border: 2px solid var(--color-border);
  border-radius: 12px;
  background: var(--color-background-soft);
  max-width: 400px;
}

.role-name {
  font-size: 2rem;
  font-weight: bold;
  margin-bottom: 1rem;
}

.role-desc {
  font-size: 1.1rem;
  line-height: 1.6;
  margin-bottom: 1rem;
}

.skill-desc {
  text-align: left;
  padding: 1rem;
  background: var(--color-background-mute);
  border-radius: 8px;
}
</style>
