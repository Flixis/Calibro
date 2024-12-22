use crate::database::{self, DbState};
use crate::models::CalibrationData;
use crate::pdf;
use std::fs;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn save_calibration(
    _app_handle: AppHandle,
    state: tauri::State<'_, DbState>,
    data: CalibrationData,
) -> Result<String, String> {
    let mut conn = state
        .0
        .lock()
        .map_err(|_| "Failed to lock database connection")?;

    database::save_calibration(&mut conn, &data)
        .map_err(|e| e.to_string())?;

    Ok("Calibration data saved successfully".to_string())
}

#[tauri::command]
pub async fn get_calibrations(
    state: tauri::State<'_, DbState>,
) -> Result<Vec<CalibrationData>, String> {
    let conn = state.0.lock().map_err(|_| "Failed to lock database connection")?;
    database::get_calibrations(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_folder(app_handle: AppHandle) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    opener::open(app_dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_calibration(
    _app_handle: AppHandle,
    state: tauri::State<'_, DbState>,
    data: CalibrationData,
) -> Result<String, String> {
    let mut conn = state
        .0
        .lock()
        .map_err(|_| "Failed to lock database connection")?;

    database::update_calibration(&mut conn, &data)
        .map_err(|e| e.to_string())?;

    Ok("Calibration updated successfully".to_string())
}

#[tauri::command]
pub async fn generate_pdf(
    app_handle: AppHandle,
    state: tauri::State<'_, DbState>,
    certificate_number: String,
) -> Result<(), String> {
    let data = {
        let conn = state.0.lock().map_err(|_| "Failed to lock database connection")?;
        database::get_calibration_by_certificate(&conn, &certificate_number)
            .map_err(|e| e.to_string())?
    };

    // Generate certificate
    let app_dir = app_handle.path().app_data_dir().expect("Failed to get app data dir");
    let base_certificates_dir = app_dir.join("certificates");
    
    // Create the certificates directory path based on customer
    let certificates_dir = match &data.customer {
        Some(customer) => {
            let customer_dir = base_certificates_dir.join(customer.replace(" ", "_"));
            fs::create_dir_all(&customer_dir).map_err(|e| e.to_string())?;
            customer_dir
        },
        None => {
            let general_dir = base_certificates_dir.join("general");
            fs::create_dir_all(&general_dir).map_err(|e| e.to_string())?;
            general_dir
        }
    };

    // Create filename using just the certificate number since it's already in a customer folder
    let filename = format!("{}.pdf", &certificate_number);
    let cert_path = certificates_dir.join(filename);
    pdf::generate_certificate(&data, &cert_path).map_err(|e| e.to_string())?;
    
    // Open the generated PDF file
    opener::open(&cert_path).map_err(|e| e.to_string())?;

    Ok(())
}
