use crate::api::middleware::AppError;
use crate::models::quote::Dimensions;
use quick_xml::events::Event;
use quick_xml::reader::Reader as XmlReader;
use std::io::{BufReader, Read};
use std::path::Path;

/// Result of processing a 3D model file
#[derive(Debug)]
pub struct ProcessedModel {
    pub volume_cm3: f64,
    pub dimensions_mm: Dimensions,
    pub triangle_count: i32,
}

/// Validate file format and size
pub fn validate_file(bytes: &[u8], filename: &str, max_size: i64) -> Result<String, AppError> {
    // Check file size
    if bytes.len() as i64 > max_size {
        return Err(AppError::FileTooLarge(bytes.len() as i64, max_size));
    }

    // Determine format from extension
    let extension = Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "stl" => {
            // Verify it's a valid STL (binary or ASCII)
            if bytes.len() < 84 {
                return Err(AppError::FileProcessing(
                    "Fichier STL trop petit pour être valide".to_string(),
                ));
            }

            // Check if ASCII STL (starts with "solid")
            if bytes.starts_with(b"solid") {
                Ok("stl".to_string())
            } else {
                // Binary STL - verify header and triangle count
                let triangle_count =
                    u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]);
                let expected_size = 84 + (triangle_count as usize * 50);

                if bytes.len() != expected_size {
                    return Err(AppError::FileProcessing(
                        "Fichier STL binaire corrompu".to_string(),
                    ));
                }

                Ok("stl".to_string())
            }
        }
        "3mf" => {
            // 3MF is a ZIP file - check for ZIP signature
            if bytes.len() < 4 || &bytes[0..4] != b"PK\x03\x04" {
                return Err(AppError::FileProcessing(
                    "Fichier 3MF invalide (signature ZIP manquante)".to_string(),
                ));
            }
            Ok("3mf".to_string())
        }
        _ => Err(AppError::InvalidFileFormat(extension)),
    }
}

/// Process an uploaded STL file
pub fn process_stl_file(file_path: &str) -> Result<ProcessedModel, AppError> {
    let file = std::fs::File::open(file_path)
        .map_err(|e| AppError::FileProcessing(format!("Impossible d'ouvrir le fichier: {}", e)))?;

    let mut reader = BufReader::new(file);
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|e| AppError::FileProcessing(format!("Erreur de lecture: {}", e)))?;

    // Parse STL using stl_io
    let stl = stl_io::read_stl(&mut std::io::Cursor::new(&bytes))
        .map_err(|e| AppError::FileProcessing(format!("Erreur de parsing STL: {}", e)))?;

    let triangles: Vec<[f32; 9]> = stl
        .faces
        .iter()
        .map(|face| {
            // stl_io IndexedTriangle has vertices as indices into mesh.vertices
            let v0 = stl.vertices[face.vertices[0]];
            let v1 = stl.vertices[face.vertices[1]];
            let v2 = stl.vertices[face.vertices[2]];
            [
                v0[0], v0[1], v0[2], v1[0], v1[1], v1[2], v2[0], v2[1], v2[2],
            ]
        })
        .collect();

    let triangle_count = triangles.len() as i32;
    let volume_cm3 = calculate_volume(&triangles);
    let dimensions_mm = calculate_dimensions(&triangles);

    Ok(ProcessedModel {
        volume_cm3,
        dimensions_mm,
        triangle_count,
    })
}

/// Process an uploaded 3MF file
pub fn process_3mf_file(file_path: &str) -> Result<ProcessedModel, AppError> {
    let file = std::fs::File::open(file_path)
        .map_err(|e| AppError::FileProcessing(format!("Impossible d'ouvrir le fichier: {}", e)))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::FileProcessing(format!("Erreur de lecture ZIP: {}", e)))?;

    // Find the 3D model file (usually 3D/3dmodel.model)
    let model_xml = find_and_read_3mf_model(&mut archive)?;

    // Parse the XML to extract mesh data
    let triangles = parse_3mf_mesh(&model_xml)?;

    let triangle_count = triangles.len() as i32;
    let volume_cm3 = calculate_volume(&triangles);
    let dimensions_mm = calculate_dimensions(&triangles);

    Ok(ProcessedModel {
        volume_cm3,
        dimensions_mm,
        triangle_count,
    })
}

/// Find and read the 3D model XML from a 3MF archive
fn find_and_read_3mf_model(
    archive: &mut zip::ZipArchive<std::fs::File>,
) -> Result<String, AppError> {
    // Try common locations for the 3D model file
    let possible_paths = vec!["3D/3dmodel.model", "3d/3dmodel.model", "3D/3DModel.model"];

    // First, try to get the main model file
    let mut main_model_content = None;
    for path in &possible_paths {
        if let Ok(mut file) = archive.by_name(path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).map_err(|e| {
                AppError::FileProcessing(format!("Erreur de lecture du modèle 3MF: {}", e))
            })?;
            main_model_content = Some(contents);
            break;
        }
    }

    // Check if main model has actual mesh data or references components
    if let Some(ref content) = main_model_content {
        // Check if it has actual triangles
        if content.contains("<triangles") || content.contains("<triangle") {
            return Ok(content.clone());
        }

        // Extract component paths from main model
        let component_paths = extract_component_paths(content);
        if !component_paths.is_empty() {
            // Load all component models and concatenate
            let mut all_contents = content.clone();
            for comp_path in component_paths {
                let clean_path = comp_path.trim_start_matches('/');
                if let Ok(mut comp_file) = archive.by_name(clean_path) {
                    let mut comp_content = String::new();
                    if comp_file.read_to_string(&mut comp_content).is_ok() {
                        all_contents.push_str(&comp_content);
                    }
                }
            }
            return Ok(all_contents);
        }
    }

    // Fallback: find largest .model file (likely has mesh data)
    let mut largest_model = None;
    let mut largest_size = 0;

    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| {
            AppError::FileProcessing(format!("Erreur d'accès au fichier ZIP: {}", e))
        })?;
        if file.name().ends_with(".model") && file.size() > largest_size {
            largest_size = file.size();
            largest_model = Some(i);
        }
    }

    if let Some(idx) = largest_model {
        let mut file = archive.by_index(idx).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            AppError::FileProcessing(format!("Erreur de lecture du modèle 3MF: {}", e))
        })?;
        return Ok(contents);
    }

    Err(AppError::FileProcessing(
        "Fichier 3MF invalide: modèle 3D non trouvé".to_string(),
    ))
}

/// Extract component paths from 3MF main model XML
fn extract_component_paths(xml_content: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut reader = XmlReader::from_str(xml_content);
    reader.trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                if e.local_name().as_ref() == b"component" {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"path" {
                            if let Ok(path_str) = std::str::from_utf8(&attr.value) {
                                paths.push(path_str.to_string());
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    paths
}

/// Parse 3MF XML to extract mesh triangles
fn parse_3mf_mesh(xml_content: &str) -> Result<Vec<[f32; 9]>, AppError> {
    let mut reader = XmlReader::from_str(xml_content);
    reader.trim_text(true);

    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut triangles: Vec<[f32; 9]> = Vec::new();
    let mut buf = Vec::new();
    let mut in_vertices = false;
    let mut in_triangles = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"vertices" => in_vertices = true,
                    b"triangles" => in_triangles = true,
                    b"vertex" if in_vertices => {
                        let mut x = 0.0f32;
                        let mut y = 0.0f32;
                        let mut z = 0.0f32;

                        for attr in e.attributes().flatten() {
                            match attr.key.local_name().as_ref() {
                                b"x" => x = parse_f32_attr(&attr.value),
                                b"y" => y = parse_f32_attr(&attr.value),
                                b"z" => z = parse_f32_attr(&attr.value),
                                _ => {}
                            }
                        }
                        vertices.push([x, y, z]);
                    }
                    b"triangle" if in_triangles => {
                        let mut v1: usize = 0;
                        let mut v2: usize = 0;
                        let mut v3: usize = 0;

                        for attr in e.attributes().flatten() {
                            match attr.key.local_name().as_ref() {
                                b"v1" => v1 = parse_usize_attr(&attr.value),
                                b"v2" => v2 = parse_usize_attr(&attr.value),
                                b"v3" => v3 = parse_usize_attr(&attr.value),
                                _ => {}
                            }
                        }

                        if v1 < vertices.len() && v2 < vertices.len() && v3 < vertices.len() {
                            triangles.push([
                                vertices[v1][0],
                                vertices[v1][1],
                                vertices[v1][2],
                                vertices[v2][0],
                                vertices[v2][1],
                                vertices[v2][2],
                                vertices[v3][0],
                                vertices[v3][1],
                                vertices[v3][2],
                            ]);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                b"vertices" => in_vertices = false,
                b"triangles" => in_triangles = false,
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(AppError::FileProcessing(format!(
                    "Erreur de parsing XML 3MF: {}",
                    e
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    if triangles.is_empty() {
        return Err(AppError::FileProcessing(
            "Fichier 3MF invalide: aucun triangle trouvé".to_string(),
        ));
    }

    Ok(triangles)
}

fn parse_f32_attr(value: &[u8]) -> f32 {
    std::str::from_utf8(value)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0)
}

fn parse_usize_attr(value: &[u8]) -> usize {
    std::str::from_utf8(value)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Calculate volume from mesh triangles using signed tetrahedron method
/// Volume = Σ (v1 · (v2 × v3)) / 6
pub fn calculate_volume(triangles: &[[f32; 9]]) -> f64 {
    let mut total_volume = 0.0f64;

    for tri in triangles {
        // Extract vertices
        let v1 = [tri[0] as f64, tri[1] as f64, tri[2] as f64];
        let v2 = [tri[3] as f64, tri[4] as f64, tri[5] as f64];
        let v3 = [tri[6] as f64, tri[7] as f64, tri[8] as f64];

        // Calculate signed volume of tetrahedron formed with origin
        // V = (v1 · (v2 × v3)) / 6
        let cross = [
            v2[1] * v3[2] - v2[2] * v3[1],
            v2[2] * v3[0] - v2[0] * v3[2],
            v2[0] * v3[1] - v2[1] * v3[0],
        ];

        let dot = v1[0] * cross[0] + v1[1] * cross[1] + v1[2] * cross[2];
        total_volume += dot / 6.0;
    }

    // Convert from mm³ to cm³ (divide by 1000)
    // Take absolute value in case mesh normals are inverted
    (total_volume / 1000.0).abs()
}

/// Calculate bounding box dimensions from triangles
pub fn calculate_dimensions(triangles: &[[f32; 9]]) -> Dimensions {
    if triangles.is_empty() {
        return Dimensions {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
    }

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;

    for tri in triangles {
        // Check all 3 vertices
        for i in 0..3 {
            let x = tri[i * 3];
            let y = tri[i * 3 + 1];
            let z = tri[i * 3 + 2];

            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
            min_z = min_z.min(z);
            max_z = max_z.max(z);
        }
    }

    Dimensions {
        x: (max_x - min_x) as f64,
        y: (max_y - min_y) as f64,
        z: (max_z - min_z) as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_calculation_cube() {
        // A 10mm cube should have volume of 1 cm³
        // Cube vertices at (0,0,0) to (10,10,10)
        let triangles = create_cube_triangles(10.0);
        let volume = calculate_volume(&triangles);

        // Allow for floating point precision
        assert!(
            (volume - 1.0).abs() < 0.01,
            "Expected ~1.0 cm³, got {}",
            volume
        );
    }

    #[test]
    fn test_dimensions_calculation() {
        let triangles = create_cube_triangles(20.0);
        let dims = calculate_dimensions(&triangles);

        assert!((dims.x - 20.0).abs() < 0.01);
        assert!((dims.y - 20.0).abs() < 0.01);
        assert!((dims.z - 20.0).abs() < 0.01);
    }

    // Helper to create a simple cube mesh
    // Vertices ordered counter-clockwise when viewed from outside (normals pointing outward)
    fn create_cube_triangles(size: f32) -> Vec<[f32; 9]> {
        vec![
            // Front face (z = size, normal +z)
            [0.0, 0.0, size, size, 0.0, size, size, size, size],
            [0.0, 0.0, size, size, size, size, 0.0, size, size],
            // Back face (z = 0, normal -z)
            [size, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, size, 0.0],
            [size, 0.0, 0.0, 0.0, size, 0.0, size, size, 0.0],
            // Top face (y = size, normal +y)
            [0.0, size, 0.0, 0.0, size, size, size, size, size],
            [0.0, size, 0.0, size, size, size, size, size, 0.0],
            // Bottom face (y = 0, normal -y)
            [0.0, 0.0, 0.0, size, 0.0, 0.0, size, 0.0, size],
            [0.0, 0.0, 0.0, size, 0.0, size, 0.0, 0.0, size],
            // Right face (x = size, normal +x)
            [size, 0.0, 0.0, size, size, 0.0, size, size, size],
            [size, 0.0, 0.0, size, size, size, size, 0.0, size],
            // Left face (x = 0, normal -x)
            [0.0, 0.0, size, 0.0, 0.0, 0.0, 0.0, size, 0.0],
            [0.0, 0.0, size, 0.0, size, 0.0, 0.0, size, size],
        ]
    }

    #[test]
    fn test_validate_file_size() {
        let big_file = vec![0u8; 100_000_000]; // 100MB
        let result = validate_file(&big_file, "test.stl", 50_000_000);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_invalid_format() {
        let file = vec![0u8; 100];
        let result = validate_file(&file, "test.obj", 50_000_000);
        assert!(result.is_err());
    }
}
