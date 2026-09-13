/**
 * Utilidades de caminho válidas tanto para `/` (POSIX) quanto para `\` (Windows).
 *
 * O backend devolve o cwd no formato nativo de cada sistema (`/home/user` no
 * Linux, `C:\Users\user` no Windows), então nunca assuma um separador fixo.
 */

/** Separa um caminho em segmentos, ignorando separadores repetidos e vazios. */
function segments(path: string): string[] {
  return path.split(/[\\/]+/).filter(Boolean);
}

/** Raiz de um drive do Windows (`C:` ou `C:\`). */
function isDriveRoot(value: string): boolean {
  return /^[a-zA-Z]:\\?$/.test(value);
}

/** Remove separadores finais sem transformar a raiz de um drive em `C:`. */
export function stripTrailingSeparators(path: string): string {
  const trimmed = path.replace(/[\\/]+$/, "");
  if (/^[a-zA-Z]:$/.test(trimmed)) return `${trimmed}\\`;
  return trimmed || "/";
}

/**
 * Último segmento do caminho (nome da pasta), com fallback para a raiz.
 *
 * - `/var/www` -> `www`
 * - `C:\Users\Marcio\git\xterminium` -> `xterminium`
 * - `/` -> `/` e `C:\` -> `C:\`
 */
export function pathBaseName(path: string): string {
  const clean = stripTrailingSeparators(path);
  const parts = segments(clean);
  const last = parts[parts.length - 1];
  if (!last) return clean;
  if (isDriveRoot(clean)) return `${last}\\`;
  return last;
}
