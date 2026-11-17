import {
  type SshHost,
  type CustomCommand,
  type CustomAlias,
  type SavedPath,
  type AppTheme,
  type ShellsConfig,
  type ShellProfile,
  type ShellPreference,
  type VpsPipelineConfig,
  defaultCustomCommands,
  defaultCustomAliases,
  defaultVpsPipeline,
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
  vpsPipeline = $state<VpsPipelineConfig>({ ...defaultVpsPipeline });
  initialized = $state(false);

  #initPromise: Promise<void> | null = null;

  static #loaders = {
    hosts: () => ConfigService.loadSshHosts(),
    commands: () => ConfigService.loadCustomCommands(),
    aliases: () => ConfigService.loadAliases(),
    shortcuts: () => ConfigService.loadShortcuts(),
    paths: () => ConfigService.loadPaths(),
    theme: () => ConfigService.loadTheme(),
    customThemes: () => ConfigService.loadCustomThemes(),
    shells: () => ConfigService.loadShells(),
    vpsPipeline: () => ConfigService.loadVpsPipeline(),
  } as const;

  /**
   * Relê do disco antes de alterar. Evita que um estado antigo em memória
   * (carregado antes da pasta sincronizada atualizar) sobrescreva arquivos
   * mais novos vindos do outro sistema.
   */
  async #refresh(...keys: (keyof typeof ConfigStore.#loaders)[]) {
    for (const k of keys) {
      try {
        (this as any)[k] = await ConfigStore.#loaders[k]();
      } catch (e) {
        console.error(`[config] falha ao recarregar ${k}:`, e);
      }
    }
  }

  #focusHooked = false;
  #hookFocusReload() {
    if (this.#focusHooked || typeof window === "undefined") return;
    this.#focusHooked = true;
    window.addEventListener("focus", () => {
      if (!this.initialized) return;
      this.#refresh("hosts", "commands", "aliases", "shortcuts", "paths",
        "customThemes", "shells", "vpsPipeline");
    });
  }

  async init() {
    if (this.initialized) return;
    if (this.#initPromise) return this.#initPromise;

    this.#initPromise = (async () => {
      const [h, c, a, s, p, t, ct, sh, vp] = await Promise.all([
        ConfigService.loadSshHosts(),
        ConfigService.loadCustomCommands(),
        ConfigService.loadAliases(),
        ConfigService.loadShortcuts(),
        ConfigService.loadPaths(),
        ConfigService.loadTheme(),
        ConfigService.loadCustomThemes(),
        ConfigService.loadShells(),
        ConfigService.loadVpsPipeline(),
      ]);
      this.hosts = h;
      this.commands = c;
      this.aliases = a;
      this.shortcuts = s;
      this.paths = p;
      this.theme = t;
      this.customThemes = ct;
      this.shells = sh;
      this.vpsPipeline = vp;
      applyThemeToDom(t);
      this.initialized = true;
      this.#hookFocusReload();
    })().catch((e) => {
      // Libera a promise para permitir nova tentativa e não quebra os chamadores
      console.error("[config] falha ao carregar configurações:", e);
      this.#initPromise = null;
    });

    return this.#initPromise;
  }

  // SSH Hosts
  async addHost(host: SshHost) {
    await this.#refresh("hosts");
    this.hosts.push(host);
    await ConfigService.saveSshHosts(this.hosts);
  }

  async updateHost(host: SshHost) {
    await this.#refresh("hosts");
    const idx = this.hosts.findIndex((h) => h.id === host.id);
    if (idx !== -1) {
      this.hosts[idx] = host;
      await ConfigService.saveSshHosts(this.hosts);
    }
  }

  async removeHost(id: string) {
    await this.#refresh("hosts");
    this.hosts = this.hosts.filter((h) => h.id !== id);
    await ConfigService.saveSshHosts(this.hosts);
  }

  // Custom Commands
  async addCommand(cmd: CustomCommand) {
    await this.#refresh("commands");
    this.commands.push(cmd);
    await ConfigService.saveCustomCommands(this.commands);
  }

  async updateCommand(cmd: CustomCommand) {
    await this.#refresh("commands");
    const idx = this.commands.findIndex((c) => c.id === cmd.id);
    if (idx !== -1) {
      this.commands[idx] = cmd;
      await ConfigService.saveCustomCommands(this.commands);
    }
  }

  async removeCommand(id: string) {
    await this.#refresh("commands");
    this.commands = this.commands.filter((c) => c.id !== id);
    await ConfigService.saveCustomCommands(this.commands);
  }

  async resetCommands() {
    await this.#refresh("commands");
    this.commands = [...defaultCustomCommands];
    await ConfigService.saveCustomCommands(this.commands);
  }

  // Métodos do Pipeline VPS Desacoplado
  async addPipelineTrigger(cmd: string) {
    await this.#refresh("vpsPipeline");
    const val = cmd.trim().toLowerCase();
    if (val && !this.vpsPipeline.triggers.includes(val)) {
      this.vpsPipeline.triggers.push(val);
      await ConfigService.saveVpsPipeline(this.vpsPipeline);
    }
  }

  async removePipelineTrigger(cmd: string) {
    await this.#refresh("vpsPipeline");
    this.vpsPipeline.triggers = this.vpsPipeline.triggers.filter((t) => t !== cmd);
    await ConfigService.saveVpsPipeline(this.vpsPipeline);
  }

  async addPipelinePrefixArg(arg: string) {
    await this.#refresh("vpsPipeline");
    const val = arg.trim();
    if (val && !this.vpsPipeline.prefixArgs.includes(val)) {
      this.vpsPipeline.prefixArgs.push(val);
      await ConfigService.saveVpsPipeline(this.vpsPipeline);
    }
  }

  async removePipelinePrefixArg(arg: string) {
    await this.#refresh("vpsPipeline");
    this.vpsPipeline.prefixArgs = this.vpsPipeline.prefixArgs.filter((a) => a !== arg);
    await ConfigService.saveVpsPipeline(this.vpsPipeline);
  }

  async addPipelineSuffixArg(arg: string) {
    await this.#refresh("vpsPipeline");
    const val = arg.trim();
    if (val && !this.vpsPipeline.suffixArgs.includes(val)) {
      this.vpsPipeline.suffixArgs.push(val);
      await ConfigService.saveVpsPipeline(this.vpsPipeline);
    }
  }

  async removePipelineSuffixArg(arg: string) {
    await this.#refresh("vpsPipeline");
    this.vpsPipeline.suffixArgs = this.vpsPipeline.suffixArgs.filter((a) => a !== arg);
    await ConfigService.saveVpsPipeline(this.vpsPipeline);
  }

  async resetVpsPipeline() {
    await this.#refresh("vpsPipeline");
    this.vpsPipeline = { ...defaultVpsPipeline };
    await ConfigService.saveVpsPipeline(this.vpsPipeline);
  }

  // Custom Aliases
  async addAlias(alias: CustomAlias) {
    await this.#refresh("aliases");
    this.aliases.push(alias);
    await ConfigService.saveAliases(this.aliases);
  }

  async updateAlias(alias: CustomAlias) {
    await this.#refresh("aliases");
    const idx = this.aliases.findIndex((a) => a.id === alias.id);
    if (idx !== -1) {
      this.aliases[idx] = alias;
      await ConfigService.saveAliases(this.aliases);
    }
  }

  async removeAlias(id: string) {
    await this.#refresh("aliases");
    this.aliases = this.aliases.filter((a) => a.id !== id);
    await ConfigService.saveAliases(this.aliases);
  }

  async resetAliases() {
    await this.#refresh("aliases");
    this.aliases = [...defaultCustomAliases];
    await ConfigService.saveAliases(this.aliases);
  }

  // Shortcuts
  async updateShortcut(actionId: string, keys: string) {
    await this.#refresh("shortcuts");
    this.shortcuts[actionId] = keys;
    await ConfigService.saveShortcuts(this.shortcuts);
  }

  async resetShortcuts() {
    await this.#refresh("shortcuts");
    this.shortcuts = { ...defaultShortcuts };
    await ConfigService.saveShortcuts(this.shortcuts);
  }

  // Paths
  async addPath(path: SavedPath) {
    await this.#refresh("paths");
    this.paths.push(path);
    await ConfigService.savePaths(this.paths);
  }

  async updatePath(path: SavedPath) {
    await this.#refresh("paths");
    const idx = this.paths.findIndex((p) => p.id === path.id);
    if (idx !== -1) {
      this.paths[idx] = path;
      await ConfigService.savePaths(this.paths);
    }
  }

  async removePath(id: string) {
    await this.#refresh("paths");
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
    await this.#refresh("theme");
    this.applyTheme({ ...defaultTheme });
    await ConfigService.saveTheme(this.theme);
  }

  // Temas Customizados
  async addCustomTheme(theme: AppTheme) {
    await this.#refresh("customThemes");
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
    await this.#refresh("customThemes", "theme");
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
    await this.#refresh("customThemes");
    this.customThemes = this.customThemes.filter((t) => t.name !== name);
    await ConfigService.saveCustomThemes(this.customThemes);
  }

  // Shell do Terminal
  /** Define o shell padrão das novas abas locais (null = padrão do sistema). */
  async setDefaultShell(shell: ShellPreference | null) {
    await this.#refresh("shells");
    this.shells = { ...this.shells, default: shell ? { ...shell } : null };
    await ConfigService.saveShells(this.shells);
  }

  /** Ajusta apenas os argumentos do shell padrão atual. */
  async setDefaultShellArgs(args: string[]) {
    await this.#refresh("shells");
    if (!this.shells.default) return;
    this.shells = {
      ...this.shells,
      default: { ...this.shells.default, args: [...args] },
    };
    await ConfigService.saveShells(this.shells);
  }

  async addCustomShell(shell: ShellProfile) {
    await this.#refresh("shells");
    this.shells = {
      ...this.shells,
      custom: [...this.shells.custom, shell],
    };
    await ConfigService.saveShells(this.shells);
  }

  async removeCustomShell(id: string) {
    await this.#refresh("shells");
    this.shells = {
      ...this.shells,
      custom: this.shells.custom.filter((s) => s.id !== id),
    };
    await ConfigService.saveShells(this.shells);
  }

  // Recarregar todas as configurações (útil após migração de pasta)
  async reloadAll() {
    this.initialized = false;
    this.#initPromise = null;
    await this.init();
  }
}

export const configStore = new ConfigStore();
