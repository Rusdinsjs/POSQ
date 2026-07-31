export interface ShortcutHandlers {
  onCategorySelect?: (index: number) => void;
  onOpenPayment?: () => void;
  onFocusSearch?: () => void;
  onEscape?: () => void;
  onEnter?: () => void;
}

/**
  Helper function to check if user is currently typing in an input field
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
 * Svelte 5 Custom Hook for POS Keyboard Shortcuts System
 */
export function useKeyboardShortcuts(handlers: ShortcutHandlers) {
  $effect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      // Escape key handler (Always allowed even inside input to dismiss/cancel)
      if (event.key === 'Escape') {
        if (handlers.onEscape) {
          event.preventDefault();
          handlers.onEscape();
          return;
        }
      }

      // Ignore all other global hotkeys if the user is typing in an input/textarea
      if (isTypingInInput(event)) {
        return;
      }

      // F1 - F9: Quick Category Selection (Index 0 - 8)
      if (event.key.startsWith('F') && event.key.length === 2) {
        const num = parseInt(event.key.substring(1), 10);
        if (num >= 1 && num <= 9) {
          if (handlers.onCategorySelect) {
            event.preventDefault();
            handlers.onCategorySelect(num - 1);
            return;
          }
        }
      }

      // F10 or Ctrl + K: Open Payment Modal
      if (
        event.key === 'F10' ||
        ((event.ctrlKey || event.metaKey) && (event.key === 'k' || event.key === 'K'))
      ) {
        if (handlers.onOpenPayment) {
          event.preventDefault();
          handlers.onOpenPayment();
          return;
        }
      }

      // Ctrl + F or F2: Focus Search Bar
      if (
        event.key === 'F2' ||
        ((event.ctrlKey || event.metaKey) && (event.key === 'f' || event.key === 'F'))
      ) {
        if (handlers.onFocusSearch) {
          event.preventDefault();
          handlers.onFocusSearch();
          return;
        }
      }

      // Enter: Confirm Action
      if (event.key === 'Enter') {
        if (handlers.onEnter) {
          event.preventDefault();
          handlers.onEnter();
          return;
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  });
}
