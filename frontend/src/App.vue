<script setup>
import { RouterLink, RouterView } from 'vue-router'
</script>

<template>
  <header>
    <nav>
      <RouterLink to="/">首页</RouterLink>
      <RouterLink to="/admin">管理后台</RouterLink>
      <RouterLink to="/about">关于</RouterLink>
    </nav>
  </header>

  <RouterView />
</template>

<style scoped>
header {
  line-height: 1.5;
}

nav {
  width: 100%;
  font-size: 1rem;
  text-align: center;
  padding: 1rem 0;
  border-bottom: 1px solid var(--color-border);
}

nav a {
  display: inline-block;
  padding: 0 1rem;
  color: var(--color-text);
  text-decoration: none;
}

nav a.router-link-exact-active {
  color: #42b883;
}

nav a:hover {
  background-color: transparent;
}
</style>
