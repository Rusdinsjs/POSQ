import { invoke } from '@tauri-apps/api/core';

export interface EffectiveCapabilitySet {
  outlet_id: string;
  primary_preset_code: string;
  enabled_capabilities: string[];
}

export interface BusinessPreset {
  code: string;
  name: string;
  version: number;
  description: string;
  default_capabilities: string[];
}

class CapabilityStore {
  activePreset = $state<string>('general_flexible');
  capabilities = $state<Set<string>>(new Set(['inventory.basic', 'checkout.basic', 'shift.basic']));
  availablePresets = $state<BusinessPreset[]>([]);
  isLoading = $state<boolean>(false);

  async loadCapabilities(outletId: string = 'default_outlet') {
    this.isLoading = true;
    try {
      const result = await invoke<EffectiveCapabilitySet>('get_effective_capabilities_cmd', { outletId });
      this.activePreset = result.primary_preset_code;
      this.capabilities = new Set(result.enabled_capabilities);
    } catch (e) {
      console.warn('Gagal memuat kapabilitas outlet:', e);
    } finally {
      this.isLoading = false;
    }
  }

  async fetchAvailablePresets() {
    try {
      this.availablePresets = await invoke<BusinessPreset[]>('get_available_presets_cmd');
    } catch (e) {
      console.error('Gagal mengambil daftar preset:', e);
    }
  }

  async changePreset(outletId: string, presetCode: string, userId: string = 'system') {
    this.isLoading = true;
    try {
      const result = await invoke<EffectiveCapabilitySet>('set_outlet_preset_cmd', {
        outletId,
        presetCode,
        userId,
      });
      this.activePreset = result.primary_preset_code;
      this.capabilities = new Set(result.enabled_capabilities);
    } finally {
      this.isLoading = false;
    }
  }

  hasCapability(capabilityKey: string): boolean {
    return this.capabilities.has(capabilityKey);
  }
}

export const capabilityStore = new CapabilityStore();
