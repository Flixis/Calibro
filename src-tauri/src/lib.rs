use chrono::Local;
use printpdf::*;
use rusqlite::{Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct CalibrationData {
    voltage: f64,
    current: f64,
    frequency: f64,
    power: f64,
    calibration_date: String,
    certificate_number: String,
    model_details: String,
    company_name: String,
    po_number: String,
}

// Ensure DbState is thread-safe
#[derive(Debug)]
struct DbState(Mutex<Connection>);
unsafe impl Send for DbState {}
unsafe impl Sync for DbState {}

fn init_db(app_handle: &AppHandle) -> SqlResult<Connection> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    fs::create_dir_all(&app_dir).expect("Failed to create app data directory");

    let db_path = app_dir.join("calibration.db");
    let db = Connection::open(db_path)?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS calibrations (
            id INTEGER PRIMARY KEY,
            voltage REAL NOT NULL,
            current REAL NOT NULL,
            frequency REAL NOT NULL,
            power REAL NOT NULL,
            calibration_date TEXT NOT NULL,
            certificate_number TEXT NOT NULL UNIQUE,
            model_details TEXT NOT NULL,
            company_name TEXT NOT NULL,
            po_number TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    Ok(db)
}

fn generate_certificate(
    data: &CalibrationData,
    output_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let (doc, page1, layer1) =
        PdfDocument::new("Calibration Certificate", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Add content to PDF
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    // Helper function to add text
    let add_text = |text: &str, size: f32, x: Mm, y: Mm| {
        current_layer.begin_text_section();
        current_layer.set_font(&font, size as _);
        current_layer.set_text_cursor(x, y);
        current_layer.write_text(text, &font);
        current_layer.end_text_section();
    };

    // Title
    add_text("Calibration Certificate", 20_f32, Mm(105.0), Mm(280.0));

    // Company and Certificate Details
    add_text(
        &format!("Company: {}", data.company_name),
        12_f32,
        Mm(20.0),
        Mm(260.0),
    );
    add_text(
        &format!("Certificate Number: {}", data.certificate_number),
        12_f32,
        Mm(20.0),
        Mm(250.0),
    );
    add_text(
        &format!("PO Number: {}", data.po_number),
        12_f32,
        Mm(20.0),
        Mm(240.0),
    );
    add_text(
        &format!("Model Details: {}", data.model_details),
        12_f32,
        Mm(20.0),
        Mm(230.0),
    );
    add_text(
        &format!("Calibration Date: {}", data.calibration_date),
        12_f32,
        Mm(20.0),
        Mm(220.0),
    );

    // Measurements
    add_text("Calibration Measurements:", 14_f32, Mm(20.0), Mm(200.0));
    add_text(
        &format!("Voltage: {} V", data.voltage),
        12_f32,
        Mm(30.0),
        Mm(190.0),
    );
    add_text(
        &format!("Current: {} A", data.current),
        12_f32,
        Mm(30.0),
        Mm(180.0),
    );
    add_text(
        &format!("Frequency: {} Hz", data.frequency),
        12_f32,
        Mm(30.0),
        Mm(170.0),
    );
    add_text(
        &format!("Power: {} W", data.power),
        12_f32,
        Mm(30.0),
        Mm(160.0),
    );

    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer)?;
    Ok(())
}

#[tauri::command]
async fn save_calibration(
    app_handle: AppHandle,
    state: tauri::State<'_, DbState>,
    data: CalibrationData,
) -> Result<String, String> {
    let conn = state
        .0
        .lock()
        .map_err(|_| "Failed to lock database connection")?;

    // Save to database
    conn.execute(
        "INSERT INTO calibrations (
            voltage, current, frequency, power, 
            calibration_date, certificate_number, model_details,
            company_name, po_number, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (
            data.voltage,
            data.current,
            data.frequency,
            data.power,
            &data.calibration_date,
            &data.certificate_number,
            &data.model_details,
            &data.company_name,
            &data.po_number,
            Local::now().to_string(),
        ),
    )
    .map_err(|e| e.to_string())?;

    // Generate certificate
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    let certificates_dir = app_dir.join("certificates");
    fs::create_dir_all(&certificates_dir).map_err(|e| e.to_string())?;

    let cert_path = certificates_dir.join(format!("{}.pdf", data.certificate_number));
    generate_certificate(&data, &cert_path).map_err(|e| e.to_string())?;

    Ok("Calibration data saved and certificate generated successfully".to_string())
}

#[tauri::command]
async fn get_calibrations(
    state: tauri::State<'_, DbState>,
) -> Result<Vec<CalibrationData>, String> {
    let conn = state
        .0
        .lock()
        .map_err(|_| "Failed to lock database connection")?;

    let mut stmt = conn
        .prepare(
            "SELECT voltage, current, frequency, power, 
                calibration_date, certificate_number, model_details,
                company_name, po_number 
         FROM calibrations 
         ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let calibrations = stmt
        .query_map([], |row| {
            Ok(CalibrationData {
                voltage: row.get(0)?,
                current: row.get(1)?,
                frequency: row.get(2)?,
                power: row.get(3)?,
                calibration_date: row.get(4)?,
                certificate_number: row.get(5)?,
                model_details: row.get(6)?,
                company_name: row.get(7)?,
                po_number: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let results: Result<Vec<_>, _> = calibrations.collect();
    results.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db = init_db(&app.handle())?;
            app.manage(DbState(Mutex::new(db)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![save_calibration, get_calibrations])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
