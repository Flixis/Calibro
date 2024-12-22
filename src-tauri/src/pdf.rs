use crate::models::CalibrationData;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

pub fn generate_certificate(
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
