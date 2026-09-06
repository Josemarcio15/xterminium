export interface AppTheme {
  name: string;

  // Backgrounds
  bgBase: string;
  bgTitlebar: string;
  bgPanel: string;
  bgItem: string;
  bgItemHover?: string;
  bgItemInput: string;
  bgTabActive: string;
  bgTabHover?: string;

  // Accents
  accentPrimary: string;
  accentSecondary: string;
  accentSftp: string;
  accentWarn: string;

  // Text
  textBase: string;
  textMuted: string;
  textFaint: string;
  textSpecial?: string; // Textos especiais / templates (ex: {user}@{ip})

  // Borders
  borderSubtle?: string;
  borderPanel?: string;

  // Terminal
  terminalBg: string;
  terminalFg: string;
  terminalCursorLocal: string;
  terminalCursorSsh: string;
  terminalSelection: string;

  // Buttons: Primário / Ação Principal
  btnPrimaryBg?: string;
  btnPrimaryText?: string;
  btnPrimaryHover?: string;

  // Buttons: Secundário / Neutro
  btnSecondaryBg?: string;
  btnSecondaryText?: string;
  btnSecondaryHover?: string;

  // Buttons: Perigo / Remoção
  btnDangerBg?: string;
  btnDangerText?: string;
  btnDangerHover?: string;

  // Buttons: Sucesso / Concluído
  btnSuccessBg?: string;
  btnSuccessText?: string;

  // Buttons: Vidro / Translúcido (Glass)
  btnGlassBg?: string;
  btnGlassText?: string;
  btnGlassHover?: string;

  // Buttons: Borda global
  btnBorder?: string;

  // Formato & Arredondamento (Border Radius)
  btnRadius?: string;     // ex: '6px' ou '0px'
  windowRadius?: string;  // ex: '10px' ou '0px'
  elevation?: number;     // ex: 0 (flat) a 4 (alto relevo / sombras profundas)

  // Permite acesso indexado dinâmico por chave
  [key: string]: string | number | undefined;
}

export const defaultTheme: AppTheme = {
  name: 'Default Dark',
  bgBase: '#0f111a',
  bgTitlebar: '#13151f',
  bgPanel: '#171926',
  bgItem: '#1d2030',
  bgItemHover: '#25293d',
  bgItemInput: '#10121c',
  bgTabActive: '#1b1e2c',
  bgTabHover: '#222638',
  accentPrimary: '#38bdf8',
  accentSecondary: '#00e699',
  accentSftp: '#818cf8',
  accentWarn: '#fbbf24',
  textBase: '#e6e6e6',
  textMuted: '#94a3b8',
  textFaint: '#64748b',
  borderSubtle: 'rgba(255, 255, 255, 0.08)',
  borderPanel: 'rgba(255, 255, 255, 0.12)',
  terminalBg: '#0f111a',
  terminalFg: '#e6e6e6',
  terminalCursorLocal: '#00e699',
  terminalCursorSsh: '#38bdf8',
  terminalSelection: 'rgba(255, 255, 255, 0.2)',

  btnPrimaryBg: '#0284c7',
  btnPrimaryText: '#ffffff',
  btnPrimaryHover: '#0369a1',
  btnSecondaryBg: '#1d2030',
  btnSecondaryText: '#94a3b8',
  btnSecondaryHover: '#25293d',
  btnDangerBg: '#dc2626',
  btnDangerText: '#ffffff',
  btnDangerHover: '#b91c1c',
  btnSuccessBg: '#059669',
  btnSuccessText: '#ffffff',
};

export const presetThemes: AppTheme[] = [
  defaultTheme,
  {
    name: 'Dracula',
    bgBase: '#282a36',
    bgTitlebar: '#21222c',
    bgPanel: '#2d2f3f',
    bgItem: '#383a4a',
    bgItemInput: '#1e1f29',
    bgTabActive: '#323445',
    accentPrimary: '#bd93f9',
    accentSecondary: '#50fa7b',
    accentSftp: '#ff79c6',
    accentWarn: '#f1fa8c',
    textBase: '#f8f8f2',
    textMuted: '#6272a4',
    textFaint: '#44475a',
    borderSubtle: 'rgba(255, 255, 255, 0.08)',
    borderPanel: 'rgba(255, 255, 255, 0.12)',
    terminalBg: '#282a36',
    terminalFg: '#f8f8f2',
    terminalCursorLocal: '#50fa7b',
    terminalCursorSsh: '#bd93f9',
    terminalSelection: 'rgba(68, 71, 90, 0.6)',
    btnPrimaryBg: '#bd93f9',
    btnPrimaryText: '#282a36',
    btnPrimaryHover: '#a77be8',
    btnSecondaryBg: '#383a4a',
    btnSecondaryText: '#f8f8f2',
    btnSecondaryHover: '#44475a',
    btnDangerBg: '#ff5555',
    btnDangerText: '#282a36',
    btnDangerHover: '#e04545',
    btnSuccessBg: '#50fa7b',
    btnSuccessText: '#282a36',
  },
  {
    name: 'Nord',
    bgBase: '#2e3440',
    bgTitlebar: '#272c37',
    bgPanel: '#3b4252',
    bgItem: '#434c5e',
    bgItemInput: '#252b36',
    bgTabActive: '#3d4656',
    accentPrimary: '#88c0d0',
    accentSecondary: '#a3be8c',
    accentSftp: '#b48ead',
    accentWarn: '#ebcb8b',
    textBase: '#eceff4',
    textMuted: '#81a1c1',
    textFaint: '#94a3b8',
    borderSubtle: 'rgba(255, 255, 255, 0.08)',
    borderPanel: 'rgba(255, 255, 255, 0.12)',
    terminalBg: '#2e3440',
    terminalFg: '#eceff4',
    terminalCursorLocal: '#a3be8c',
    terminalCursorSsh: '#88c0d0',
    terminalSelection: 'rgba(67, 76, 94, 0.7)',
    btnPrimaryBg: '#88c0d0',
    btnPrimaryText: '#2e3440',
    btnPrimaryHover: '#70adc0',
    btnSecondaryBg: '#434c5e',
    btnSecondaryText: '#eceff4',
    btnSecondaryHover: '#4c566a',
    btnDangerBg: '#bf616a',
    btnDangerText: '#eceff4',
    btnDangerHover: '#ab5059',
    btnSuccessBg: '#a3be8c',
    btnSuccessText: '#2e3440',
  },
  {
    name: 'Solarized Dark',
    bgBase: '#002b36',
    bgTitlebar: '#00232d',
    bgPanel: '#073642',
    bgItem: '#0d4455',
    bgItemInput: '#001e26',
    bgTabActive: '#0a3e4e',
    accentPrimary: '#268bd2',
    accentSecondary: '#859900',
    accentSftp: '#d33682',
    accentWarn: '#b58900',
    textBase: '#839496',
    textMuted: '#657b83',
    textFaint: '#586e75',
    borderSubtle: 'rgba(255, 255, 255, 0.08)',
    borderPanel: 'rgba(255, 255, 255, 0.12)',
    terminalBg: '#002b36',
    terminalFg: '#839496',
    terminalCursorLocal: '#859900',
    terminalCursorSsh: '#268bd2',
    terminalSelection: 'rgba(7, 54, 66, 0.8)',
    btnPrimaryBg: '#268bd2',
    btnPrimaryText: '#fdf6e3',
    btnPrimaryHover: '#1e72ad',
    btnSecondaryBg: '#0d4455',
    btnSecondaryText: '#93a1a1',
    btnSecondaryHover: '#155366',
    btnDangerBg: '#dc322f',
    btnDangerText: '#fdf6e3',
    btnDangerHover: '#b82421',
    btnSuccessBg: '#859900',
    btnSuccessText: '#fdf6e3',
  },
  {
    name: 'Midnight',
    bgBase: '#0a0a0f',
    bgTitlebar: '#0d0d14',
    bgPanel: '#111118',
    bgItem: '#18181f',
    bgItemInput: '#08080d',
    bgTabActive: '#16161e',
    accentPrimary: '#7c3aed',
    accentSecondary: '#10b981',
    accentSftp: '#ec4899',
    accentWarn: '#f59e0b',
    textBase: '#e2e8f0',
    textMuted: '#94a3b8',
    textFaint: '#475569',
    borderSubtle: 'rgba(255, 255, 255, 0.08)',
    borderPanel: 'rgba(255, 255, 255, 0.12)',
    terminalBg: '#0a0a0f',
    terminalFg: '#e2e8f0',
    terminalCursorLocal: '#10b981',
    terminalCursorSsh: '#7c3aed',
    terminalSelection: 'rgba(24, 24, 31, 0.8)',
    btnPrimaryBg: '#7c3aed',
    btnPrimaryText: '#ffffff',
    btnPrimaryHover: '#6d28d9',
    btnSecondaryBg: '#18181f',
    btnSecondaryText: '#e2e8f0',
    btnSecondaryHover: '#22222b',
    btnDangerBg: '#ef4444',
    btnDangerText: '#ffffff',
    btnDangerHover: '#dc2626',
    btnSuccessBg: '#10b981',
    btnSuccessText: '#ffffff',
  },
  {
    name: 'Light',
    bgBase: '#f8fafc',
    bgTitlebar: '#eef2f6',
    bgPanel: '#ffffff',
    bgItem: '#f1f5f9',
    bgItemHover: '#e2e8f0',
    bgItemInput: '#ffffff',
    bgTabActive: '#ffffff',
    bgTabHover: '#e2e8f0',
    accentPrimary: '#0284c7',
    accentSecondary: '#059669',
    accentSftp: '#7c3aed',
    accentWarn: '#d97706',
    textBase: '#0f172a',
    textMuted: '#475569',
    textFaint: '#94a3b8',
    borderSubtle: '#e2e8f0',
    borderPanel: '#cbd5e1',
    terminalBg: '#ffffff',
    terminalFg: '#0f172a',
    terminalCursorLocal: '#059669',
    terminalCursorSsh: '#0284c7',
    terminalSelection: 'rgba(2, 132, 199, 0.18)',
    btnPrimaryBg: '#0284c7',
    btnPrimaryText: '#ffffff',
    btnPrimaryHover: '#0369a1',
    btnSecondaryBg: '#f1f5f9',
    btnSecondaryText: '#334155',
    btnSecondaryHover: '#e2e8f0',
    btnDangerBg: '#dc2626',
    btnDangerText: '#ffffff',
    btnDangerHover: '#b91c1c',
    btnSuccessBg: '#059669',
    btnSuccessText: '#ffffff',
    btnBorder: '#cbd5e1',
    btnGlassBg: 'rgba(0, 0, 0, 0.04)',
    btnGlassText: '#0f172a',
    btnGlassHover: 'rgba(0, 0, 0, 0.08)',
  },
  {
    name: 'Light Yellow',
    bgBase: '#fefdfa',
    bgTitlebar: '#fbf6e8',
    bgPanel: '#ffffff',
    bgItem: '#fef9ec',
    bgItemHover: '#fef2d6',
    bgItemInput: '#ffffff',
    bgTabActive: '#ffffff',
    bgTabHover: '#fef2d6',
    accentPrimary: '#d97706', // Âmbar quente
    accentSecondary: '#16a34a',
    accentSftp: '#b45309',
    accentWarn: '#f59e0b',
    textBase: '#292524', // Warm stone escuro
    textMuted: '#78716c',
    textFaint: '#a8a29e',
    textSpecial: '#d97706',
    borderSubtle: '#f3e8cb',
    borderPanel: '#e7d7ad',
    terminalBg: '#fffefc',
    terminalFg: '#292524',
    terminalCursorLocal: '#16a34a',
    terminalCursorSsh: '#d97706',
    terminalSelection: 'rgba(245, 158, 11, 0.22)',
    btnPrimaryBg: '#d97706',
    btnPrimaryText: '#ffffff',
    btnPrimaryHover: '#b45309',
    btnSecondaryBg: '#fef9ec',
    btnSecondaryText: '#44403c',
    btnSecondaryHover: '#fef2d6',
    btnDangerBg: '#dc2626',
    btnDangerText: '#ffffff',
    btnDangerHover: '#b91c1c',
    btnSuccessBg: '#16a34a',
    btnSuccessText: '#ffffff',
    btnGlassBg: 'rgba(217, 119, 6, 0.08)',
    btnGlassText: '#b45309',
    btnGlassHover: 'rgba(217, 119, 6, 0.16)',
    btnBorder: '#e7d7ad',
  },
  {
    name: 'Light Blue',
    bgBase: '#f0f7ff',
    bgTitlebar: '#e2effe',
    bgPanel: '#ffffff',
    bgItem: '#e8f3fe',
    bgItemHover: '#d8eafc',
    bgItemInput: '#ffffff',
    bgTabActive: '#ffffff',
    bgTabHover: '#d8eafc',
    accentPrimary: '#0284c7',
    accentSecondary: '#059669',
    accentSftp: '#6366f1',
    accentWarn: '#ea580c',
    textBase: '#0f172a',
    textMuted: '#475569',
    textFaint: '#94a3b8',
    textSpecial: '#0284c7',
    borderSubtle: '#cfe4fc',
    borderPanel: '#bad9fa',
    terminalBg: '#f8fbff',
    terminalFg: '#0f172a',
    terminalCursorLocal: '#059669',
    terminalCursorSsh: '#0284c7',
    terminalSelection: 'rgba(2, 132, 199, 0.2)',
    btnPrimaryBg: '#0284c7',
    btnPrimaryText: '#ffffff',
    btnPrimaryHover: '#0369a1',
    btnSecondaryBg: '#e8f3fe',
    btnSecondaryText: '#1e293b',
    btnSecondaryHover: '#d8eafc',
    btnDangerBg: '#ef4444',
    btnDangerText: '#ffffff',
    btnDangerHover: '#dc2626',
    btnSuccessBg: '#059669',
    btnSuccessText: '#ffffff',
    btnGlassBg: 'rgba(2, 132, 199, 0.08)',
    btnGlassText: '#0284c7',
    btnGlassHover: 'rgba(2, 132, 199, 0.16)',
    btnBorder: '#bad9fa',
  },
  {
    name: 'Light Purple',
    bgBase: '#faf5ff',
    bgTitlebar: '#f3e8ff',
    bgPanel: '#ffffff',
    bgItem: '#f5eafd',
    bgItemHover: '#ecd7fa',
    bgItemInput: '#ffffff',
    bgTabActive: '#ffffff',
    bgTabHover: '#ecd7fa',
    accentPrimary: '#9333ea', // Roxo / Lilás elegante
    accentSecondary: '#10b981',
    accentSftp: '#ec4899',
    accentWarn: '#f59e0b',
    textBase: '#1e1b4b', // Deep indigo escuro
    textMuted: '#64748b',
    textFaint: '#94a3b8',
    textSpecial: '#9333ea',
    borderSubtle: '#eed8fd',
    borderPanel: '#dfbbfb',
    terminalBg: '#fdfaff',
    terminalFg: '#1e1b4b',
    terminalCursorLocal: '#10b981',
    terminalCursorSsh: '#9333ea',
    terminalSelection: 'rgba(147, 51, 234, 0.2)',
    btnPrimaryBg: '#9333ea',
    btnPrimaryText: '#ffffff',
    btnPrimaryHover: '#7e22ce',
    btnSecondaryBg: '#f5eafd',
    btnSecondaryText: '#3b0764',
    btnSecondaryHover: '#ecd7fa',
    btnDangerBg: '#ef4444',
    btnDangerText: '#ffffff',
    btnDangerHover: '#dc2626',
    btnSuccessBg: '#10b981',
    btnSuccessText: '#ffffff',
    btnGlassBg: 'rgba(147, 51, 234, 0.08)',
    btnGlassText: '#9333ea',
    btnGlassHover: 'rgba(147, 51, 234, 0.16)',
    btnBorder: '#dfbbfb',
  },
];

/** Aplica um tema definindo as CSS custom properties no elemento raiz */
export function applyThemeToDom(theme: AppTheme): void {
  const isLight = isLightColor(theme.bgBase);
  const r = document.documentElement.style;

  r.setProperty('--bg-base', theme.bgBase);
  r.setProperty('--bg-titlebar', theme.bgTitlebar);
  r.setProperty('--bg-panel', theme.bgPanel);
  r.setProperty('--bg-item', theme.bgItem);
  r.setProperty('--bg-item-input', theme.bgItemInput);
  r.setProperty('--bg-tab-active', theme.bgTabActive);

  // Hover dinâmico
  const itemHover = theme.bgItemHover || (isLight ? '#e2e8f0' : 'rgba(255, 255, 255, 0.06)');
  const tabHover = theme.bgTabHover || (isLight ? '#e2e8f0' : 'rgba(255, 255, 255, 0.05)');
  r.setProperty('--bg-item-hover', itemHover);
  r.setProperty('--bg-tab-hover', tabHover);

  r.setProperty('--accent-primary', theme.accentPrimary);
  r.setProperty('--accent-secondary', theme.accentSecondary);
  r.setProperty('--accent-sftp', theme.accentSftp);
  r.setProperty('--accent-warn', theme.accentWarn);
  r.setProperty('--text-base', theme.textBase);
  r.setProperty('--text-muted', theme.textMuted);
  r.setProperty('--text-faint', theme.textFaint);
  r.setProperty('--text-special', theme.textSpecial || theme.accentWarn);

  // Bordas automáticas se não definidas especificamente
  const subtleBorder = theme.borderSubtle || (isLight ? '#e2e8f0' : 'rgba(255, 255, 255, 0.08)');
  const panelBorder = theme.borderPanel || (isLight ? '#cbd5e1' : 'rgba(255, 255, 255, 0.12)');
  r.setProperty('--border-subtle', subtleBorder);
  r.setProperty('--border-panel', panelBorder);

  r.setProperty('--terminal-bg', theme.terminalBg);
  r.setProperty('--terminal-fg', theme.terminalFg);
  r.setProperty('--terminal-cursor-local', theme.terminalCursorLocal);
  r.setProperty('--terminal-cursor-ssh', theme.terminalCursorSsh);
  r.setProperty('--terminal-selection', theme.terminalSelection);

  // Botões por tipo
  r.setProperty('--btn-primary-bg', theme.btnPrimaryBg || theme.accentPrimary);
  r.setProperty('--btn-primary-text', theme.btnPrimaryText || (isLightColor(theme.btnPrimaryBg || theme.accentPrimary) ? '#0f172a' : '#ffffff'));
  r.setProperty('--btn-primary-hover', theme.btnPrimaryHover || (theme.btnPrimaryBg || theme.accentPrimary));

  r.setProperty('--btn-secondary-bg', theme.btnSecondaryBg || theme.bgItem);
  r.setProperty('--btn-secondary-text', theme.btnSecondaryText || theme.textMuted);
  r.setProperty('--btn-secondary-hover', theme.btnSecondaryHover || (theme.bgItemHover || itemHover));

  r.setProperty('--btn-danger-bg', theme.btnDangerBg || '#dc2626');
  r.setProperty('--btn-danger-text', theme.btnDangerText || '#ffffff');
  r.setProperty('--btn-danger-hover', theme.btnDangerHover || '#b91c1c');

  r.setProperty('--btn-success-bg', theme.btnSuccessBg || theme.accentSecondary);
  r.setProperty('--btn-success-text', theme.btnSuccessText || (isLightColor(theme.btnSuccessBg || theme.accentSecondary) ? '#0f172a' : '#ffffff'));

  // Botões Vidro / Translúcido
  r.setProperty('--btn-glass-bg', theme.btnGlassBg || (isLight ? 'rgba(0, 0, 0, 0.04)' : 'rgba(255, 255, 255, 0.06)'));
  r.setProperty('--btn-glass-text', theme.btnGlassText || theme.textBase);
  r.setProperty('--btn-glass-hover', theme.btnGlassHover || (isLight ? 'rgba(0, 0, 0, 0.08)' : 'rgba(255, 255, 255, 0.12)'));

  // Borda global de botões
  const defaultBtnBorder = theme.btnBorder || (isLight ? '#cbd5e1' : 'rgba(255, 255, 255, 0.15)');
  r.setProperty('--btn-border', defaultBtnBorder);

  // Arredondamento (border-radius)
  r.setProperty('--btn-radius', theme.btnRadius || '6px');
  r.setProperty('--window-radius', theme.windowRadius || '10px');

  // Nível de Elevação / Sombras (0: Flat, 1: Sutil, 2: Padrão/Médio, 3: Elevado, 4: Alto Relevo)
  const elev = typeof theme.elevation === 'number' ? theme.elevation : 1;
  const shadowColor = isLight ? 'rgba(15, 23, 42, ' : 'rgba(0, 0, 0, ';

  let shadowBtn = 'none';
  let shadowItem = 'none';
  let shadowPanel = 'none';

  if (elev === 1) {
    shadowBtn = `0 1px 2px ${shadowColor}0.08), 0 1px 1px ${shadowColor}0.04)`;
    shadowItem = `0 1px 2px ${shadowColor}0.05)`;
    shadowPanel = `0 8px 24px ${shadowColor}0.14)`;
  } else if (elev === 2) {
    shadowBtn = `0 2px 4px ${shadowColor}0.14), 0 1px 2px ${shadowColor}0.08)`;
    shadowItem = `0 2px 5px ${shadowColor}0.10)`;
    shadowPanel = `0 14px 32px ${shadowColor}0.24)`;
  } else if (elev === 3) {
    shadowBtn = `0 4px 8px ${shadowColor}0.22), 0 2px 4px ${shadowColor}0.14)`;
    shadowItem = `0 4px 10px ${shadowColor}0.16)`;
    shadowPanel = `0 20px 42px ${shadowColor}0.34)`;
  } else if (elev >= 4) {
    shadowBtn = `0 6px 14px ${shadowColor}0.30), 0 3px 6px ${shadowColor}0.20)`;
    shadowItem = `0 6px 16px ${shadowColor}0.24)`;
    shadowPanel = `0 28px 56px ${shadowColor}0.44)`;
  }

  r.setProperty('--shadow-btn', shadowBtn);
  r.setProperty('--shadow-item', shadowItem);
  r.setProperty('--shadow-panel', shadowPanel);
  r.setProperty('--theme-elevation', elev.toString());

  // Ajusta color-scheme para que scrollbars e elementos nativos combinem com o tema
  document.documentElement.style.setProperty('color-scheme', isLight ? 'light' : 'dark');
}

/** Heurística simples para detectar se uma cor hex é clara */
function isLightColor(hex: string): boolean {
  const clean = hex.replace('#', '');
  if (clean.length < 6) return false;
  const r = parseInt(clean.slice(0, 2), 16);
  const g = parseInt(clean.slice(2, 4), 16);
  const b = parseInt(clean.slice(4, 6), 16);
  // Luminância perceptual
  return (r * 299 + g * 587 + b * 114) / 1000 > 128;
}

