<script setup lang="ts">
import { ref, onMounted } from "vue";
import CalibrationForm from './components/CalibrationForm.vue';
import CalibrationTable from './components/CalibrationTable.vue';
import Footer from './components/Footer.vue';
import { useCalibrations } from './composables/useCalibrations';
import type { CalibrationData } from './types';

const { calibrations, loadCalibrations, openCalibroFolder } = useCalibrations();
const isEditMode = ref(false);
const selectedCalibration = ref<CalibrationData | undefined>(undefined);

onMounted(async () => {
  await loadCalibrations();
});

function startEdit(calibration: CalibrationData) {
  selectedCalibration.value = calibration;
  isEditMode.value = true;
  // Scroll to form
  document.querySelector('.calibration-form')?.scrollIntoView({ behavior: 'smooth' });
}

function cancelEdit() {
  selectedCalibration.value = undefined;
  isEditMode.value = false;
}

async function onSaved() {
  await loadCalibrations();
  selectedCalibration.value = undefined;
  isEditMode.value = false;
}
</script>

<template>
  <Suspense>
    <main class="container">
      <h1>Calibration Certificate System</h1>
      <button @click="openCalibroFolder" class="folder-button">
        Open Data & Certificates Folder
      </button>

      <CalibrationForm 
        :initial-data="selectedCalibration"
        :is-edit-mode="isEditMode"
        @saved="onSaved"
        @cancelled="cancelEdit"
      />

      <CalibrationTable 
        :calibrations="calibrations"
        @edit="startEdit"
      />
    </main>
    <template #fallback>
      <div class="loading">Loading...</div>
    </template>
  </Suspense>
  <Footer />
</template>

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

.container {
  margin: 0;
  padding: 10vh 2rem 5rem;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 2rem;
  max-width: 1200px;
  margin: 0 auto;
  text-align: center;
}

@media (min-width: 1024px) {
  .container {
    grid-template-columns: repeat(2, 1fr);
  }
  
  .container h1 {
    grid-column: 1 / -1;
  }
}

h1 {
  text-align: center;
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

input,
button {
  outline: none;
}

.folder-button {
  background-color: #4CAF50;
  color: white;
  border: none;
  padding: 0.8em 1.6em;
  margin-bottom: 1em;
  grid-column: 1 / -1;
}

.folder-button:hover {
  background-color: #45a049;
  border-color: transparent;
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
