<script setup lang="ts">
import { ref, onMounted } from "vue";
import { getTauriVersion, getVersion } from '@tauri-apps/api/app';

const appVersion = ref("");
const tauriVersion = ref("");

onMounted(async () => {
  appVersion.value = await getVersion();
  tauriVersion.value = await getTauriVersion();
});
</script>

<template>
  <footer class="footer">
    <div class="footer-content">
      <span>Created by Tariq Dinmohamed</span>
      <span>|</span>
      <a href="https://github.com/flixis" target="_blank" rel="noopener noreferrer">GitHub</a>
      <span>|</span>
      <span>App v{{ appVersion }}</span>
      <span>|</span>
      <span>Tauri v{{ tauriVersion }}</span>
    </div>
  </footer>
</template>

<style scoped>
.footer {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  background-color: #f8f8f8;
  border-top: 1px solid #ddd;
  padding: 1rem;
  text-align: center;
  font-size: 0.9rem;
}

.footer-content {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
  max-width: 1200px;
  margin: 0 auto;
}

.footer-content span {
  color: #666;
}

.footer-content a {
  color: #396cd8;
  text-decoration: none;
}

.footer-content a:hover {
  text-decoration: underline;
}
</style>
