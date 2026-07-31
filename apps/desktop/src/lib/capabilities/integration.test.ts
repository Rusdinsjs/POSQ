import { capabilityStore } from './store.svelte';
import { canUseCapability, canUseAnyCapability, canUseAllCapabilities } from './helpers';

/**
 * Capability Kernel Integration Tests
 */
export function runCapabilityTests() {
  capabilityStore.activeCapabilities = new Set([
    'TableManagement',
    'SplitBill',
    'MultiPayment',
    'OfflineMode'
  ]);

  console.assert(canUseCapability('TableManagement') === true, 'TableManagement should be true');
  console.assert(canUseCapability('SplitBill') === true, 'SplitBill should be true');
  console.assert(canUseCapability('BarcodePrinting') === false, 'BarcodePrinting should be false');
  console.assert(canUseAllCapabilities(['TableManagement', 'SplitBill']) === true, 'All should be true');
  console.assert(canUseAllCapabilities(['TableManagement', 'BarcodePrinting']) === false, 'All should be false');
  console.assert(canUseAnyCapability(['BarcodePrinting', 'SplitBill']) === true, 'Any should be true');
}
