/**
 * Utilitários para manipulação de cores com canal de opacidade (Alpha)
 */

export interface ColorParts {
  hex: string;       // #rrggbb
  opacity: number;   // 0 a 100
}

/**
 * Converte qualquer formato de cor (hex, hex8, rgba, rgb) em { hex: '#rrggbb', opacity: 0-100 }
 */
export function parseColorWithOpacity(colorStr?: string): ColorParts {
  if (!colorStr) {
    return { hex: '#000000', opacity: 100 };
  }

  const str = colorStr.trim();

  // Caso hex 8 dígitos: #rrggbbaa ou 5 dígitos: #rgba
  if (str.startsWith('#') && (str.length === 9 || str.length === 5)) {
    if (str.length === 9) {
      const hex = str.slice(0, 7);
      const alphaHex = str.slice(7, 9);
      const opacity = Math.round((parseInt(alphaHex, 16) / 255) * 100);
      return { hex, opacity: isNaN(opacity) ? 100 : opacity };
    } else if (str.length === 5) {
      const r = str[1];
      const g = str[2];
      const b = str[3];
      const a = str[4];
      const hex = `#${r}${r}${g}${g}${b}${b}`;
      const opacity = Math.round((parseInt(`${a}${a}`, 16) / 255) * 100);
      return { hex, opacity: isNaN(opacity) ? 100 : opacity };
    }
  }

  // Caso hex 6 ou 3 dígitos: #rrggbb ou #rgb
  if (str.startsWith('#')) {
    if (str.length === 4) {
      const r = str[1];
      const g = str[2];
      const b = str[3];
      return { hex: `#${r}${r}${g}${g}${b}${b}`, opacity: 100 };
    }
    return { hex: str.slice(0, 7), opacity: 100 };
  }

  // Caso rgba(r, g, b, a) ou rgb(r, g, b)
  const rgbMatch = str.match(/rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)(?:\s*,\s*([\d.]+))?\s*\)/i);
  if (rgbMatch) {
    const r = Math.min(255, Math.max(0, parseInt(rgbMatch[1], 10)));
    const g = Math.min(255, Math.max(0, parseInt(rgbMatch[2], 10)));
    const b = Math.min(255, Math.max(0, parseInt(rgbMatch[3], 10)));
    const hex = `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
    let opacity = 100;
    if (rgbMatch[4] !== undefined) {
      const alphaVal = parseFloat(rgbMatch[4]);
      opacity = Math.round(alphaVal <= 1 ? alphaVal * 100 : alphaVal);
    }
    return { hex, opacity: isNaN(opacity) ? 100 : opacity };
  }

  return { hex: '#000000', opacity: 100 };
}

/**
 * Combina um hex (#rrggbb) e uma opacidade (0-100) em uma string hex8 (#rrggbbaa) ou hex (#rrggbb se 100%)
 */
export function formatColorWithOpacity(hex: string, opacity: number): string {
  const cleanHex = hex.startsWith('#') ? hex.slice(0, 7) : `#${hex.slice(0, 6)}`;
  const clampedOpacity = Math.max(0, Math.min(100, Math.round(opacity)));

  if (clampedOpacity === 100) {
    return cleanHex;
  }

  const alpha = Math.round((clampedOpacity / 100) * 255);
  const alphaHex = alpha.toString(16).padStart(2, '0');
  return `${cleanHex}${alphaHex}`;
}
