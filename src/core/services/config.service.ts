import { invoke } from "@tauri-apps/api/core";
import {
  type SshHost,
  type SavedPath,
  type CustomCommand,
  type CustomAlias,
  type AppTheme,
  type ShellsConfig,
  type VpsPipelineConfig,
  defaultShortcuts,
  defaultCustomCommands,
  defaultCustomAliases,
  defaultVpsPipeline,
  defaultTheme,
} from "../types";

export class ConfigService {
  // Hosts SSH
  static async loadSshHosts(): Promise<SshHost[]> {
    try {
      const content = await invoke<string>("load_config", { filename: "ssh" });
      if (content && content.trim()) {
        return JSON.parse(content);
      }
    } catch (e) {
      console.error("Erro ao ler ~/.config/xterminium/ssh.json", e);
    }
    return [];
  }

  static async saveSshHosts(hosts: SshHost[]): Promise<void> {
    try {
      const json = JSON.stringify(hosts, null, 2);
      await invoke("save_config", { filename: "ssh", content: json });
    } catch (e) {
      console.error("Erro ao salvar ~/.config/xterminium/ssh.json", e);
    }
  }

  // Atalhos de Teclado
  private static cachedShortcuts: Record<string, string> | null = null;

  static async loadShortcuts(): Promise<Record<string, string>> {
    if (this.cachedShortcuts) {
      return this.cachedShortcuts;
    }
    try {
      const content = await invoke<string>("load_config", {
        filename: "shortcuts",
      });
      if (content && content.trim()) {
        const parsed = { ...defaultShortcuts, ...JSON.parse(content) };
        this.cachedShortcuts = parsed;
        return parsed;
      }
    } catch (e) {
      console.error("Erro ao ler ~/.config/xterminium/shortcuts.json", e);
    }

    const defaults = { ...defaultShortcuts };
    this.cachedShortcuts = defaults;
    return defaults;
  }

  static async saveShortcuts(shortcuts: Record<string, string>): Promise<void> {
    this.cachedShortcuts = { ...shortcuts };
    try {
      const json = JSON.stringify(shortcuts, null, 2);
      await invoke("save_config", { filename: "shortcuts", content: json });
    } catch (e) {
      console.error("Erro ao salvar ~/.config/xterminium/shortcuts.json", e);
    }
  }

  // Caminhos Salvos (Paths)
  static async loadPaths(): Promise<SavedPath[]> {
    try {
      const content = await invoke<string>("load_config", {
        filename: "paths",
      });
      if (content && content.trim()) {
        return JSON.parse(content);
      }
    } catch (e) {
      console.error("Erro ao ler ~/.config/xterminium/paths.json", e);
    }
    return [];
  }

  static async savePaths(paths: SavedPath[]): Promise<void> {
    try {
      const json = JSON.stringify(paths, null, 2);
      await invoke("save_config", { filename: "paths", content: json });
    } catch (e) {
      console.error("Erro ao salvar ~/.config/xterminium/paths.json", e);
    }
  }

  // Comandos Customizados
  private static cachedCommands: CustomCommand[] | null = null;

  static async loadCustomCommands(): Promise<CustomCommand[]> {
    if (this.cachedCommands) {
      return this.cachedCommands;
    }
    try {
      const content = await invoke<string>("load_config", {
        filename: "commands",
      });
      if (content && content.trim()) {
        const parsed = JSON.parse(content);
        if (Array.isArray(parsed) && parsed.length > 0) {
          this.cachedCommands = parsed;
          return parsed;
        }
      }
    } catch (e) {
      console.error("Erro ao ler ~/.config/xterminium/commands.json", e);
    }

    const defaults = [...defaultCustomCommands];
    this.cachedCommands = defaults;
    return defaults;
  }

  static async saveCustomCommands(commands: CustomCommand[]): Promise<void> {
    this.cachedCommands = [...commands];
    try {
      const json = JSON.stringify(commands, null, 2);
      await invoke("save_config", { filename: "commands", content: json });
    } catch (e) {
      console.error("Erro ao salvar ~/.config/xterminium/commands.json", e);
    }
  }

  // Pipeline VPS Desacoplado (Gatilhos, Args 1 e Args 2 independentes)
  static async loadVpsPipeline(): Promise<VpsPipelineConfig> {
    try {
      const content = await invoke<string>("load_config", {
        filename: "vps_pipeline",
      });
      if (content && content.trim()) {
        const parsed = JSON.parse(content);
        return {
          triggers: Array.isArray(parsed?.triggers) ? parsed.triggers : defaultVpsPipeline.triggers,
          prefixArgs: Array.isArray(parsed?.prefixArgs) ? parsed.prefixArgs : defaultVpsPipeline.prefixArgs,
          suffixArgs: Array.isArray(parsed?.suffixArgs) ? parsed.suffixArgs : defaultVpsPipeline.suffixArgs,
        };
      }
    } catch (e) {
      console.error("Erro ao ler ~/.config/xterminium/vps_pipeline.json", e);
    }
    return { ...defaultVpsPipeline };
  }

  static async saveVpsPipeline(pipeline: VpsPipelineConfig): Promise<void> {
    try {
      const json = JSON.stringify(pipeline, null, 2);
      await invoke("save_config", { filename: "vps_pipeline", content: json });
    } catch (e) {
      console.error("Erro ao salvar ~/.config/xterminium/vps_pipeline.json", e);
    }
  }

  // Tema
  static async loadTheme(): Promise<AppTheme> {
    try {
      const content = await invoke<string>("load_config", {
        filename: "theme",
      });
      if (content && content.trim()) {
        return { ...defaultTheme, ...JSON.parse(content) };
      }
    } catch (e) {
      console.error("Erro ao ler ~/.config/xterminium/theme.json", e);
    }
    return { ...defaultTheme };
  }

  static async saveTheme(theme: AppTheme): Promise<void> {
    try {
      const json = JSON.stringify(theme, null, 2);
      await invoke("save_config", { filename: "theme", content: json });
    } catch (e) {
      console.error("Erro ao salvar ~/.config/xterminium/theme.json", e);
    }
  }

  // Temas Customizados
  static async loadCustomThemes(): Promise<AppTheme[]> {
    try {
      const content = await invoke<string>("load_config", {
        filename: "custom_themes",
      });
      if (content && content.trim()) {
        return JSON.parse(content);
      }
    } catch (e) {
      console.error("Erro ao ler ~/.config/xterminium/custom_themes.json", e);
    }
    return [];
  }

  static async saveCustomThemes(themes: AppTheme[]): Promise<void> {
    try {
      const json = JSON.stringify(themes, null, 2);
      await invoke("save_config", { filename: "custom_themes", content: json });
    } catch (e) {
      console.error(
        "Erro ao salvar ~/.config/xterminium/custom_themes.json",
        e,
      );
    }
  }

  // Aliases de Comandos
  static async loadAliases(): Promise<CustomAlias[]> {
    try {
      const content = await invoke<string>("load_config", {
        filename: "aliases",
      });
      if (content && content.trim()) {
        return JSON.parse(content);
      }
    } catch (e) {
      console.error("Erro ao ler ~/.config/xterminium/aliases.json", e);
    }
    return [...defaultCustomAliases];
  }

  static async saveAliases(aliases: CustomAlias[]): Promise<void> {
    try {
      const json = JSON.stringify(aliases, null, 2);
      await invoke("save_config", { filename: "aliases", content: json });
    } catch (e) {
      console.error("Erro ao salvar ~/.config/xterminium/aliases.json", e);
    }
  }

  // Shell do Terminal
  static async loadShells(): Promise<ShellsConfig> {
    try {
      const content = await invoke<string>("load_config", {
        filename: "shells",
      });
      if (content && content.trim()) {
        const parsed = JSON.parse(content);
        return {
          default: parsed?.default ?? null,
          custom: Array.isArray(parsed?.custom) ? parsed.custom : [],
        };
      }
    } catch (e) {
      console.error("Erro ao ler ~/.config/xterminium/shells.json", e);
    }
    return { default: null, custom: [] };
  }

  static async saveShells(config: ShellsConfig): Promise<void> {
    try {
      const json = JSON.stringify(config, null, 2);
      await invoke("save_config", { filename: "shells", content: json });
    } catch (e) {
      console.error("Erro ao salvar ~/.config/xterminium/shells.json", e);
    }
  }

  // Localização da pasta de configurações
  static async getConfigLocation(): Promise<{ current_path: string; default_path: string; is_custom: boolean }> {
    return await invoke<{ current_path: string; default_path: string; is_custom: boolean }>("get_config_location");
  }

  static async checkMigrationTarget(target_path: string): Promise<{
    target_exists: boolean;
    target_has_files: boolean;
    conflicting_files: string[];
  }> {
    return await invoke<{
      target_exists: boolean;
      target_has_files: boolean;
      conflicting_files: string[];
    }>("check_migration_target", { targetPath: target_path });
  }

  static async changeConfigLocation(target_path: string, overwrite: boolean): Promise<string> {
    return await invoke<string>("change_config_location", { targetPath: target_path, overwrite });
  }

  static async pickConfigFolder(): Promise<string | null> {
    return await invoke<string | null>("pick_config_folder");
  }
}
