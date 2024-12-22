import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { CalibrationData } from '../types';

export function useCalibrations() {
  const calibrations = ref<CalibrationData[]>([]);
  const loading = ref(false);
  const message = ref("");

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

  return {
    calibrations,
    loading,
    message,
    loadCalibrations,
    openCalibroFolder
  };
}
