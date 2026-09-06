import { invoke } from '@tauri-apps/api/core';
import { 
  type SshHost, 
  type SavedPath, 
  type CustomCommand, 
  type AppTheme,
  defaultShortcuts, 
  defaultCustomCommands,
  defaultTheme,
} from '../types';


export class ConfigService {
  // Hosts SSH
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

  // Atalhos de Teclado
  private static cachedShortcuts: Record<string, string> | null = null;

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

  // Caminhos Salvos (Paths)
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

  // Comandos Customizados
  private static cachedCommands: CustomCommand[] | null = null;

  static async loadCustomCommands(): Promise<CustomCommand[]> {
    if (this.cachedCommands) {
      return this.cachedCommands;
    }
    try {
      const content = await invoke<string>('load_config', { filename: 'commands' });
      if (content && content.trim()) {
        const parsed = JSON.parse(content);
        if (Array.isArray(parsed) && parsed.length > 0) {
          this.cachedCommands = parsed;
          return parsed;
        }
      }
    } catch (e) {
      console.error('Erro ao ler ~/.config/xterminium/commands.json', e);
    }

    const defaults = [...defaultCustomCommands];
    this.cachedCommands = defaults;
    return defaults;
  }

  static async saveCustomCommands(commands: CustomCommand[]): Promise<void> {
    this.cachedCommands = [...commands];
    try {
      const json = JSON.stringify(commands, null, 2);
      await invoke('save_config', { filename: 'commands', content: json });
    } catch (e) {
      console.error('Erro ao salvar ~/.config/xterminium/commands.json', e);
    }
  }

  // Tema
  static async loadTheme(): Promise<AppTheme> {
    try {
      const content = await invoke<string>('load_config', { filename: 'theme' });
      if (content && content.trim()) {
        return { ...defaultTheme, ...JSON.parse(content) };
      }
    } catch (e) {
      console.error('Erro ao ler ~/.config/xterminium/theme.json', e);
    }
    return { ...defaultTheme };
  }

  static async saveTheme(theme: AppTheme): Promise<void> {
    try {
      const json = JSON.stringify(theme, null, 2);
      await invoke('save_config', { filename: 'theme', content: json });
    } catch (e) {
      console.error('Erro ao salvar ~/.config/xterminium/theme.json', e);
    }
  }

  // Temas Customizados
  static async loadCustomThemes(): Promise<AppTheme[]> {
    try {
      const content = await invoke<string>('load_config', { filename: 'custom_themes' });
      if (content && content.trim()) {
        return JSON.parse(content);
      }
    } catch (e) {
      console.error('Erro ao ler ~/.config/xterminium/custom_themes.json', e);
    }
    return [];
  }

  static async saveCustomThemes(themes: AppTheme[]): Promise<void> {
    try {
      const json = JSON.stringify(themes, null, 2);
      await invoke('save_config', { filename: 'custom_themes', content: json });
    } catch (e) {
      console.error('Erro ao salvar ~/.config/xterminium/custom_themes.json', e);
    }
  }
}

