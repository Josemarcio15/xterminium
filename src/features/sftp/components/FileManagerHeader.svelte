<script lang="ts">
  import Button from '@/shared/components/Button.svelte';

  interface Props {
    title?: string;
    showHiddenFiles: boolean;
    notificationCount: number;
    activeToast: { id: string; message: string; collapsing: boolean } | null;
    isViewMode: boolean;
    onToggleHiddenFiles: (val: boolean) => void;
    onOpenNotifications: () => void;
    onCloseModal: () => void;
  }

  let {
    title = 'Explorador de Arquivos (SFTP)',
    showHiddenFiles,
    notificationCount,
    activeToast,
    isViewMode,
    onToggleHiddenFiles,
    onOpenNotifications,
    onCloseModal,
  }: Props = $props();
</script>

<div class="flex items-center justify-between px-4 py-2.5 bg-[var(--bg-titlebar)] border-b border-[var(--border-subtle)] shrink-0 select-none relative">
  <div class="flex items-center gap-2">
    <svg class="w-4 h-4 text-[var(--accent-sftp)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="2"
        d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
      />
    </svg>
    <span class="font-semibold text-[var(--text-base)] tracking-wide text-xs">{title}</span>
  </div>

  <div class="flex items-center gap-3 relative">
    <label class="flex items-center gap-1.5 text-xs text-[var(--text-muted)] hover:text-[var(--text-base)] cursor-pointer select-none">
      <input
        type="checkbox"
        checked={showHiddenFiles}
        onchange={(e) => onToggleHiddenFiles(e.currentTarget.checked)}
        class="w-3.5 h-3.5 rounded border-[var(--border-subtle)] bg-[var(--bg-item-input)] text-blue-500 focus:ring-0 cursor-pointer"
      />
      <span>Mostrar ocultos</span>
    </label>

    <!-- Container relativo ao Sininho -->
    <div class="relative flex items-center">
      <!-- Balão de Fala estilo Quadrinhos com Ponta apontando para o Sininho -->
      {#if activeToast}
        <div
          class="absolute right-full mr-3.5 top-1/2 -translate-y-1/2 z-50 pointer-events-none transition-all duration-400 ease-out origin-right {activeToast.collapsing
            ? 'opacity-0 scale-50 translate-x-3'
            : 'opacity-100 scale-100 translate-x-0'}"
        >
          <div
            class="relative px-4 py-2.5 bg-gradient-to-r from-[#0c2340] to-[#123055] border border-sky-400/60 rounded-2xl shadow-xl shadow-sky-950/60 text-xs text-sky-100 font-semibold flex items-center gap-2.5 whitespace-nowrap"
          >
            <span class="w-2.5 h-2.5 rounded-full bg-sky-400 animate-pulse shadow-[0_0_8px_rgba(56,189,248,0.8)] shrink-0"></span>
            <span class="max-w-[280px] truncate tracking-wide">{activeToast.message}</span>

            <!-- Ponta do Balão de Quadrinhos (triângulo apontando para a direita no sino) -->
            <div
              class="absolute top-1/2 -right-[8px] -translate-y-1/2 w-0 h-0 border-y-[7px] border-y-transparent border-l-[9px] border-l-sky-400/60"
            ></div>
            <div
              class="absolute top-1/2 -right-[6px] -translate-y-1/2 w-0 h-0 border-y-[6px] border-y-transparent border-l-[8px] border-l-[#123055]"
            ></div>
          </div>
        </div>
      {/if}

      <!-- Botão Sininho de Notificações -->
      <button
        onclick={onOpenNotifications}
        class="p-1.5 rounded-lg border border-[var(--border-subtle)] hover:border-amber-500/40 bg-[var(--bg-item)] hover:bg-amber-500/10 text-[var(--text-muted)] hover:text-amber-400 transition-all cursor-pointer relative flex items-center justify-center group active:scale-95"
        title="Notificações da conexão SFTP"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="transition-transform group-hover:rotate-12">
          <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"></path>
          <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"></path>
        </svg>

        <!-- Badge com contagem de notificações -->
        {#if notificationCount > 0}
          <span class="absolute -top-1 -right-1 min-w-[15px] h-[15px] px-0.5 bg-[var(--accent-sftp)] text-white text-[9px] font-bold rounded-full flex items-center justify-center border border-[var(--bg-titlebar)] shadow">
            {notificationCount > 9 ? '9+' : notificationCount}
          </span>
        {/if}
      </button>
    </div>

    {#if !isViewMode}
      <Button
        variant="danger"
        size="xs"
        onclick={onCloseModal}
        title="Fechar e encerrar conexão"
      >
        <span>Fechar</span>
        <svg class="w-3.5 h-3.5 ml-1" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </Button>
    {/if}
  </div>
</div>

