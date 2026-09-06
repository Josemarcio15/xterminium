import { 
  type SshHost, 
  type CustomCommand, 
  type SavedPath,
  type AppTheme,
  defaultCustomCommands,
  defaultShortcuts,
  defaultTheme,
  applyThemeToDom,
} from '../types';
import { ConfigService } from '../services/config.service';

export class ConfigStore {
  hosts = $state<SshHost[]>([]);
  commands = $state<CustomCommand[]>([]);
  shortcuts = $state<Record<string, string>>({ ...defaultShortcuts });
  paths = $state<SavedPath[]>([]);
  theme = $state<AppTheme>({ ...defaultTheme });
  customThemes = $state<AppTheme[]>([]);
  initialized = $state(false);

  async init() {
    if (this.initialized) return;
    const [h, c, s, p, t, ct] = await Promise.all([
      ConfigService.loadSshHosts(),
      ConfigService.loadCustomCommands(),
      ConfigService.loadShortcuts(),
      ConfigService.loadPaths(),
      ConfigService.loadTheme(),
      ConfigService.loadCustomThemes(),
    ]);
    this.hosts = h;
    this.commands = c;
    this.shortcuts = s;
    this.paths = p;
    this.theme = t;
    this.customThemes = ct;
    applyThemeToDom(t);
    this.initialized = true;
  }

  // SSH Hosts
  async addHost(host: SshHost) {
    this.hosts.push(host);
    await ConfigService.saveSshHosts(this.hosts);
  }

  async updateHost(host: SshHost) {
    const idx = this.hosts.findIndex((h) => h.id === host.id);
    if (idx !== -1) {
      this.hosts[idx] = host;
      await ConfigService.saveSshHosts(this.hosts);
    }
  }

  async removeHost(id: string) {
    this.hosts = this.hosts.filter((h) => h.id !== id);
    await ConfigService.saveSshHosts(this.hosts);
  }

  // Custom Commands
  async addCommand(cmd: CustomCommand) {
    this.commands.push(cmd);
    await ConfigService.saveCustomCommands(this.commands);
  }

  async updateCommand(cmd: CustomCommand) {
    const idx = this.commands.findIndex((c) => c.id === cmd.id);
    if (idx !== -1) {
      this.commands[idx] = cmd;
      await ConfigService.saveCustomCommands(this.commands);
    }
  }

  async removeCommand(id: string) {
    this.commands = this.commands.filter((c) => c.id !== id);
    await ConfigService.saveCustomCommands(this.commands);
  }

  async resetCommands() {
    this.commands = [...defaultCustomCommands];
    await ConfigService.saveCustomCommands(this.commands);
  }

  // Shortcuts
  async updateShortcut(actionId: string, keys: string) {
    this.shortcuts[actionId] = keys;
    await ConfigService.saveShortcuts(this.shortcuts);
  }

  async resetShortcuts() {
    this.shortcuts = { ...defaultShortcuts };
    await ConfigService.saveShortcuts(this.shortcuts);
  }

  // Paths
  async addPath(path: SavedPath) {
    this.paths.push(path);
    await ConfigService.savePaths(this.paths);
  }

  async updatePath(path: SavedPath) {
    const idx = this.paths.findIndex((p) => p.id === path.id);
    if (idx !== -1) {
      this.paths[idx] = path;
      await ConfigService.savePaths(this.paths);
    }
  }

  async removePath(id: string) {
    this.paths = this.paths.filter((p) => p.id !== id);
    await ConfigService.savePaths(this.paths);
  }

  // Tema
  applyTheme(theme: AppTheme) {
    this.theme = { ...theme };
    applyThemeToDom(this.theme);
  }

  async saveTheme() {
    await ConfigService.saveTheme(this.theme);
  }

  async resetTheme() {
    this.applyTheme({ ...defaultTheme });
    await ConfigService.saveTheme(this.theme);
  }

  // Temas Customizados
  async addCustomTheme(theme: AppTheme) {
    // Evita nome duplicado
    const exists = this.customThemes.findIndex((t) => t.name === theme.name);
    if (exists !== -1) {
      this.customThemes[exists] = { ...theme };
    } else {
      this.customThemes.push({ ...theme });
    }
    await ConfigService.saveCustomThemes(this.customThemes);
  }

  async renameCustomTheme(oldName: string, newName: string) {
    const idx = this.customThemes.findIndex((t) => t.name === oldName);
    if (idx !== -1) {
      this.customThemes[idx] = { ...this.customThemes[idx], name: newName };
      // Se o tema ativo for esse, atualiza também
      if (this.theme.name === oldName) {
        this.theme = { ...this.theme, name: newName };
        await ConfigService.saveTheme(this.theme);
      }
      await ConfigService.saveCustomThemes(this.customThemes);
    }
  }

  async deleteCustomTheme(name: string) {
    this.customThemes = this.customThemes.filter((t) => t.name !== name);
    await ConfigService.saveCustomThemes(this.customThemes);
  }
}

export const configStore = new ConfigStore();

