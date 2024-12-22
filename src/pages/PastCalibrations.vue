<script setup lang="ts">
import { onMounted } from "vue";
import CalibrationTable from '../components/CalibrationTable.vue';
import { useCalibrations } from '../composables/useCalibrations';
import type { CalibrationData } from '../types';
import { useRouter } from 'vue-router';

const router = useRouter();
const { calibrations, loadCalibrations, openCalibroFolder } = useCalibrations();

onMounted(async () => {
  await loadCalibrations();
});

function startEdit(calibration: CalibrationData) {
  router.push({ 
    path: '/new',
    query: { edit: calibration.certificate_number }
  });
}
</script>

<template>
  <main class="container">
    <h1>Past Calibrations</h1>
    <button @click="openCalibroFolder" class="folder-button">
      Open Data & Certificates Folder
    </button>

    <CalibrationTable 
      :calibrations="calibrations"
      @edit="startEdit"
    />
  </main>
</template>

<style scoped>
.container {
  margin: 0;
  padding: 10vh 2rem 5rem;
  max-width: 1200px;
  margin: 0 auto;
}

h1 {
  text-align: center;
  margin-bottom: 2rem;
}

.folder-button {
  background-color: #4CAF50;
  color: white;
  border: none;
  padding: 0.8em 1.6em;
  margin-bottom: 2rem;
  display: block;
  margin-left: auto;
  margin-right: auto;
}

.folder-button:hover {
  background-color: #45a049;
  border-color: transparent;
}
</style>
