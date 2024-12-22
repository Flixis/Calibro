<script setup lang="ts">
import { ref } from "vue";
import { invoke } from '@tauri-apps/api/core';
import type { CalibrationData, Measurement } from '../types';

const props = defineProps<{
  initialData?: CalibrationData;
  isEditMode: boolean;
}>();

const emit = defineEmits<{
  (e: 'saved'): void;
  (e: 'cancelled'): void;
}>();

const emptyMeasurement = (): Measurement => ({
  name: "",
  voltage: 0,
  current: 0,
  frequency: 0,
  power: 0
});

const defaultCalibrationData = (): CalibrationData => ({
  measurements: [emptyMeasurement()],
  calibration_date: "",
  certificate_number: "",
  model_details: "",
  company_name: "",
  po_number: "",
  customer: null
});

const calibrationData = ref<CalibrationData>(props.initialData || defaultCalibrationData());
const loading = ref(false);
const message = ref("");

async function saveCalibration() {
  const command = props.isEditMode ? 'update_calibration' : 'save_calibration';
  try {
    loading.value = true;
    message.value = await invoke(command, { data: calibrationData.value });
    emit('saved');
  } catch (error) {
    message.value = `Error: ${error}`;
  } finally {
    loading.value = false;
  }
}

function updateMeasurementValue(measurement: Measurement, field: keyof Measurement, value: string) {
  if (field === 'name') {
    measurement[field] = value;
  } else {
    measurement[field] = parseFloat(value) || 0;
  }
}
</script>

<template>
  <div class="card">
    <h2>{{ isEditMode ? 'Edit Calibration' : 'New Calibration' }}</h2>
    <form @submit.prevent="saveCalibration" class="calibration-form">
      <div class="form-group">
        <label>Company Name:</label>
        <input v-model="calibrationData.company_name" required placeholder="Enter company name" />
      </div>

      <div class="form-group">
        <label>Certificate Number:</label>
        <input 
          v-model="calibrationData.certificate_number" 
          required 
          placeholder="Enter certificate number"
          :disabled="isEditMode" 
        />
      </div>

      <div class="form-group">
        <label>PO Number:</label>
        <input v-model="calibrationData.po_number" required placeholder="Enter PO number" />
      </div>

      <div class="form-group">
        <label>Customer (Optional):</label>
        <input v-model="calibrationData.customer" placeholder="Enter customer name" />
      </div>

      <div class="form-group">
        <label>Model Details:</label>
        <input v-model="calibrationData.model_details" required placeholder="Enter model details" />
      </div>

      <div class="form-group">
        <label>Calibration Date:</label>
        <input type="date" v-model="calibrationData.calibration_date" required />
      </div>

      <div class="measurements-container">
        <h3>Measurements</h3>
        <div v-for="(measurement, index) in calibrationData.measurements" :key="index" class="measurement-group">
          <div class="measurement-header">
            <h4>Measurement {{ index + 1 }}</h4>
            <button 
              v-if="calibrationData.measurements.length > 1" 
              type="button" 
              class="remove-btn"
              @click="calibrationData.measurements.splice(index, 1)"
            >
              Remove
            </button>
          </div>

          <div class="form-group">
            <label>Name:</label>
            <input 
              :value="measurement.name"
              @input="(e) => updateMeasurementValue(measurement, 'name', (e.target as HTMLInputElement).value)"
              required 
              placeholder="Enter measurement name" 
            />
          </div>

          <div class="measurements">
            <div class="form-group">
              <label>Voltage (V):</label>
              <input 
                type="number" 
                :value="measurement.voltage"
                @input="(e) => updateMeasurementValue(measurement, 'voltage', (e.target as HTMLInputElement).value)"
                required 
                step="0.01" 
              />
            </div>

            <div class="form-group">
              <label>Current (A):</label>
              <input 
                type="number" 
                :value="measurement.current"
                @input="(e) => updateMeasurementValue(measurement, 'current', (e.target as HTMLInputElement).value)"
                required 
                step="0.01" 
              />
            </div>

            <div class="form-group">
              <label>Frequency (Hz):</label>
              <input 
                type="number" 
                :value="measurement.frequency"
                @input="(e) => updateMeasurementValue(measurement, 'frequency', (e.target as HTMLInputElement).value)"
                required 
                step="0.01" 
              />
            </div>

            <div class="form-group">
              <label>Power (W):</label>
              <input 
                type="number" 
                :value="measurement.power"
                @input="(e) => updateMeasurementValue(measurement, 'power', (e.target as HTMLInputElement).value)"
                required 
                step="0.01" 
              />
            </div>
          </div>
        </div>

        <button 
          type="button" 
          class="add-measurement-btn"
          @click="calibrationData.measurements.push(emptyMeasurement())"
        >
          Add Measurement
        </button>
      </div>

      <div class="button-group">
        <button type="submit" :disabled="loading">
          {{ loading ? 'Saving...' : (isEditMode ? 'Update Calibration' : 'Save Calibration') }}
        </button>
        <button 
          v-if="isEditMode" 
          type="button" 
          class="cancel-btn" 
          @click="emit('cancelled')"
          :disabled="loading"
        >
          Cancel
        </button>
      </div>
    </form>

    <p class="message" :class="{ error: message.includes('Error') }">{{ message }}</p>
  </div>
</template>

<style scoped>
@import '../styles/form.css';
</style>
