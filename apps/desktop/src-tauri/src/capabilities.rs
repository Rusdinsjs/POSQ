use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDefinition {
    pub key: String,
    pub domain: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessPreset {
    pub code: String,
    pub name: String,
    pub version: i32,
    pub description: String,
    pub default_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveCapabilitySet {
    pub outlet_id: String,
    pub primary_preset_code: String,
    pub enabled_capabilities: HashSet<String>,
}

pub fn get_standard_presets() -> Vec<BusinessPreset> {
    vec![
        BusinessPreset {
            code: "general_flexible".into(),
            name: "Mode Umum & Fleksibel".into(),
            version: 1,
            description: "Modul POS dasar dengan kapabilitas yang dapat diatur manual".into(),
            default_capabilities: vec!["inventory.basic".into(), "checkout.basic".into(), "shift.basic".into()],
        },
        BusinessPreset {
            code: "retail_standard".into(),
            name: "Retail Standard".into(),
            version: 1,
            description: "Toko klontong, fashion, minimarket dasar".into(),
            default_capabilities: vec![
                "inventory.basic".into(),
                "inventory.barcode".into(),
                "checkout.basic".into(),
                "checkout.refund".into(),
                "promotion.basic".into(),
            ],
        },
        BusinessPreset {
            code: "retail_serialized".into(),
            name: "Retail Berserial / Elektronik".into(),
            version: 1,
            description: "Toko HP, laptop, unit dengan nomor seri / IMEI".into(),
            default_capabilities: vec![
                "inventory.basic".into(),
                "inventory.serial".into(),
                "checkout.basic".into(),
                "checkout.trade_in".into(),
                "warranty.tracking".into(),
            ],
        },
        BusinessPreset {
            code: "fnb_quick_service".into(),
            name: "F&B Quick Service / Booth".into(),
            version: 1,
            description: "Kedai kopi, fast food, pesanan langsung bayar".into(),
            default_capabilities: vec![
                "inventory.basic".into(),
                "recipe.bom".into(),
                "fnb.kds".into(),
                "fnb.modifiers".into(),
                "checkout.basic".into(),
            ],
        },
        BusinessPreset {
            code: "fnb_table_service".into(),
            name: "F&B Table Service / Restoran".into(),
            version: 1,
            description: "Restoran meja, bayar belakangan, split bill, KDS".into(),
            default_capabilities: vec![
                "inventory.basic".into(),
                "recipe.bom".into(),
                "fnb.kds".into(),
                "fnb.table".into(),
                "fnb.modifiers".into(),
                "fnb.split_bill".into(),
                "checkout.basic".into(),
            ],
        },
        BusinessPreset {
            code: "nonprofit_donation".into(),
            name: "Yayasan & Penggalangan Dana".into(),
            version: 1,
            description: "Penerimaan donasi, zakat, infaq, kuitansi donatur, tanpa HPP".into(),
            default_capabilities: vec!["donation.receipt".into(), "checkout.basic".into(), "shift.basic".into()],
        },
        BusinessPreset {
            code: "cooperative_member_store".into(),
            name: "Toko Koperasi Anggota".into(),
            version: 1,
            description: "Penjualan anggota, simpanan, poin SHU, harga khusus anggota".into(),
            default_capabilities: vec!["inventory.basic".into(), "member.patronage".into(), "checkout.basic".into(), "shift.basic".into()],
        },
        BusinessPreset {
            code: "public_service_fee".into(),
            name: "Retribusi / Layanan Publik".into(),
            version: 1,
            description: "Penerimaan retribusi, nomor permohonan resmi, larangan diskon".into(),
            default_capabilities: vec!["public_service.receipt".into(), "checkout.no_discount".into(), "shift.basic".into()],
        },
        BusinessPreset {
            code: "internal_issue".into(),
            name: "Pengeluaran Barang Internal".into(),
            version: 1,
            description: "Gudang internal, pantry, cost center, pengeluaran tanpa pendapatan".into(),
            default_capabilities: vec!["inventory.basic".into(), "internal.issue".into()],
        },
        BusinessPreset {
            code: "school_campus".into(),
            name: "Kantin / Toko Sekolah".into(),
            version: 1,
            description: "Kantin sekolah, batas uang saku harian siswa, pembatasan item".into(),
            default_capabilities: vec!["inventory.basic".into(), "student.allowance".into(), "checkout.basic".into(), "shift.basic".into()],
        },
        BusinessPreset {
            code: "parking".into(),
            name: "Parkir & Gate Sistem".into(),
            version: 1,
            description: "Manajemen parkir, tiket gate masuk/keluar, tarif progresif durasi".into(),
            default_capabilities: vec!["parking.gate".into(), "parking.tariff".into(), "shift.basic".into()],
        },
        BusinessPreset {
            code: "fuel_energy".into(),
            name: "SPBU / EV Charging".into(),
            version: 1,
            description: "Stasiun pengisian bahan bakar / listrik, pencatatan meter dispenser".into(),
            default_capabilities: vec!["fuel.dispenser".into(), "fuel.reconciliation".into(), "checkout.basic".into(), "shift.basic".into()],
        },
    ]
}

pub async fn resolve_effective_capabilities(
    pool: &Pool<Sqlite>,
    outlet_id: &str,
) -> Result<EffectiveCapabilitySet, String> {
    // Read outlet profile
    let profile_row = sqlx::query("SELECT primary_preset_code FROM outlet_profiles WHERE outlet_id = ?")
        .bind(outlet_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

    let preset_code = match profile_row {
        Some(r) => r.get::<String, _>("primary_preset_code"),
        None => "general_flexible".to_string(),
    };

    // Find default capabilities for preset
    let presets = get_standard_presets();
    let selected_preset = presets
        .iter()
        .find(|p| p.code == preset_code)
        .cloned()
        .unwrap_or_else(|| presets[0].clone());

    let mut enabled_caps: HashSet<String> = selected_preset.default_capabilities.into_iter().collect();

    // Apply database overrides from outlet_capabilities
    let overrides = sqlx::query("SELECT capability_key, enabled FROM outlet_capabilities WHERE outlet_id = ?")
        .bind(outlet_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    for row in overrides {
        let key: String = row.get("capability_key");
        let enabled: i32 = row.get("enabled");
        if enabled == 1 {
            enabled_caps.insert(key);
        } else {
            enabled_caps.remove(&key);
        }
    }

    Ok(EffectiveCapabilitySet {
        outlet_id: outlet_id.to_string(),
        primary_preset_code: preset_code,
        enabled_capabilities: enabled_caps,
    })
}

pub fn check_capability_guard(
    active_caps: &HashSet<String>,
    required_capability: &str,
) -> Result<(), String> {
    if active_caps.contains(required_capability) {
        Ok(())
    } else {
        Err(format!(
            "CAPABILITY_DISABLED: Kapabilitas '{}' tidak aktif untuk outlet ini",
            required_capability
        ))
    }
}

// Tauri commands
#[tauri::command]
pub async fn get_effective_capabilities_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    outlet_id: String,
) -> Result<EffectiveCapabilitySet, String> {
    resolve_effective_capabilities(&pool, &outlet_id).await
}

#[tauri::command]
pub async fn get_available_presets_cmd() -> Result<Vec<BusinessPreset>, String> {
    Ok(get_standard_presets())
}

#[tauri::command]
pub async fn set_outlet_preset_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    outlet_id: String,
    preset_code: String,
    user_id: String,
) -> Result<EffectiveCapabilitySet, String> {
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO outlet_profiles (outlet_id, primary_preset_code, preset_version, config_version, activated_at, activated_by)
         VALUES (?, ?, 1, 1, ?, ?)
         ON CONFLICT(outlet_id) DO UPDATE SET
         primary_preset_code = excluded.primary_preset_code,
         config_version = config_version + 1,
         activated_at = excluded.activated_at,
         activated_by = excluded.activated_by"
    )
    .bind(&outlet_id)
    .bind(&preset_code)
    .bind(&now)
    .bind(&user_id)
    .execute(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    resolve_effective_capabilities(&pool, &outlet_id).await
}

#[tauri::command]
pub async fn toggle_outlet_capability_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    outlet_id: String,
    capability_key: String,
    enabled: bool,
    user_id: String,
) -> Result<EffectiveCapabilitySet, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let enabled_int = if enabled { 1 } else { 0 };

    sqlx::query(
        "INSERT INTO outlet_capabilities (outlet_id, capability_key, enabled, source, updated_at, updated_by)
         VALUES (?, ?, ?, 'user_override', ?, ?)
         ON CONFLICT(outlet_id, capability_key) DO UPDATE SET
         enabled = excluded.enabled,
         source = 'user_override',
         updated_at = excluded.updated_at,
         updated_by = excluded.updated_by"
    )
    .bind(&outlet_id)
    .bind(&capability_key)
    .bind(enabled_int)
    .bind(&now)
    .bind(&user_id)
    .execute(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    resolve_effective_capabilities(&pool, &outlet_id).await
}
