import { invoke } from '@tauri-apps/api/core';
import { type SshHost, type SavedPath, defaultShortcuts } from './types';

export class ConfigService {
  // Carrega hosts SSH de ~/.config/xterminium/ssh.json
  static async loadSshHosts(): Promise<SshHost[]> {
    try {
      const content = await invoke<string>('load_config', { filename: 'ssh' });
      if (content && content.trim()) {
        return JSON.parse(content);
      }
    } catch (e) {
      console.error('Erro ao ler ~/.config/xterminium/ssh.json', e);
    }
    return [];
  }

  static async saveSshHosts(hosts: SshHost[]): Promise<void> {
    try {
      const json = JSON.stringify(hosts, null, 2);
      await invoke('save_config', { filename: 'ssh', content: json });
    } catch (e) {
      console.error('Erro ao salvar ~/.config/xterminium/ssh.json', e);
    }
  }

  private static cachedShortcuts: Record<string, string> | null = null;

  // Carrega atalhos de ~/.config/xterminium/shortcuts.json
  static async loadShortcuts(): Promise<Record<string, string>> {
    if (this.cachedShortcuts) {
      return this.cachedShortcuts;
    }
    try {
      const content = await invoke<string>('load_config', { filename: 'shortcuts' });
      if (content && content.trim()) {
        const parsed = { ...defaultShortcuts, ...JSON.parse(content) };
        this.cachedShortcuts = parsed;
        return parsed;
      }
    } catch (e) {
      console.error('Erro ao ler ~/.config/xterminium/shortcuts.json', e);
    }

    const defaults = { ...defaultShortcuts };
    this.cachedShortcuts = defaults;
    return defaults;
  }

  static async saveShortcuts(shortcuts: Record<string, string>): Promise<void> {
    this.cachedShortcuts = { ...shortcuts };
    try {
      const json = JSON.stringify(shortcuts, null, 2);
      await invoke('save_config', { filename: 'shortcuts', content: json });
    } catch (e) {
      console.error('Erro ao salvar ~/.config/xterminium/shortcuts.json', e);
    }
  }

  // Carrega caminhos salvos de ~/.config/xterminium/paths.json
  static async loadPaths(): Promise<SavedPath[]> {
    try {
      const content = await invoke<string>('load_config', { filename: 'paths' });
      if (content && content.trim()) {
        return JSON.parse(content);
      }
    } catch (e) {
      console.error('Erro ao ler ~/.config/xterminium/paths.json', e);
    }
    return [];
  }

  static async savePaths(paths: SavedPath[]): Promise<void> {
    try {
      const json = JSON.stringify(paths, null, 2);
      await invoke('save_config', { filename: 'paths', content: json });
    } catch (e) {
      console.error('Erro ao salvar ~/.config/xterminium/paths.json', e);
    }
  }
}

