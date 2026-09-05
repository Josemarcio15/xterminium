export interface SshHost {
  id: string;
  label?: string;
  user: string;
  ip: string;
  port?: string;
  key?: string;
}

export interface ShortcutAction {
  id: string;
  label: string;
  keys: string;
}

export interface SavedPath {
  id: string;
  name: string;
  path: string;
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
