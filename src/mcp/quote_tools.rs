//! MCP tools for 3D print quote generation

use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// Quote service tools configuration
#[derive(Debug, Clone)]
pub struct QuoteTools {
    pub pool: PgPool,
    pub upload_dir: String,
    pub max_file_size: usize,
    tool_router: ToolRouter<Self>,
}

/// 3D model dimensions in millimeters
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Dimensions {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Material information
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MaterialInfo {
    pub id: String,
    pub name: String,
    pub base_price_per_cm3: f64,
    pub active: bool,
}

/// Input for upload_model tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UploadModelInput {
    /// Session ID for grouping related uploads
    pub session_id: String,
    /// Original filename (must end with .stl or .3mf)
    pub filename: String,
    /// Base64-encoded file data
    pub file_data: String,
}

/// Result from upload_model tool
#[derive(Debug, Serialize, JsonSchema)]
pub struct UploadModelResult {
    pub model_id: String,
    pub filename: String,
    pub file_format: String,
    pub volume_cm3: Option<f64>,
    pub dimensions_mm: Option<Dimensions>,
    pub triangle_count: Option<i32>,
}

/// Input for configure_model tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConfigureModelInput {
    /// Session ID
    pub session_id: String,
    /// Model ID to configure
    pub model_id: String,
    /// Material ID to use
    pub material_id: String,
    /// Quantity to produce
    pub quantity: i32,
}

/// Result from configure_model tool
#[derive(Debug, Serialize, JsonSchema)]
pub struct ConfigureModelResult {
    pub model_id: String,
    pub material_id: String,
    pub quantity: i32,
    pub estimated_price: f64,
}

/// Input for generate_quote tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenerateQuoteInput {
    /// Session ID containing configured models
    pub session_id: String,
}

/// Quote item
#[derive(Debug, Serialize, JsonSchema)]
pub struct QuoteItem {
    pub model_id: String,
    pub filename: String,
    pub material_name: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub line_total: f64,
}

/// Result from generate_quote tool
#[derive(Debug, Serialize, JsonSchema)]
pub struct QuoteResult {
    pub quote_id: String,
    pub session_id: String,
    pub items: Vec<QuoteItem>,
    pub subtotal: f64,
    pub total: f64,
}

#[tool_router]
impl QuoteTools {
    pub fn new(pool: PgPool, upload_dir: String, max_file_size: usize) -> Self {
        Self {
            pool,
            upload_dir,
            max_file_size,
            tool_router: Self::tool_router(),
        }
    }

    /// Upload a 3D model file (STL or 3MF) from base64-encoded data.
    /// The file will be validated, processed to extract volume and dimensions,
    /// and stored in the session for later configuration.
    #[tool(description = "Upload a 3D model file (STL or 3MF) from base64-encoded data")]
    async fn upload_model(
        &self,
        Parameters(input): Parameters<UploadModelInput>,
    ) -> Result<String, String> {
        use crate::api::middleware::sanitize_filename;
        use crate::business::{SessionService, file_processor};
        use crate::models::{model::CreateModel, quote::UploadedModel};
        use crate::persistence::models;
        use base64::{Engine as _, engine::general_purpose};
        use std::path::PathBuf;

        // Verify session exists
        let session_service = SessionService::new(self.pool.clone(), &self.upload_dir);
        session_service
            .get_session(&input.session_id)
            .await
            .map_err(|e| format!("Session not found: {}", e))?;

        // Decode base64 file data
        let bytes = general_purpose::STANDARD
            .decode(&input.file_data)
            .map_err(|e| format!("Invalid base64 encoding: {}", e))?;

        // Check file size
        if bytes.len() > self.max_file_size {
            return Err(format!(
                "File size {} exceeds maximum of {} bytes",
                bytes.len(),
                self.max_file_size
            ));
        }

        // Sanitize filename
        let filename = sanitize_filename(&input.filename);

        // Validate file format
        let file_format =
            file_processor::validate_file(&bytes, &filename, self.max_file_size as i64)
                .map_err(|e| format!("File validation failed: {}", e))?;

        tracing::info!(
            "MCP: Received file: {} ({} bytes, format: {})",
            filename,
            bytes.len(),
            file_format
        );

        // Create model record
        let mut model = UploadedModel::new(
            input.session_id.clone(),
            filename.clone(),
            file_format.clone(),
            bytes.len() as i64,
            String::new(),
        );

        // Save file to disk
        let file_path = PathBuf::from(&self.upload_dir)
            .join(&input.session_id)
            .join(format!("{}.{}", model.id, file_format));

        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        std::fs::write(&file_path, &bytes).map_err(|e| format!("Failed to save file: {}", e))?;

        model.file_path = file_path.to_string_lossy().to_string();

        // Process file to extract metadata
        let file_path_str = file_path
            .to_str()
            .ok_or_else(|| "Invalid file path".to_string())?;

        let processed = match file_format.as_str() {
            "stl" => file_processor::process_stl_file(file_path_str)
                .map_err(|e| format!("STL processing failed: {}", e))?,
            "3mf" => file_processor::process_3mf_file(file_path_str)
                .map_err(|e| format!("3MF processing failed: {}", e))?,
            _ => return Err(format!("Unsupported file format: {}", file_format)),
        };

        model.volume_cm3 = Some(processed.volume_cm3);
        model.set_dimensions(processed.dimensions_mm);
        model.triangle_count = Some(processed.triangle_count);

        // Save model to database
        models::create(
            &self.pool,
            CreateModel {
                id: &model.id,
                session_id: &model.session_id,
                filename: &model.filename,
                file_format: &model.file_format,
                file_size_bytes: model.file_size_bytes,
                volume_cm3: model.volume_cm3,
                dimensions_mm: model.dimensions_mm.as_deref(),
                triangle_count: model.triangle_count,
                material_id: model.material_id.as_deref(),
                file_path: &model.file_path,
                preview_url: &model.preview_url,
                created_at: model.created_at,
                support_analysis: model.support_analysis.as_deref(),
            },
        )
        .await
        .map_err(|e| format!("Failed to save model: {}", e))?;

        tracing::info!(
            "MCP: Created model {} for session {}",
            model.id,
            input.session_id
        );

        let result = UploadModelResult {
            model_id: model.id,
            filename: model.filename,
            file_format: model.file_format,
            volume_cm3: model.volume_cm3,
            dimensions_mm: model.dimensions_mm.and_then(|dims_json| {
                serde_json::from_str::<serde_json::Value>(&dims_json)
                    .ok()
                    .map(|dims| Dimensions {
                        x: dims["x"].as_f64().unwrap_or(0.0),
                        y: dims["y"].as_f64().unwrap_or(0.0),
                        z: dims["z"].as_f64().unwrap_or(0.0),
                    })
            }),
            triangle_count: model.triangle_count.map(|t| t as i32),
        };

        Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
    }

    /// List all available printing materials with their base prices per cm³
    #[tool(description = "List all available printing materials with prices")]
    async fn list_materials(&self) -> Result<String, String> {
        use crate::persistence::materials;

        let materials_list = materials::list_all_active(&self.pool)
            .await
            .map_err(|e| format!("Failed to fetch materials: {}", e))?;

        let result: Vec<MaterialInfo> = materials_list
            .into_iter()
            .map(|m| MaterialInfo {
                id: m.id,
                name: m.name,
                base_price_per_cm3: m.price_per_cm3,
                active: m.active,
            })
            .collect();

        Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
    }

    /// Configure a model with material selection and quantity
    #[tool(description = "Configure a model with material and quantity")]
    async fn configure_model(
        &self,
        Parameters(input): Parameters<ConfigureModelInput>,
    ) -> Result<String, String> {
        use crate::persistence::{materials, models};

        let model = models::find_by_id_and_session(&self.pool, &input.model_id, &input.session_id)
            .await
            .map_err(|e| format!("Failed to find model: {}", e))?
            .ok_or_else(|| format!("Model {} not found in session", input.model_id))?;

        let material = materials::find_by_id(&self.pool, &input.material_id)
            .await
            .map_err(|e| format!("Failed to find material: {}", e))?
            .ok_or_else(|| format!("Material {} not found", input.material_id))?;

        if !material.active {
            return Err(format!("Material {} is not active", input.material_id));
        }

        models::update_material(&self.pool, &input.model_id, &input.material_id)
            .await
            .map_err(|e| format!("Failed to update model material: {}", e))?;

        let volume = model.volume_cm3.unwrap_or(0.0);
        let unit_price = material.calculate_price(volume);
        let unit_price_f64 = unit_price
            .to_string()
            .parse::<f64>()
            .map_err(|e| format!("Invalid price calculation: {}", e))?;
        let estimated_price = unit_price_f64 * input.quantity as f64;

        tracing::info!(
            "MCP: Configured model {} with material {} ({}), quantity: {}, estimated price: {}€",
            input.model_id,
            material.name,
            input.material_id,
            input.quantity,
            estimated_price
        );

        let result = ConfigureModelResult {
            model_id: input.model_id,
            material_id: input.material_id,
            quantity: input.quantity,
            estimated_price,
        };

        Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
    }

    /// Generate a final quote for all configured models in a session
    #[tool(description = "Generate a complete quote for a session")]
    async fn generate_quote(
        &self,
        Parameters(input): Parameters<GenerateQuoteInput>,
    ) -> Result<String, String> {
        use crate::persistence::{materials, models, quotes};

        let session_models = models::find_by_session(&self.pool, &input.session_id)
            .await
            .map_err(|e| format!("Failed to fetch session models: {}", e))?;

        if session_models.is_empty() {
            return Err("No models found in session".to_string());
        }

        let unconfigured: Vec<_> = session_models
            .iter()
            .filter(|m| m.material_id.is_none())
            .collect();

        if !unconfigured.is_empty() {
            return Err(format!(
                "{} model(s) missing material configuration",
                unconfigured.len()
            ));
        }

        let mut items = Vec::new();
        let mut subtotal = 0.0;

        for model in session_models {
            let material_id = model.material_id.as_ref().ok_or_else(|| {
                format!(
                    "Model {} missing material_id (should not happen after validation)",
                    model.id
                )
            })?;
            let material = materials::find_by_id(&self.pool, material_id)
                .await
                .map_err(|e| format!("Failed to fetch material: {}", e))?
                .ok_or_else(|| format!("Material {} not found", material_id))?;

            let volume = model.volume_cm3.unwrap_or(0.0);
            let unit_price = material.calculate_price(volume);
            let unit_price_f64 = unit_price
                .to_string()
                .parse::<f64>()
                .map_err(|e| format!("Invalid price calculation for model {}: {}", model.id, e))?;
            let quantity = 1;
            let line_total = unit_price_f64 * quantity as f64;

            items.push(QuoteItem {
                model_id: model.id.clone(),
                filename: model.filename.clone(),
                material_name: material.name.clone(),
                quantity,
                unit_price: unit_price_f64,
                line_total,
            });

            subtotal += line_total;
        }

        let quote_id = ulid::Ulid::new().to_string();
        let breakdown = serde_json::to_string(&items).unwrap_or_default();
        quotes::create(
            &self.pool,
            &quote_id,
            &input.session_id,
            subtotal,
            &breakdown,
            "pending",
            chrono::Utc::now().naive_utc(),
        )
        .await
        .map_err(|e| format!("Failed to create quote: {}", e))?;

        tracing::info!(
            "MCP: Generated quote {} for session {} with {} items, total: {}€",
            quote_id,
            input.session_id,
            items.len(),
            subtotal
        );

        let result = QuoteResult {
            quote_id,
            session_id: input.session_id,
            items,
            subtotal,
            total: subtotal,
        };

        Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
    }
}

#[tool_handler]
impl ServerHandler for QuoteTools {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "3D Print Quote Service - Upload models, configure materials, and generate quotes"
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
