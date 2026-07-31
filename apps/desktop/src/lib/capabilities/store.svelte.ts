import { invoke } from '@tauri-apps/api/core';
import type { Capability } from '$lib/types';

class CapabilityStore {
  activeCapabilities = $state<Set<Capability>>(
    new Set<Capability>([
      'TableManagement',
      'SplitBill',
      'MultiPayment',
      'OfflineMode',
      'AuditLog'
    ])
  );
  isLoading = $state<boolean>(false);
  outletId = $state<string>('');

  // Derived helpers for reactive Svelte 5 component UI bindings
  hasTableManagement = $derived(this.activeCapabilities.has('TableManagement'));
  hasSplitBill = $derived(this.activeCapabilities.has('SplitBill'));
  hasRecipeManagement = $derived(this.activeCapabilities.has('RecipeManagement'));
  hasKitchenDisplay = $derived(this.activeCapabilities.has('KitchenDisplay'));
  hasDiningSession = $derived(this.activeCapabilities.has('DiningSession'));
  hasSerialNumberTracking = $derived(this.activeCapabilities.has('SerialNumberTracking'));
  hasBundleDiscount = $derived(this.activeCapabilities.has('BundleDiscount'));
  hasBarcodePrinting = $derived(this.activeCapabilities.has('BarcodePrinting'));
  hasTimeBasedBilling = $derived(this.activeCapabilities.has('TimeBasedBilling'));
  hasDepositManagement = $derived(this.activeCapabilities.has('DepositManagement'));
  hasBookingCalendar = $derived(this.activeCapabilities.has('BookingCalendar'));
  hasMultiPayment = $derived(this.activeCapabilities.has('MultiPayment'));
  hasCustomerLoyalty = $derived(this.activeCapabilities.has('CustomerLoyalty'));
  hasDiscountApproval = $derived(this.activeCapabilities.has('DiscountApproval'));
  hasInventoryTransfer = $derived(this.activeCapabilities.has('InventoryTransfer'));

  async fetchCapabilities(outletId: string = 'default_outlet') {
    this.isLoading = true;
    this.outletId = outletId;

    try {
      const res: any = await invoke('get_effective_capabilities_cmd', { outletId });
      const caps = new Set<Capability>();

      if (Array.isArray(res)) {
        for (const item of res) {
          if (item.enabled) {
            const norm = this.normalizeCapabilityKey(item.key);
            if (norm) caps.add(norm);
          }
        }
      } else if (res && res.enabled_capabilities) {
        const rawCaps = Array.from(res.enabled_capabilities) as string[];
        for (const k of rawCaps) {
          const norm = this.normalizeCapabilityKey(k);
          if (norm) caps.add(norm);
        }
      }

      if (caps.size === 0) {
        caps.add('TableManagement');
        caps.add('SplitBill');
        caps.add('MultiPayment');
        caps.add('OfflineMode');
        caps.add('AuditLog');
      }

      this.activeCapabilities = caps;
    } catch (err) {
      console.warn('Failed to fetch outlet capabilities from backend:', err);
    } finally {
      this.isLoading = false;
    }
  }

  has(cap: Capability): boolean {
    return this.activeCapabilities.has(cap);
  }

  hasAny(caps: Capability[]): boolean {
    return caps.some((c) => this.activeCapabilities.has(c));
  }

  hasAll(caps: Capability[]): boolean {
    return caps.every((c) => this.activeCapabilities.has(c));
  }

  private normalizeCapabilityKey(k: string): Capability | null {
    const clean = k.replace(/[\.\-_]/g, '').toLowerCase();
    const map: Record<string, Capability> = {
      tablemanagement: 'TableManagement',
      fnbtable: 'TableManagement',
      splitbill: 'SplitBill',
      fnbsplitbill: 'SplitBill',
      recipemanagement: 'RecipeManagement',
      recipebom: 'RecipeManagement',
      kitchendisplay: 'KitchenDisplay',
      fnbkds: 'KitchenDisplay',
      diningsession: 'DiningSession',
      serialnumbertracking: 'SerialNumberTracking',
      inventoryserial: 'SerialNumberTracking',
      bundlediscount: 'BundleDiscount',
      promotionbasic: 'BundleDiscount',
      barcodeprinting: 'BarcodePrinting',
      inventorybarcode: 'BarcodePrinting',
      timebasedbilling: 'TimeBasedBilling',
      depositmanagement: 'DepositManagement',
      bookingcalendar: 'BookingCalendar',
      multipayment: 'MultiPayment',
      checkoutbasic: 'MultiPayment',
      customerloyalty: 'CustomerLoyalty',
      memberpatronage: 'CustomerLoyalty',
      discountapproval: 'DiscountApproval',
      inventorytransfer: 'InventoryTransfer',
      inventorybasic: 'InventoryTransfer',
      multioutlet: 'MultiOutlet',
      offlinemode: 'OfflineMode',
      auditlog: 'AuditLog'
    };
    return map[clean] || (k as Capability);
  }
}

export const capabilityStore = new CapabilityStore();
