use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedScaleBarcode {
    pub sku: String,
    pub weight_grams: i32,
    pub weight_kg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryLot {
    pub id: String,
    pub product_id: String,
    pub lot_number: String,
    pub expiry_date: String,
    pub qty_on_hand: f64,
    pub supplier_id: Option<String>,
}

pub fn parse_embedded_scale_barcode(barcode: &str) -> Result<ParsedScaleBarcode, String> {
    // Standard EAN-13 weighted barcode format: 20 SSSSS WWWWW C
    // Length: 13 digits, prefix '20'
    let trimmed = barcode.trim();
    if trimmed.len() != 13 || (!trimmed.starts_with("20") && !trimmed.starts_with("21")) {
        return Err("INVALID_SCALE_BARCODE: Barcode bukan format timbangan EAN-13 (prefix 20/21)".into());
    }

    let sku = trimmed[2..7].to_string();
    let weight_grams: i32 = trimmed[7..12]
        .parse()
        .map_err(|_| "INVALID_WEIGHT_VALUE: Gagal membaca nilai berat".to_string())?;

    let weight_kg = weight_grams as f64 / 1000.0;

    Ok(ParsedScaleBarcode {
        sku,
        weight_grams,
        weight_kg,
    })
}

pub async fn select_fefo_lot(
    pool: &Pool<Sqlite>,
    product_id: &str,
    needed_qty: f64,
) -> Result<Vec<InventoryLot>, String> {
    let rows = sqlx::query(
        "SELECT id, product_id, lot_number, expiry_date, qty_on_hand, supplier_id 
         FROM inventory_lots 
         WHERE product_id = ? AND qty_on_hand > 0 
         ORDER BY expiry_date ASC"
    )
    .bind(product_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut selected = Vec::new();
    let mut remaining = needed_qty;

    for r in rows {
        let qty: f64 = r.get("qty_on_hand");
        let lot = InventoryLot {
            id: r.get("id"),
            product_id: r.get("product_id"),
            lot_number: r.get("lot_number"),
            expiry_date: r.get("expiry_date"),
            qty_on_hand: qty,
            supplier_id: r.get("supplier_id"),
        };

        selected.push(lot);
        remaining -= qty;
        if remaining <= 0.0 {
            break;
        }
    }

    if remaining > 0.0 {
        return Err(format!("INSUFFICIENT_LOT_STOCK: Stok lot FEFO kurang {:.2} unit", remaining));
    }

    Ok(selected)
}

pub async fn verify_customer_credit_limit(
    pool: &Pool<Sqlite>,
    customer_id: &str,
    additional_ar: i32,
) -> Result<(), String> {
    let row = sqlx::query("SELECT credit_limit, current_ar FROM customer_credit_limits WHERE customer_id = ?")
        .bind(customer_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(r) = row {
        let limit: i32 = r.get("credit_limit");
        let current_ar: i32 = r.get("current_ar");

        if current_ar + additional_ar > limit {
            return Err(format!(
                "CREDIT_LIMIT_EXCEEDED: Batas piutang (Rp {}) terlampaui. Total piutang baru (Rp {})",
                limit, current_ar + additional_ar
            ));
        }
    }

    Ok(())
}

// Tauri commands
#[tauri::command]
pub fn parse_scale_barcode_cmd(barcode: String) -> Result<ParsedScaleBarcode, String> {
    parse_embedded_scale_barcode(&barcode)
}

#[tauri::command]
pub async fn register_inventory_lot_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    product_id: String,
    lot_number: String,
    expiry_date: String,
    qty: f64,
    supplier_id: Option<String>,
) -> Result<InventoryLot, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO inventory_lots (id, product_id, lot_number, expiry_date, qty_on_hand, supplier_id, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&product_id)
    .bind(&lot_number)
    .bind(&expiry_date)
    .bind(qty)
    .bind(&supplier_id)
    .bind(&now)
    .execute(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(InventoryLot {
        id,
        product_id,
        lot_number,
        expiry_date,
        qty_on_hand: qty,
        supplier_id,
    })
}
