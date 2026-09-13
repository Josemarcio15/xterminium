import {
  type SshHost,
  type CustomCommand,
  type CustomAlias,
  type SavedPath,
  type AppTheme,
  type ShellsConfig,
  type ShellProfile,
  type ShellPreference,
  defaultCustomCommands,
  defaultCustomAliases,
  defaultShortcuts,
  defaultTheme,
  applyThemeToDom,
} from "../types";
import { ConfigService } from "../services/config.service";

export class ConfigStore {
  hosts = $state<SshHost[]>([]);
  commands = $state<CustomCommand[]>([]);
  aliases = $state<CustomAlias[]>([]);
  shortcuts = $state<Record<string, string>>({ ...defaultShortcuts });
  paths = $state<SavedPath[]>([]);
  theme = $state<AppTheme>({ ...defaultTheme });
  customThemes = $state<AppTheme[]>([]);
  shells = $state<ShellsConfig>({ default: null, custom: [] });
  initialized = $state(false);

  #initPromise: Promise<void> | null = null;

  async init() {
    if (this.initialized) return;
    if (this.#initPromise) return this.#initPromise;

    this.#initPromise = (async () => {
      const [h, c, a, s, p, t, ct, sh] = await Promise.all([
        ConfigService.loadSshHosts(),
        ConfigService.loadCustomCommands(),
        ConfigService.loadAliases(),
        ConfigService.loadShortcuts(),
        ConfigService.loadPaths(),
        ConfigService.loadTheme(),
        ConfigService.loadCustomThemes(),
        ConfigService.loadShells(),
      ]);
      this.hosts = h;
      this.commands = c;
      this.aliases = a;
      this.shortcuts = s;
      this.paths = p;
      this.theme = t;
      this.customThemes = ct;
      this.shells = sh;
      applyThemeToDom(t);
      this.initialized = true;
    })().catch((e) => {
      // Libera a promise para permitir nova tentativa e não quebra os chamadores
      console.error("[config] falha ao carregar configurações:", e);
      this.#initPromise = null;
    });

    return this.#initPromise;
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

  // Custom Aliases
  async addAlias(alias: CustomAlias) {
    this.aliases.push(alias);
    await ConfigService.saveAliases(this.aliases);
  }

  async updateAlias(alias: CustomAlias) {
    const idx = this.aliases.findIndex((a) => a.id === alias.id);
    if (idx !== -1) {
      this.aliases[idx] = alias;
      await ConfigService.saveAliases(this.aliases);
    }
  }

  async removeAlias(id: string) {
    this.aliases = this.aliases.filter((a) => a.id !== id);
    await ConfigService.saveAliases(this.aliases);
  }

  async resetAliases() {
    this.aliases = [...defaultCustomAliases];
    await ConfigService.saveAliases(this.aliases);
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

  // Shell do Terminal
  /** Define o shell padrão das novas abas locais (null = padrão do sistema). */
  async setDefaultShell(shell: ShellPreference | null) {
    this.shells = { ...this.shells, default: shell ? { ...shell } : null };
    await ConfigService.saveShells(this.shells);
  }

  /** Ajusta apenas os argumentos do shell padrão atual. */
  async setDefaultShellArgs(args: string[]) {
    if (!this.shells.default) return;
    this.shells = {
      ...this.shells,
      default: { ...this.shells.default, args: [...args] },
    };
    await ConfigService.saveShells(this.shells);
  }

  async addCustomShell(shell: ShellProfile) {
    this.shells = {
      ...this.shells,
      custom: [...this.shells.custom, shell],
    };
    await ConfigService.saveShells(this.shells);
  }

  async removeCustomShell(id: string) {
    this.shells = {
      ...this.shells,
      custom: this.shells.custom.filter((s) => s.id !== id),
    };
    await ConfigService.saveShells(this.shells);
  }
}

export const configStore = new ConfigStore();
