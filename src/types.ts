export interface Measurement {
  name: string;
  voltage: number;
  current: number;
  frequency: number;
  power: number;
}

export interface CalibrationData {
  measurements: Measurement[];
  calibration_date: string;
  certificate_number: string;
  model_details: string;
  company_name: string;
  po_number: string;
  customer: string | null;
}
