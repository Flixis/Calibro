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
pub struct Measurement {
    name: String,
    voltage: f64,
    current: f64,
    frequency: f64,
    power: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalibrationData {
    measurements: Vec<Measurement>,
    calibration_date: String,
    certificate_number: String,
    model_details: String,
    company_name: String,
    po_number: String,
    customer: Option<String>,
}

// Ensure DbState is thread-safe
#[derive(Debug)]
struct DbState(Mutex<Connection>);
unsafe impl Send for DbState {}
unsafe impl Sync for DbState {}

fn migrate_db(conn: &Connection) -> SqlResult<()> {
    // Get list of columns in calibrations table
    let mut stmt = conn.prepare("PRAGMA table_info(calibrations)")?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    // Check if we need to add the customer column
    if !columns.iter().any(|c| c == "customer") {
        conn.execute(
            "ALTER TABLE calibrations ADD COLUMN customer TEXT",
            [],
        )?;
    }

    // If we find any measurement columns in calibrations table, migrate them
    let measurement_columns = ["voltage", "current", "frequency", "power"];
    for col in measurement_columns.iter() {
        if columns.iter().any(|c| c == col) {
            // Create a temporary table with correct schema
            conn.execute(
                "CREATE TABLE calibrations_new (
                    id INTEGER PRIMARY KEY,
                    calibration_date TEXT NOT NULL,
                    certificate_number TEXT NOT NULL UNIQUE,
                    model_details TEXT NOT NULL,
                    company_name TEXT NOT NULL,
                    po_number TEXT NOT NULL,
                    customer TEXT,
                    created_at TEXT NOT NULL
                )",
                [],
            )?;

            // Copy data to new table
            conn.execute(
                "INSERT INTO calibrations_new 
                SELECT id, calibration_date, certificate_number, model_details,
                       company_name, po_number, customer, created_at
                FROM calibrations",
                [],
            )?;

            // Drop old table and rename new one
            conn.execute("DROP TABLE calibrations", [])?;
            conn.execute("ALTER TABLE calibrations_new RENAME TO calibrations", [])?;

            // No need to check other columns since we've already migrated
            break;
        }
    }

    Ok(())
}

fn init_db(app_handle: &AppHandle) -> SqlResult<Connection> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    fs::create_dir_all(&app_dir).expect("Failed to create app data directory");

    let db_path = app_dir.join("calibration.db");
    let db = Connection::open(db_path)?;

    // Create tables if they don't exist with correct schema
    db.execute(
        "CREATE TABLE IF NOT EXISTS calibrations (
            id INTEGER PRIMARY KEY,
            calibration_date TEXT NOT NULL,
            certificate_number TEXT NOT NULL UNIQUE,
            model_details TEXT NOT NULL,
            company_name TEXT NOT NULL,
            po_number TEXT NOT NULL,
            customer TEXT,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS measurements (
            id INTEGER PRIMARY KEY,
            calibration_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            voltage REAL NOT NULL,
            current REAL NOT NULL,
            frequency REAL NOT NULL,
            power REAL NOT NULL,
            FOREIGN KEY(calibration_id) REFERENCES calibrations(id)
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
    
    if let Some(customer) = &data.customer {
        add_text(
            &format!("Customer: {}", customer),
            12_f32,
            Mm(20.0),
            Mm(250.0),
        );
    }
    add_text(
        &format!("Certificate Number: {}", data.certificate_number),
        12_f32,
        Mm(20.0),
        if data.customer.is_some() { Mm(240.0) } else { Mm(250.0) },
    );
    add_text(
        &format!("PO Number: {}", data.po_number),
        12_f32,
        Mm(20.0),
        if data.customer.is_some() { Mm(230.0) } else { Mm(240.0) },
    );
    add_text(
        &format!("Model Details: {}", data.model_details),
        12_f32,
        Mm(20.0),
        if data.customer.is_some() { Mm(220.0) } else { Mm(230.0) },
    );
    add_text(
        &format!("Calibration Date: {}", data.calibration_date),
        12_f32,
        Mm(20.0),
        if data.customer.is_some() { Mm(210.0) } else { Mm(220.0) },
    );

    // Measurements
    let measurements_y = if data.customer.is_some() { Mm(190.0) } else { Mm(200.0) };
    add_text("Calibration Measurements:", 14_f32, Mm(20.0), measurements_y);
    
    let mut y_pos = if data.customer.is_some() { Mm(180.0) } else { Mm(190.0) };
    for measurement in &data.measurements {
        add_text(&format!("Measurement: {}", measurement.name), 12_f32, Mm(30.0), y_pos);
        y_pos = y_pos - Mm(10.0);
        
        add_text(&format!("  Voltage: {} V", measurement.voltage), 12_f32, Mm(40.0), y_pos);
        y_pos = y_pos - Mm(10.0);
        
        add_text(&format!("  Current: {} A", measurement.current), 12_f32, Mm(40.0), y_pos);
        y_pos = y_pos - Mm(10.0);
        
        add_text(&format!("  Frequency: {} Hz", measurement.frequency), 12_f32, Mm(40.0), y_pos);
        y_pos = y_pos - Mm(10.0);
        
        add_text(&format!("  Power: {} W", measurement.power), 12_f32, Mm(40.0), y_pos);
        y_pos = y_pos - Mm(15.0); // Extra space between measurements
    }

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
    {
        let mut conn = state
            .0
            .lock()
            .map_err(|_| "Failed to lock database connection")?;

        // Start a transaction
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // Save calibration data
        tx.execute(
            "INSERT INTO calibrations (
                calibration_date, certificate_number, model_details,
                company_name, po_number, customer, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                &data.calibration_date,
                &data.certificate_number,
                &data.model_details,
                &data.company_name,
                &data.po_number,
                &data.customer,
                Local::now().to_string(),
            ),
        )
        .map_err(|e| e.to_string())?;

        let calibration_id = tx.last_insert_rowid();

        // Save measurements
        for measurement in &data.measurements {
            tx.execute(
                "INSERT INTO measurements (
                    calibration_id, name, voltage, current, frequency, power
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (
                    calibration_id,
                    &measurement.name,
                    measurement.voltage,
                    measurement.current,
                    measurement.frequency,
                    measurement.power,
                ),
            )
            .map_err(|e| e.to_string())?;
        }

        // Commit transaction
        tx.commit().map_err(|e| e.to_string())?;
    }

    Ok("Calibration data saved successfully".to_string())
}

#[tauri::command]
async fn get_calibrations(
    state: tauri::State<'_, DbState>,
) -> Result<Vec<CalibrationData>, String> {
    let mut conn = state.0.lock().map_err(|_| "Failed to lock database connection")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, calibration_date, certificate_number, model_details,
                    company_name, po_number, customer 
             FROM calibrations 
             ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let mut calibrations = Vec::new();
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let calibration_id = row.get::<_, i64>(0).map_err(|e| e.to_string())?;
        let calibration_date = row.get::<_, String>(1).map_err(|e| e.to_string())?;
        let certificate_number = row.get::<_, String>(2).map_err(|e| e.to_string())?;
        let model_details = row.get::<_, String>(3).map_err(|e| e.to_string())?;
        let company_name = row.get::<_, String>(4).map_err(|e| e.to_string())?;
        let po_number = row.get::<_, String>(5).map_err(|e| e.to_string())?;
        let customer = row.get::<_, Option<String>>(6).map_err(|e| e.to_string())?;
        
        // Get measurements for this calibration
        let mut measurements = Vec::new();
        {
            let mut stmt = conn
                .prepare(
                    "SELECT name, voltage, current, frequency, power 
                     FROM measurements 
                     WHERE calibration_id = ?",
                )
                .map_err(|e| e.to_string())?;

            let mut rows = stmt.query([calibration_id]).map_err(|e| e.to_string())?;
            while let Some(mrow) = rows.next().map_err(|e| e.to_string())? {
                measurements.push(Measurement {
                    name: mrow.get::<_, String>(0).map_err(|e| e.to_string())?,
                    voltage: mrow.get::<_, f64>(1).map_err(|e| e.to_string())?,
                    current: mrow.get::<_, f64>(2).map_err(|e| e.to_string())?,
                    frequency: mrow.get::<_, f64>(3).map_err(|e| e.to_string())?,
                    power: mrow.get::<_, f64>(4).map_err(|e| e.to_string())?,
                });
            }
        }

        calibrations.push(CalibrationData {
            measurements,
            calibration_date,
            certificate_number,
            model_details,
            company_name,
            po_number,
            customer,
        });
    }

    Ok(calibrations)
}

#[tauri::command]
async fn open_folder(app_handle: AppHandle) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    opener::open(app_dir).map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_calibration(
    app_handle: AppHandle,
    state: tauri::State<'_, DbState>,
    data: CalibrationData,
) -> Result<String, String> {
    let mut conn = state
        .0
        .lock()
        .map_err(|_| "Failed to lock database connection")?;

    // Start a transaction
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Get calibration ID
    let calibration_id: i64 = tx
        .query_row(
            "SELECT id FROM calibrations WHERE certificate_number = ?",
            [&data.certificate_number],
            |row| row.get(0),
        )
        .map_err(|_| "Calibration not found".to_string())?;

    // Update calibration data
    tx.execute(
        "UPDATE calibrations SET 
            calibration_date = ?1,
            model_details = ?2,
            company_name = ?3,
            po_number = ?4,
            customer = ?5
        WHERE certificate_number = ?6",
        (
            &data.calibration_date,
            &data.model_details,
            &data.company_name,
            &data.po_number,
            &data.customer,
            &data.certificate_number,
        ),
    )
    .map_err(|e| e.to_string())?;

    // Delete old measurements
    tx.execute(
        "DELETE FROM measurements WHERE calibration_id = ?",
        [calibration_id],
    )
    .map_err(|e| e.to_string())?;

    // Insert new measurements
    for measurement in &data.measurements {
        tx.execute(
            "INSERT INTO measurements (
                calibration_id, name, voltage, current, frequency, power
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                calibration_id,
                &measurement.name,
                measurement.voltage,
                measurement.current,
                measurement.frequency,
                measurement.power,
            ),
        )
        .map_err(|e| e.to_string())?;
    }

    // Commit transaction
    tx.commit().map_err(|e| e.to_string())?;

    Ok("Calibration updated successfully".to_string())
}

#[tauri::command]
async fn generate_pdf(
    app_handle: AppHandle,
    state: tauri::State<'_, DbState>,
    certificate_number: String,
) -> Result<(), String> {
    let data = {
        let conn = state.0.lock().map_err(|_| "Failed to lock database connection")?;
        
        // Get calibration data
        let mut stmt = conn.prepare(
            "SELECT calibration_date, certificate_number, model_details, company_name, po_number, customer 
             FROM calibrations 
             WHERE certificate_number = ?",
        ).map_err(|e| e.to_string())?;

        let mut rows = stmt.query([&certificate_number]).map_err(|e| e.to_string())?;
        let row = rows.next().map_err(|e| e.to_string())?.ok_or("Calibration not found")?;

        let calibration_date = row.get::<_, String>(0).map_err(|e| e.to_string())?;
        let cert_number = row.get::<_, String>(1).map_err(|e| e.to_string())?;
        let model_details = row.get::<_, String>(2).map_err(|e| e.to_string())?;
        let company_name = row.get::<_, String>(3).map_err(|e| e.to_string())?;
        let po_number = row.get::<_, String>(4).map_err(|e| e.to_string())?;
        let customer = row.get::<_, Option<String>>(5).map_err(|e| e.to_string())?;

        // Get measurements
        let mut measurements = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT m.name, m.voltage, m.current, m.frequency, m.power 
                 FROM measurements m
                 JOIN calibrations c ON m.calibration_id = c.id
                 WHERE c.certificate_number = ?",
            ).map_err(|e| e.to_string())?;

            let mut rows = stmt.query([&certificate_number]).map_err(|e| e.to_string())?;
            while let Some(row) = rows.next().map_err(|e| e.to_string())? {
                measurements.push(Measurement {
                    name: row.get::<_, String>(0).map_err(|e| e.to_string())?,
                    voltage: row.get::<_, f64>(1).map_err(|e| e.to_string())?,
                    current: row.get::<_, f64>(2).map_err(|e| e.to_string())?,
                    frequency: row.get::<_, f64>(3).map_err(|e| e.to_string())?,
                    power: row.get::<_, f64>(4).map_err(|e| e.to_string())?,
                });
            }
        }

        CalibrationData {
            measurements,
            calibration_date,
            certificate_number: cert_number,
            model_details,
            company_name,
            po_number,
            customer,
        }
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
    generate_certificate(&data, &cert_path).map_err(|e| e.to_string())?;
    
    // Open the generated PDF file
    opener::open(&cert_path).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db = init_db(&app.handle())?;
            migrate_db(&db)?;
            app.manage(DbState(Mutex::new(db)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![save_calibration, update_calibration, get_calibrations, open_folder, generate_pdf])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
