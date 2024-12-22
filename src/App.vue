<script setup lang="ts">
import Footer from './components/Footer.vue';
</script>

<template>
  <nav>
    <router-link to="/new" class="nav-link">New Calibration</router-link>
    <router-link to="/past" class="nav-link">Past Calibrations</router-link>
  </nav>

  <Suspense>
    <router-view></router-view>
    <template #fallback>
      <div class="loading">Loading...</div>
    </template>
  </Suspense>
  
  <Footer />
</template>

<style scoped>
nav {
  padding: 1rem;
  background-color: #f8f9fa;
  text-align: center;
  border-bottom: 1px solid #dee2e6;
}

.nav-link {
  display: inline-block;
  padding: 0.5rem 1rem;
  margin: 0 0.5rem;
  color: #495057;
  text-decoration: none;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.nav-link:hover {
  background-color: #e9ecef;
}

.nav-link.router-link-active {
  background-color: #007bff;
  color: white;
}
</style>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;
  color: #000000;
  background-color: #ffffff;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  outline: none;
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}

button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

.loading {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100vh;
  font-size: 1.2em;
  color: #666;
}
</style>
