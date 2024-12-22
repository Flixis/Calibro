<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import type { CalibrationData } from '../types';

const props = defineProps<{
  calibrations: CalibrationData[];
}>();

const emit = defineEmits<{
  (e: 'edit', calibration: CalibrationData): void;
}>();

async function generatePdf(certificateNumber: string) {
  try {
    await invoke('generate_pdf', { certificateNumber });
  } catch (error) {
    console.error('Error generating PDF:', error);
  }
}
</script>

<template>
  <div class="card" v-if="calibrations.length > 0">
    <h2>Recent Calibrations</h2>
    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>Certificate #</th>
            <th>Company</th>
            <th>Customer</th>
            <th>Model</th>
            <th>Date</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="cal in calibrations" :key="cal.certificate_number">
            <td>{{ cal.certificate_number }}</td>
            <td>{{ cal.company_name }}</td>
            <td>{{ cal.customer || '-' }}</td>
            <td>{{ cal.model_details }}</td>
            <td>{{ cal.calibration_date }}</td>
            <td class="action-buttons">
              <button @click="generatePdf(cal.certificate_number)" class="action-btn">
                Generate PDF
              </button>
              <button @click="emit('edit', cal)" class="action-btn edit-btn">
                Edit
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.table-container {
  overflow-x: auto;
}

table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 1em;
}

th, td {
  padding: 0.75em;
  text-align: left;
  border-bottom: 1px solid #ddd;
}

th {
  background: #f5f5f5;
  font-weight: 600;
}

tr:hover {
  background: #f8f8f8;
}

.action-buttons {
  display: flex;
  gap: 0.5em;
}

.action-btn {
  background-color: #007bff;
  color: white;
  border: none;
  padding: 0.4em 0.8em;
  border-radius: 4px;
}

.action-btn:hover {
  background-color: #0056b3;
  border-color: transparent;
}

.edit-btn {
  background-color: #28a745;
}

.edit-btn:hover {
  background-color: #218838;
}
</style>
