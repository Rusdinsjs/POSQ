use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    // F&B Domain
    TableManagement,
    SplitBill,
    RecipeManagement,
    KitchenDisplay,
    DiningSession,

    // Retail Domain
    SerialNumberTracking,
    BundleDiscount,
    BarcodePrinting,

    // Service/Rental Domain
    TimeBasedBilling,
    DepositManagement,
    BookingCalendar,

    // General/Commerce
    MultiPayment,
    CustomerLoyalty,
    DiscountApproval,
    InventoryTransfer,
    MultiOutlet,
    OfflineMode,
    AuditLog,
}

impl Capability {
    pub fn description(&self) -> &'static str {
        match self {
            Capability::TableManagement => "Manajemen Meja & Denah Restoran",
            Capability::SplitBill => "Pemisahan Tagihan (Split Bill)",
            Capability::RecipeManagement => "Manajemen Resep & HPP Bahan Baku",
            Capability::KitchenDisplay => "Sistem Layar Dapur / KDS Ticket",
            Capability::DiningSession => "Sesi Makan Di Tempat",
            Capability::SerialNumberTracking => "Pelacakan Nomor Seri & IMEI Produk",
            Capability::BundleDiscount => "Diskon Paket / Bundling Produk",
            Capability::BarcodePrinting => "Cetak Label Barcode",
            Capability::TimeBasedBilling => "Penagihan Berbasis Durasi/Waktu",
            Capability::DepositManagement => "Manajemen Uang Muka / Deposit",
            Capability::BookingCalendar => "Kalender Pemesanan & Jadwal Reservasi",
            Capability::MultiPayment => "Pembayaran Ganda / Split Payment",
            Capability::CustomerLoyalty => "Poin Pelanggan & Program Loyalitas",
            Capability::DiscountApproval => "Persetujuan Diskon Khusus Supervisor",
            Capability::InventoryTransfer => "Transfer Stok Antar Cabang",
            Capability::MultiOutlet => "Manajemen Banyak Cabang",
            Capability::OfflineMode => "Operasional Offline-First",
            Capability::AuditLog => "Log Audit & Jejak Aktivitas",
        }
    }

    pub fn domain(&self) -> &'static str {
        match self {
            Capability::TableManagement
            | Capability::SplitBill
            | Capability::RecipeManagement
            | Capability::KitchenDisplay
            | Capability::DiningSession => "fnb",

            Capability::SerialNumberTracking
            | Capability::BundleDiscount
            | Capability::BarcodePrinting => "retail",

            Capability::TimeBasedBilling
            | Capability::DepositManagement
            | Capability::BookingCalendar => "service",

            Capability::MultiPayment
            | Capability::CustomerLoyalty
            | Capability::DiscountApproval
            | Capability::InventoryTransfer
            | Capability::MultiOutlet
            | Capability::OfflineMode
            | Capability::AuditLog => "general",
        }
    }

    pub fn all_variants() -> Vec<Capability> {
        vec![
            Capability::TableManagement,
            Capability::SplitBill,
            Capability::RecipeManagement,
            Capability::KitchenDisplay,
            Capability::DiningSession,
            Capability::SerialNumberTracking,
            Capability::BundleDiscount,
            Capability::BarcodePrinting,
            Capability::TimeBasedBilling,
            Capability::DepositManagement,
            Capability::BookingCalendar,
            Capability::MultiPayment,
            Capability::CustomerLoyalty,
            Capability::DiscountApproval,
            Capability::InventoryTransfer,
            Capability::MultiOutlet,
            Capability::OfflineMode,
            Capability::AuditLog,
        ]
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Capability::TableManagement => "table_management",
            Capability::SplitBill => "split_bill",
            Capability::RecipeManagement => "recipe_management",
            Capability::KitchenDisplay => "kitchen_display",
            Capability::DiningSession => "dining_session",
            Capability::SerialNumberTracking => "serial_number_tracking",
            Capability::BundleDiscount => "bundle_discount",
            Capability::BarcodePrinting => "barcode_printing",
            Capability::TimeBasedBilling => "time_based_billing",
            Capability::DepositManagement => "deposit_management",
            Capability::BookingCalendar => "booking_calendar",
            Capability::MultiPayment => "multi_payment",
            Capability::CustomerLoyalty => "customer_loyalty",
            Capability::DiscountApproval => "discount_approval",
            Capability::InventoryTransfer => "inventory_transfer",
            Capability::MultiOutlet => "multi_outlet",
            Capability::OfflineMode => "offline_mode",
            Capability::AuditLog => "audit_log",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Capability {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean = s.to_lowercase().replace('.', "_").replace('-', "_");
        match clean.as_str() {
            "table_management" | "tablemanagement" | "fnb_table" => Ok(Capability::TableManagement),
            "split_bill" | "splitbill" | "fnb_split_bill" => Ok(Capability::SplitBill),
            "recipe_management" | "recipemanagement" | "recipe_bom" => Ok(Capability::RecipeManagement),
            "kitchen_display" | "kitchendisplay" | "fnb_kds" => Ok(Capability::KitchenDisplay),
            "dining_session" | "diningsession" => Ok(Capability::DiningSession),
            "serial_number_tracking" | "serialnumbertracking" | "inventory_serial" => {
                Ok(Capability::SerialNumberTracking)
            }
            "bundle_discount" | "bundlediscount" | "promotion_basic" => Ok(Capability::BundleDiscount),
            "barcode_printing" | "barcodeprinting" | "inventory_barcode" => {
                Ok(Capability::BarcodePrinting)
            }
            "time_based_billing" | "timebasedbilling" => Ok(Capability::TimeBasedBilling),
            "deposit_management" | "depositmanagement" => Ok(Capability::DepositManagement),
            "booking_calendar" | "bookingcalendar" => Ok(Capability::BookingCalendar),
            "multi_payment" | "multipayment" | "checkout_basic" => Ok(Capability::MultiPayment),
            "customer_loyalty" | "customerloyalty" | "member_patronage" => {
                Ok(Capability::CustomerLoyalty)
            }
            "discount_approval" | "discountapproval" => Ok(Capability::DiscountApproval),
            "inventory_transfer" | "inventorytransfer" | "inventory_basic" => {
                Ok(Capability::InventoryTransfer)
            }
            "multi_outlet" | "multioutlet" => Ok(Capability::MultiOutlet),
            "offline_mode" | "offlinemode" => Ok(Capability::OfflineMode),
            "audit_log" | "auditlog" => Ok(Capability::AuditLog),
            _ => Err(format!("Unknown capability string: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDTO {
    pub key: String,
    pub domain: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveCapabilitySet {
    pub outlet_id: String,
    pub enabled_capabilities: HashSet<String>,
}

/// Core function to check if an outlet has a capability enabled in DB
pub async fn has_capability(
    pool: &SqlitePool,
    outlet_id: &str,
    cap: Capability,
) -> Result<bool, String> {
    let key = cap.to_string();
    let row = sqlx::query(
        "SELECT enabled FROM outlet_capabilities WHERE outlet_id = ? AND (capability = ? OR capability_key = ?) LIMIT 1"
    )
    .bind(outlet_id)
    .bind(&key)
    .bind(&key)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(r) = row {
        let enabled: i64 = r.get("enabled");
        Ok(enabled == 1)
    } else {
        // Default general capabilities to enabled for seamless fallback
        Ok(matches!(
            cap,
            Capability::MultiPayment | Capability::OfflineMode | Capability::AuditLog
        ))
    }
}

/// Enforce a capability server-side. Returns Err if disabled.
pub async fn enforce_capability(
    pool: &SqlitePool,
    outlet_id: &str,
    cap: Capability,
) -> Result<(), String> {
    if has_capability(pool, outlet_id, cap).await? {
        Ok(())
    } else {
        Err(format!(
            "CAPABILITY_DISABLED: Kapabilitas '{}' ({}) tidak aktif untuk outlet ini",
            cap.to_string(),
            cap.description()
        ))
    }
}

/// Fetch all effective capabilities for an outlet
pub async fn get_outlet_capabilities(
    pool: &SqlitePool,
    outlet_id: &str,
) -> Result<Vec<CapabilityDTO>, String> {
    let active_rows = sqlx::query(
        "SELECT capability, capability_key, enabled FROM outlet_capabilities WHERE outlet_id = ?"
    )
    .bind(outlet_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut enabled_map: HashSet<String> = HashSet::new();
    for r in active_rows {
        let cap_str: String = r.get::<Option<String>, _>("capability").unwrap_or_else(|| r.get("capability_key"));
        let enabled: i64 = r.get("enabled");
        if enabled == 1 {
            enabled_map.insert(cap_str);
        }
    }

    let mut dtos = Vec::new();
    for cap in Capability::all_variants() {
        let key = cap.to_string();
        let is_enabled = enabled_map.contains(&key)
            || (enabled_map.is_empty()
                && matches!(
                    cap,
                    Capability::MultiPayment
                        | Capability::OfflineMode
                        | Capability::AuditLog
                        | Capability::TableManagement
                        | Capability::SplitBill
                ));

        dtos.push(CapabilityDTO {
            key,
            domain: cap.domain().to_string(),
            description: cap.description().to_string(),
            enabled: is_enabled,
        });
    }

    Ok(dtos)
}

// Tauri commands
#[tauri::command]
pub async fn get_effective_capabilities_cmd(
    pool: tauri::State<'_, SqlitePool>,
    outlet_id: String,
) -> Result<Vec<CapabilityDTO>, String> {
    get_outlet_capabilities(&pool, &outlet_id).await
}

#[tauri::command]
pub async fn toggle_outlet_capability_cmd(
    pool: tauri::State<'_, SqlitePool>,
    outlet_id: String,
    capability_key: String,
    enabled: bool,
    user_id: String,
) -> Result<Vec<CapabilityDTO>, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let enabled_int = if enabled { 1 } else { 0 };

    sqlx::query(
        "INSERT INTO outlet_capabilities (outlet_id, capability, enabled, granted_at, granted_by)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(outlet_id, capability) DO UPDATE SET
         enabled = excluded.enabled,
         granted_at = excluded.granted_at,
         granted_by = excluded.granted_by"
    )
    .bind(&outlet_id)
    .bind(&capability_key)
    .bind(enabled_int)
    .bind(&now)
    .bind(&user_id)
    .execute(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    get_outlet_capabilities(&pool, &outlet_id).await
}
