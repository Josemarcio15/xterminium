export interface ShortcutAction {
  id: string;
  label: string;
  keys: string;
}

export const defaultShortcuts: Record<string, string> = {
  copy: 'Ctrl+Shift+C',
  paste: 'Ctrl+Shift+V',
  selectAll: 'Ctrl+Shift+A',
  autocomplete: 'Ctrl+Space',
  stop: 'Ctrl+C',
  newTab: 'Ctrl+Shift+T',
  newWindow: 'Ctrl+Shift+N',
};
