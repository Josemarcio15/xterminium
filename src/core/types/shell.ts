/**
 * Configuração de shell do terminal local.
 *
 * Os shells instalados são descobertos em tempo de execução pelo backend
 * (comando `list_shells`). O usuário escolhe qual usar como padrão e pode
 * adicionar shells customizados informando caminho + argumentos.
 */

/** Shell descoberto automaticamente no sistema. */
export interface ShellProfile {
  id: string;
  name: string;
  path: string;
  args: string[];
  /** "powershell" | "pwsh" | "cmd" | "wsl" | "git-bash" | "bash" | "zsh" | "fish" | "nu" | "sh" | "other" */
  kind: string;
  /** true quando é o shell padrão do sistema. */
  recommended?: boolean;
  /** true para shells adicionados manualmente pelo usuário. */
  custom?: boolean;
}

/** Shell selecionado como padrão para novas abas locais. */
export interface ShellPreference {
  name: string;
  path: string;
  args: string[];
  kind?: string;
}

export interface ShellsConfig {
  /** null = usar o shell padrão do sistema. */
  default: ShellPreference | null;
  custom: ShellProfile[];
}

export const defaultShellsConfig: ShellsConfig = {
  default: null,
  custom: [],
};

/** Compara caminhos de shell ignorando caixa e barras finais. */
export function sameShellPath(a?: string | null, b?: string | null): boolean {
  if (!a || !b) return false;
  const normalize = (p: string) =>
    p
      .trim()
      .replace(/[\\/]+$/, "")
      .toLowerCase();
  return normalize(a) === normalize(b);
}

/** Identificador estável a partir do caminho (mesma regra do backend). */
export function shellIdFromPath(path: string): string {
  return path
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}
