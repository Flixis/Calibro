<script setup lang="ts">
import CalibrationForm from '../components/CalibrationForm.vue';
import { ref, onMounted } from 'vue';
import type { CalibrationData } from '../types';
import { useRoute } from 'vue-router';
import { useCalibrations } from '../composables/useCalibrations';

const route = useRoute();
const { calibrations, loadCalibrations } = useCalibrations();
const selectedCalibration = ref<CalibrationData | undefined>(undefined);
const isEditMode = ref(false);

onMounted(async () => {
  const editCertNumber = route.query.edit as string;
  if (editCertNumber) {
    await loadCalibrations();
    const calibrationToEdit = calibrations.value.find(
      cal => cal.certificate_number === editCertNumber
    );
    if (calibrationToEdit) {
      selectedCalibration.value = calibrationToEdit;
      isEditMode.value = true;
    }
  }
});

async function onSaved() {
  // Navigate to past calibrations after saving
  window.location.href = '#/past';
}

function cancelEdit() {
  selectedCalibration.value = undefined;
  isEditMode.value = false;
}
</script>

<template>
  <main class="container">
    <h1>New Calibration</h1>
    <CalibrationForm 
      :initial-data="selectedCalibration"
      :is-edit-mode="isEditMode"
      @saved="onSaved"
      @cancelled="cancelEdit"
    />
  </main>
</template>

<style scoped>
.container {
  margin: 0;
  padding: 10vh 2rem 5rem;
  max-width: 800px;
  margin: 0 auto;
}

h1 {
  text-align: center;
  margin-bottom: 2rem;
}
</style>
