<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from '@tauri-apps/api/core';
import { getTauriVersion, getVersion } from '@tauri-apps/api/app';

const appVersion = ref("");
const tauriVersion = ref("");

const calibrationData = ref({
  voltage: 0,
  current: 0,
  frequency: 0,
  power: 0,
  calibration_date: "",
  certificate_number: "",
  model_details: "",
  company_name: "",
  po_number: ""
});

const calibrations = ref<any[]>([]);
const message = ref("");
const loading = ref(false);

onMounted(async () => {
  // now you can await safely inside onMounted
  appVersion.value = await getVersion();
  tauriVersion.value = await getTauriVersion();
  await loadCalibrations();
});


async function saveCalibration() {
  try {
    loading.value = true;
    message.value = await invoke("save_calibration", { data: calibrationData.value });
    await loadCalibrations();
    // Reset form
    calibrationData.value = {
      voltage: 0,
      current: 0,
      frequency: 0,
      power: 0,
      calibration_date: "",
      certificate_number: "",
      model_details: "",
      company_name: "",
      po_number: ""
    };
  } catch (error) {
    message.value = `Error: ${error}`;
  } finally {
    loading.value = false;
  }
}

async function loadCalibrations() {
  try {
    calibrations.value = await invoke("get_calibrations");
  } catch (error) {
    message.value = `Error loading calibrations: ${error}`;
  }
}

async function openCalibroFolder() {
  try {
    await invoke('open_folder');
  } catch (error) {
    message.value = `Error opening folder: ${error}`;
  }
}

loadCalibrations();
</script>

<template>
  <Suspense>
    <main class="container">
    <h1>Calibration Certificate System</h1>
    <button @click="openCalibroFolder" class="folder-button">
      Open Data & Certificates Folder
    </button>

    <div class="card">
      <h2>New Calibration</h2>
      <form @submit.prevent="saveCalibration" class="calibration-form">
        <div class="form-group">
          <label>Company Name:</label>
          <input v-model="calibrationData.company_name" required placeholder="Enter company name" />
        </div>

        <div class="form-group">
          <label>Certificate Number:</label>
          <input v-model="calibrationData.certificate_number" required placeholder="Enter certificate number" />
        </div>

        <div class="form-group">
          <label>PO Number:</label>
          <input v-model="calibrationData.po_number" required placeholder="Enter PO number" />
        </div>

        <div class="form-group">
          <label>Model Details:</label>
          <input v-model="calibrationData.model_details" required placeholder="Enter model details" />
        </div>

        <div class="form-group">
          <label>Calibration Date:</label>
          <input type="date" v-model="calibrationData.calibration_date" required />
        </div>

        <div class="measurements">
          <div class="form-group">
            <label>Voltage (V):</label>
            <input type="number" v-model="calibrationData.voltage" required step="0.01" />
          </div>

          <div class="form-group">
            <label>Current (A):</label>
            <input type="number" v-model="calibrationData.current" required step="0.01" />
          </div>

          <div class="form-group">
            <label>Frequency (Hz):</label>
            <input type="number" v-model="calibrationData.frequency" required step="0.01" />
          </div>

          <div class="form-group">
            <label>Power (W):</label>
            <input type="number" v-model="calibrationData.power" required step="0.01" />
          </div>
        </div>

        <button type="submit" :disabled="loading">
          {{ loading ? 'Saving...' : 'Save Calibration' }}
        </button>
      </form>

      <p class="message" :class="{ error: message.includes('Error') }">{{ message }}</p>
    </div>

    <div class="card" v-if="calibrations.length > 0">
      <h2>Recent Calibrations</h2>
      <div class="table-container">
        <table>
          <thead>
            <tr>
              <th>Certificate #</th>
              <th>Company</th>
              <th>Model</th>
              <th>Date</th>
              <th>Measurements</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="cal in calibrations" :key="cal.certificate_number">
              <td>{{ cal.certificate_number }}</td>
              <td>{{ cal.company_name }}</td>
              <td>{{ cal.model_details }}</td>
              <td>{{ cal.calibration_date }}</td>
              <td>
                {{ cal.voltage }}V, {{ cal.current }}A,
                {{ cal.frequency }}Hz, {{ cal.power }}W
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
    </main>
    <template #fallback>
      <div class="loading">Loading...</div>
    </template>
  </Suspense>
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
.card {
  background: white;
  padding: 2em;
  border-radius: 8px;
  margin: 1em 0;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.calibration-form {
  display: flex;
  flex-direction: column;
  gap: 1em;
  max-width: 800px;
  margin: 0 auto;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.5em;
  text-align: left;
}

.form-group label {
  font-weight: 500;
}

.measurements {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1em;
  padding: 1em;
  background: #f8f8f8;
  border-radius: 8px;
}

.message {
  margin-top: 1em;
  padding: 1em;
  border-radius: 4px;
  background: #e8f5e9;
}

.message.error {
  background: #ffebee;
  color: #c62828;
}

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

@media (max-width: 768px) {
  .measurements {
    grid-template-columns: 1fr;
  }
}
</style>
<style>
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

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
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

#greet-input {
  margin-right: 5px;
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
