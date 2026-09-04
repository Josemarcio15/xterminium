/**
 * Utilitários para parsing e matching universal de atalhos de teclado
 */

// Normaliza e converte um KeyboardEvent em uma string canônica (ex: "Ctrl+C", "Ctrl+Shift+V")
export function parseKeyboardEvent(e: KeyboardEvent): string {
  const parts: string[] = [];
  if (e.ctrlKey) parts.push('Ctrl');
  if (e.altKey) parts.push('Alt');
  if (e.shiftKey) parts.push('Shift');
  if (e.metaKey) parts.push('Meta');

  let key = e.key;

  // Normalizações de teclas
  if (key === ' ') key = 'Space';
  else if (key.length === 1) key = key.toUpperCase();

  if (!['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) {
    parts.push(key);
  }

  return parts.join('+');
}

// Normaliza uma string de atalho para comparação insensível a maiúsculas e ordem
export function normalizeShortcut(combo: string): string {
  if (!combo) return '';
  const parts = combo.split('+').map((p) => p.trim().toLowerCase());
  // Ordena para garantir que 'shift+ctrl+c' case com 'ctrl+shift+c'
  const modifiers: string[] = [];
  let mainKey = '';

  for (const part of parts) {
    if (['ctrl', 'control'].includes(part)) modifiers.push('ctrl');
    else if (['alt'].includes(part)) modifiers.push('alt');
    else if (['shift'].includes(part)) modifiers.push('shift');
    else if (['meta', 'cmd', 'command', 'win'].includes(part)) modifiers.push('meta');
    else mainKey = part;
  }

  modifiers.sort();
  if (mainKey) modifiers.push(mainKey);
  return modifiers.join('+');
}

// Verifica se um evento do teclado corresponde a um atalho configurado
export function matchesShortcut(e: KeyboardEvent, combo: string): boolean {
  if (!combo) return false;
  const eventCombo = normalizeShortcut(parseKeyboardEvent(e));
  return eventCombo === normalizeShortcut(combo);
}

// Tipo para as funções de comando
export type CommandHandler = () => void | Promise<void>;
