import { capabilityStore } from './capabilityStore.svelte';
import { getPresetConfig, type SubModuleItem } from './presetRegistry';

export type MenuItem = SubModuleItem;

export function getVisibleMenuItems(): MenuItem[] {
  const activeCode = capabilityStore.activePreset || 'general_flexible';
  const config = getPresetConfig(activeCode);

  return config.menuItems.filter((item) => {
    if (item.requiredCapability && !capabilityStore.hasCapability(item.requiredCapability)) {
      return false;
    }
    return true;
  });
}
