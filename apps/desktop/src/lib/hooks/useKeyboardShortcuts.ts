export interface ShortcutConfig {
  onCategorySelect?: (index: number) => void;
  onOpenPayment?: () => void;
  onFocusSearch?: () => void;
  onEscape?: () => void;
  onEnter?: () => void;
}

/**
 * Helper function to check if the user is currently typing in an input element
 */
export function isTypingInInput(event: KeyboardEvent): boolean {
  const activeEl = document.activeElement;
  if (!activeEl) return false;

  const tagName = activeEl.tagName.toLowerCase();
  const isInputOrTextarea = tagName === 'input' || tagName === 'textarea' || tagName === 'select';
  const isEditable =
    activeEl.getAttribute('contenteditable') === 'true' ||
    (activeEl as HTMLElement).isContentEditable === true;

  return isInputOrTextarea || isEditable;
}

/**
 * Composible function for keyboard shortcuts system in POSQ
 */
export function useKeyboardShortcuts(config: ShortcutConfig) {
  function handleKeyDown(event: KeyboardEvent) {
    // Escape key handler: always allowed even inside input to close modal or cancel
    if (event.key === 'Escape') {
      if (config.onEscape) {
        event.preventDefault();
        config.onEscape();
        return;
      }
    }

    // Bypass hotkeys if user is actively typing in text inputs
    if (isTypingInInput(event)) {
      return;
    }

    // F1 - F9: Quick Category Selection (Index 0 - 8)
    if (event.key.startsWith('F') && event.key.length === 2) {
      const num = parseInt(event.key.substring(1), 10);
      if (num >= 1 && num <= 9) {
        if (config.onCategorySelect) {
          event.preventDefault();
          config.onCategorySelect(num - 1);
          return;
        }
      }
    }

    // F10 or Ctrl + K: Open Payment Modal
    if (
      event.key === 'F10' ||
      ((event.ctrlKey || event.metaKey) && (event.key === 'k' || event.key === 'K'))
    ) {
      if (config.onOpenPayment) {
        event.preventDefault();
        config.onOpenPayment();
        return;
      }
    }

    // Ctrl + F or F2: Focus Search Bar
    if (
      event.key === 'F2' ||
      ((event.ctrlKey || event.metaKey) && (event.key === 'f' || event.key === 'F'))
    ) {
      if (config.onFocusSearch) {
        event.preventDefault();
        config.onFocusSearch();
        return;
      }
    }

    // Enter: Confirm Action
    if (event.key === 'Enter') {
      if (config.onEnter) {
        event.preventDefault();
        config.onEnter();
        return;
      }
    }
  }

  window.addEventListener('keydown', handleKeyDown);
  return () => {
    window.removeEventListener('keydown', handleKeyDown);
  };
}
