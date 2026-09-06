<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  export type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'success' | 'warn' | 'ghost' | 'glass' | 'tab' | 'tab-active';
  export type ButtonSize = 'xs' | 'sm' | 'md' | 'icon' | 'tab';

  interface Props extends HTMLButtonAttributes {
    variant?: ButtonVariant;
    size?: ButtonSize;
    loading?: boolean;
    class?: string;
    children?: Snippet;
  }

  let {
    variant = 'secondary',
    size = 'sm',
    loading = false,
    disabled = false,
    type = 'button',
    class: extraClass = '',
    children,
    ...rest
  }: Props = $props();

  const sizeClasses: Record<ButtonSize, string> = {
    xs: 'px-2 py-0.5 text-[10px]',
    sm: 'px-2.5 py-1 text-xs',
    md: 'px-3.5 py-1.5 text-xs',
    icon: 'p-1.5 text-xs',
    tab: 'px-2.5 py-1 text-xs gap-1.5 whitespace-nowrap',
  };

  const variantClasses: Record<ButtonVariant, string> = {
    primary: 'bg-[var(--btn-primary-bg)] hover:bg-[var(--btn-primary-hover)] text-[var(--btn-primary-text)] border-[var(--btn-border)]',
    secondary: 'bg-[var(--btn-secondary-bg)] hover:bg-[var(--btn-secondary-hover)] text-[var(--btn-secondary-text)] hover:text-[var(--text-base)] border-[var(--btn-border)]',
    danger: 'bg-[var(--btn-danger-bg)] hover:bg-[var(--btn-danger-hover)] text-[var(--btn-danger-text)] border-[var(--btn-border)]',
    success: 'bg-[var(--btn-success-bg)] text-[var(--btn-success-text)] border-[var(--btn-border)]',
    warn: 'bg-[var(--accent-warn)] hover:brightness-110 text-slate-950 border-[var(--btn-border)]',
    glass: 'bg-[var(--btn-glass-bg)] hover:bg-[var(--btn-glass-hover)] text-[var(--btn-glass-text)] border-[var(--btn-border)] backdrop-blur-sm',
    tab: 'bg-black/[0.02] dark:bg-white/[0.03] text-[var(--text-muted)] border-transparent hover:bg-[var(--bg-tab-hover)] hover:text-[var(--text-base)] hover:border-[var(--btn-border)] shadow-none',
    'tab-active': 'bg-[var(--bg-tab-active)] text-[var(--text-base)] border-[var(--btn-border)] shadow-xs font-medium',
    ghost: 'bg-transparent hover:bg-black/5 dark:hover:bg-white/10 text-[var(--text-muted)] hover:text-[var(--text-base)] border-transparent shadow-none',
  };
</script>

<button
  {type}
  disabled={disabled || loading}
  class="btn-swing inline-flex items-center justify-center font-medium rounded-[var(--btn-radius)] border shadow-[var(--shadow-btn)] transition-all select-none cursor-pointer {sizeClasses[size]} {variantClasses[variant]} {extraClass} {disabled || loading ? 'opacity-50 cursor-not-allowed pointer-events-none' : ''}"
  {...rest}
>
  {#if loading}
    <svg class="animate-spin -ml-0.5 mr-1.5 h-3.5 w-3.5 text-current" fill="none" viewBox="0 0 24 24">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
    </svg>
  {/if}
  {#if children}
    {@render children()}
  {/if}
</button>

<style>
  /* Efeito de botão evidente (estilo Swing/Desktop tátil) */
  .btn-swing {
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.14), inset 0 1px 0 rgba(255, 255, 255, 0.12);
  }
  .btn-swing:hover:not(:disabled) {
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.18), inset 0 1px 0 rgba(255, 255, 255, 0.2);
  }
  .btn-swing:active:not(:disabled) {
    transform: translateY(1px);
    box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.25);
  }
</style>
