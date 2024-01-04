<script setup>
import { reactive, onMounted, ref } from 'vue'
// import { tokenStore } from '../store.js'

const userEmail = ref("");
const userPassword = ref("");

const signedToken = ref({});

async function ready(event) {
  const data = await fetch(
    `http://127.0.0.1:3004/ready?number=1`,
    {
      method: "get",
    }
  ).then(rsp => rsp.json())


  console.log("setToken save to localStorage:" + jwtStr);
  localStorage.setItem("JwtKey", jwtStr);
}
</script>

<template>
  <main>
    <title>Choose number to ready.</title>
    <div>
      <button @click="ready">ready</button>
    </div>
  </main>
</template>
