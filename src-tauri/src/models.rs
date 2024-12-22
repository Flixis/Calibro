use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Measurement {
    pub name: String,
    pub voltage: f64,
    pub current: f64,
    pub frequency: f64,
    pub power: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalibrationData {
    pub measurements: Vec<Measurement>,
    pub calibration_date: String,
    pub certificate_number: String,
    pub model_details: String,
    pub company_name: String,
    pub po_number: String,
    pub customer: Option<String>,
}
