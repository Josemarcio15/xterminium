<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Button, { type ButtonVariant } from '@/shared/components/Button.svelte';

  export type IconButtonSize = 'xs' | 'sm' | 'md' | 'lg';

  interface Props extends HTMLButtonAttributes {
    variant?: ButtonVariant;
    size?: IconButtonSize;
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

  const sizeClasses: Record<IconButtonSize, string> = {
    xs: 'w-[22px] h-[22px] p-0 text-[10px]',
    sm: 'w-7 h-7 p-1 text-xs',
    md: 'w-8 h-8 p-1.5 text-xs',
    lg: 'w-9 h-9 p-2 text-sm',
  };
</script>

<Button
  {type}
  {variant}
  size="icon"
  {disabled}
  {loading}
  class="shrink-0 {sizeClasses[size]} {extraClass}"
  {...rest}
>
  {#if children}
    {@render children()}
  {/if}
</Button>
