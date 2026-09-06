<script lang="ts">
  import IconButton from '@/shared/components/IconButton.svelte';

  interface Props {
    canUpload: boolean;
    canDownload: boolean;
    canChecksum: boolean;
    isTransferring: boolean;
    onUpload: () => void;
    onDownload: () => void;
    onChecksum: () => void;
  }

  let {
    canUpload,
    canDownload,
    canChecksum,
    isTransferring,
    onUpload,
    onDownload,
    onChecksum,
  }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center gap-2 px-1 select-none">
  <!-- Botão Checar Integridade SHA-256 -->
  <IconButton
    onclick={onChecksum}
    disabled={!canChecksum}
    variant={canChecksum ? 'success' : 'secondary'}
    size="md"
    class="mb-1"
    title="Verificar Hash SHA-256 do(s) arquivo(s) selecionado(s)"
  >
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
      <path d="m9 12 2 2 4-4"></path>
    </svg>
  </IconButton>

  <!-- Divisor sutil -->
  <div class="w-4 h-[1px] bg-[var(--border-subtle)] my-0.5"></div>

  <!-- Botão Upload (Local -> Remoto) -->
  <IconButton
    onclick={onUpload}
    disabled={!canUpload || isTransferring}
    variant="primary"
    size="md"
    title="Enviar para o servidor remoto (Upload)"
  >
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
      <line x1="5" y1="12" x2="19" y2="12"></line>
      <polyline points="12 5 19 12 12 19"></polyline>
    </svg>
  </IconButton>

  <!-- Botão Download (Remoto -> Local) -->
  <IconButton
    onclick={onDownload}
    disabled={!canDownload || isTransferring}
    variant="primary"
    size="md"
    title="Baixar para a máquina local (Download)"
  >
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
      <line x1="19" y1="12" x2="5" y2="12"></line>
      <polyline points="12 19 5 12 12 5"></polyline>
    </svg>
  </IconButton>
</div>

