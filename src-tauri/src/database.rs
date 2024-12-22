use crate::models::{CalibrationData, Measurement};
use chrono::Local;
use rusqlite::{Connection, Result as SqlResult};
use std::fs;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

// Ensure DbState is thread-safe
#[derive(Debug)]
pub struct DbState(pub Mutex<Connection>);
unsafe impl Send for DbState {}
unsafe impl Sync for DbState {}

pub fn migrate_db(conn: &Connection) -> SqlResult<()> {
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

pub fn init_db(app_handle: &AppHandle) -> SqlResult<Connection> {
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

pub fn save_calibration(conn: &mut Connection, data: &CalibrationData) -> SqlResult<()> {
    let tx = conn.transaction()?;

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
    )?;

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
        )?;
    }

    tx.commit()?;
    Ok(())
}

pub fn update_calibration(conn: &mut Connection, data: &CalibrationData) -> SqlResult<()> {
    let tx = conn.transaction()?;

    // Get calibration ID
    let calibration_id: i64 = tx.query_row(
        "SELECT id FROM calibrations WHERE certificate_number = ?",
        [&data.certificate_number],
        |row| row.get(0),
    )?;

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
    )?;

    // Delete old measurements
    tx.execute(
        "DELETE FROM measurements WHERE calibration_id = ?",
        [calibration_id],
    )?;

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
        )?;
    }

    tx.commit()?;
    Ok(())
}

pub fn get_calibrations(conn: &Connection) -> SqlResult<Vec<CalibrationData>> {
    let mut stmt = conn.prepare(
        "SELECT id, calibration_date, certificate_number, model_details,
                company_name, po_number, customer 
         FROM calibrations 
         ORDER BY created_at DESC",
    )?;

    let mut calibrations = Vec::new();
    let mut rows = stmt.query([])?;
    
    while let Some(row) = rows.next()? {
        let calibration_id = row.get::<_, i64>(0)?;
        let calibration_date = row.get::<_, String>(1)?;
        let certificate_number = row.get::<_, String>(2)?;
        let model_details = row.get::<_, String>(3)?;
        let company_name = row.get::<_, String>(4)?;
        let po_number = row.get::<_, String>(5)?;
        let customer = row.get::<_, Option<String>>(6)?;
        
        // Get measurements for this calibration
        let mut measurements = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT name, voltage, current, frequency, power 
                 FROM measurements 
                 WHERE calibration_id = ?",
            )?;

            let mut rows = stmt.query([calibration_id])?;
            while let Some(mrow) = rows.next()? {
                measurements.push(Measurement {
                    name: mrow.get::<_, String>(0)?,
                    voltage: mrow.get::<_, f64>(1)?,
                    current: mrow.get::<_, f64>(2)?,
                    frequency: mrow.get::<_, f64>(3)?,
                    power: mrow.get::<_, f64>(4)?,
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

pub fn get_calibration_by_certificate(conn: &Connection, certificate_number: &str) -> SqlResult<CalibrationData> {
    let mut stmt = conn.prepare(
        "SELECT calibration_date, certificate_number, model_details, company_name, po_number, customer 
         FROM calibrations 
         WHERE certificate_number = ?",
    )?;

    let mut rows = stmt.query([certificate_number])?;
    let row = rows.next()?.ok_or_else(|| {
        rusqlite::Error::QueryReturnedNoRows
    })?;

    let calibration_date = row.get::<_, String>(0)?;
    let cert_number = row.get::<_, String>(1)?;
    let model_details = row.get::<_, String>(2)?;
    let company_name = row.get::<_, String>(3)?;
    let po_number = row.get::<_, String>(4)?;
    let customer = row.get::<_, Option<String>>(5)?;

    // Get measurements
    let mut measurements = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT m.name, m.voltage, m.current, m.frequency, m.power 
             FROM measurements m
             JOIN calibrations c ON m.calibration_id = c.id
             WHERE c.certificate_number = ?",
        )?;

        let mut rows = stmt.query([certificate_number])?;
        while let Some(row) = rows.next()? {
            measurements.push(Measurement {
                name: row.get::<_, String>(0)?,
                voltage: row.get::<_, f64>(1)?,
                current: row.get::<_, f64>(2)?,
                frequency: row.get::<_, f64>(3)?,
                power: row.get::<_, f64>(4)?,
            });
        }
    }

    Ok(CalibrationData {
        measurements,
        calibration_date,
        certificate_number: cert_number,
        model_details,
        company_name,
        po_number,
        customer,
    })
}
