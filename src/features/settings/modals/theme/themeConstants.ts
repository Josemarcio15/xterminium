import type { AppTheme } from "../../../../core/types";

export interface ColorField {
  key: keyof AppTheme;
  label: string;
  group: string;
  allowOpacity?: boolean;
}

export const colorFields: ColorField[] = [
  { key: "bgBase", label: "Fundo Principal", group: "Backgrounds", allowOpacity: true },
  { key: "bgTitlebar", label: "Titlebar", group: "Backgrounds", allowOpacity: true },
  { key: "bgPanel", label: "Painéis / Modais", group: "Backgrounds", allowOpacity: true },
  { key: "bgItem", label: "Itens de Lista", group: "Backgrounds", allowOpacity: true },
  {
    key: "bgItemHover",
    label: "Hover de Itens/Linhas",
    group: "Backgrounds",
    allowOpacity: true,
  },
  { key: "bgItemInput", label: "Inputs", group: "Backgrounds", allowOpacity: true },
  { key: "bgTabActive", label: "Aba Ativa", group: "Backgrounds", allowOpacity: true },
  { key: "bgTabHover", label: "Hover de Abas", group: "Backgrounds", allowOpacity: true },
  {
    key: "accentPrimary",
    label: "Acento Principal (SSH)",
    group: "Cores de Acento",
    allowOpacity: true,
  },
  { key: "accentSecondary", label: "Acento Local", group: "Cores de Acento", allowOpacity: true },
  { key: "accentSftp", label: "Acento SFTP", group: "Cores de Acento", allowOpacity: true },
  { key: "accentWarn", label: "Aviso / Comandos", group: "Cores de Acento", allowOpacity: true },
  { key: "terminalBg", label: "Fundo do Terminal", group: "Terminal", allowOpacity: true },
  { key: "terminalFg", label: "Texto do Terminal", group: "Terminal", allowOpacity: true },
  { key: "terminalCursorLocal", label: "Cursor Local", group: "Terminal", allowOpacity: true },
  { key: "terminalCursorSsh", label: "Cursor SSH", group: "Terminal", allowOpacity: true },
  { key: "textBase", label: "Texto Principal", group: "Fontes & Textos", allowOpacity: true },
  { key: "textMuted", label: "Texto Secundário", group: "Fontes & Textos", allowOpacity: true },
  { key: "textFaint", label: "Texto Suave", group: "Fontes & Textos", allowOpacity: true },
  { key: "textSpecial", label: "Textos Especiais (Templates)", group: "Fontes & Textos", allowOpacity: true },
  {
    key: "btnPrimaryBg",
    label: "Fundo Botão Primário",
    group: "Botões: Ação Principal",
    allowOpacity: true,
  },
  {
    key: "btnPrimaryText",
    label: "Texto Botão Primário",
    group: "Botões: Ação Principal",
    allowOpacity: true,
  },
  {
    key: "btnPrimaryHover",
    label: "Hover Botão Primário",
    group: "Botões: Ação Principal",
    allowOpacity: true,
  },
  {
    key: "btnSecondaryBg",
    label: "Fundo Botão Secundário",
    group: "Botões: Secundários",
    allowOpacity: true,
  },
  {
    key: "btnSecondaryText",
    label: "Texto Botão Secundário",
    group: "Botões: Secundários",
    allowOpacity: true,
  },
  {
    key: "btnSecondaryHover",
    label: "Hover Botão Secundário",
    group: "Botões: Secundários",
    allowOpacity: true,
  },
  {
    key: "btnDangerBg",
    label: "Fundo Botão Perigo",
    group: "Botões: Perigo & Remoção",
    allowOpacity: true,
  },
  {
    key: "btnDangerText",
    label: "Texto Botão Perigo",
    group: "Botões: Perigo & Remoção",
    allowOpacity: true,
  },
  {
    key: "btnDangerHover",
    label: "Hover Botão Perigo",
    group: "Botões: Perigo & Remoção",
    allowOpacity: true,
  },
  {
    key: "btnSuccessBg",
    label: "Fundo Botão Sucesso",
    group: "Botões: Sucesso",
    allowOpacity: true,
  },
  {
    key: "btnSuccessText",
    label: "Texto Botão Sucesso",
    group: "Botões: Sucesso",
    allowOpacity: true,
  },
  {
    key: "btnGlassBg",
    label: "Fundo Botão Vidro (Glass)",
    group: "Botões: Vidro & Translúcido",
    allowOpacity: true,
  },
  {
    key: "btnGlassText",
    label: "Texto Botão Vidro (Glass)",
    group: "Botões: Vidro & Translúcido",
    allowOpacity: true,
  },
  {
    key: "btnGlassHover",
    label: "Hover Botão Vidro (Glass)",
    group: "Botões: Vidro & Translúcido",
    allowOpacity: true,
  },
  {
    key: "btnBorder",
    label: "Borda Global de Botões",
    group: "Botões: Aparência & Borda",
    allowOpacity: true,
  },
];

export const themeColorGroups = [...new Set(colorFields.map((f) => f.group))];
