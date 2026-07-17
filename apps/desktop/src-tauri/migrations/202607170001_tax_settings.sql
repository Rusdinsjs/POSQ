-- D-2: Configurable tax/service rules (DEC-041, PDEC-021)
-- Tax is disabled by default; merchant must enable and configure the rate.

INSERT OR IGNORE INTO system_settings (key, value) VALUES ('tax_enabled', 'false');
INSERT OR IGNORE INTO system_settings (key, value) VALUES ('tax_rate', '0.11');
INSERT OR IGNORE INTO system_settings (key, value) VALUES ('service_charge_enabled', 'false');
INSERT OR IGNORE INTO system_settings (key, value) VALUES ('service_charge_rate', '0.05');
INSERT OR IGNORE INTO system_settings (key, value) VALUES ('cp_api_url', '');
