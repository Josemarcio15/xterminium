<script lang="ts">
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
  <button
    onclick={onChecksum}
    disabled={!canChecksum}
    class="p-2.5 rounded-lg border transition-all flex items-center justify-center mb-1 {canChecksum
      ? 'bg-emerald-600/30 hover:bg-emerald-600/50 text-emerald-300 hover:text-white shadow-lg shadow-emerald-500/20 border-emerald-400/40 cursor-pointer active:scale-95'
      : 'bg-white/5 text-gray-600 border-white/5 cursor-not-allowed opacity-50'}"
    title="Verificar Hash SHA-256 do(s) arquivo(s) selecionado(s)"
  >
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
      <path d="m9 12 2 2 4-4"></path>
    </svg>
  </button>

  <!-- Divisor sutil -->
  <div class="w-4 h-[1px] bg-white/10 my-0.5"></div>
  <!-- Botão Upload (Local -> Remoto) -->
  <button
    onclick={onUpload}
    disabled={!canUpload || isTransferring}
    class="p-2.5 rounded-lg border transition-all flex items-center justify-center {canUpload && !isTransferring
      ? 'bg-blue-600 hover:bg-blue-500 text-white shadow-lg shadow-blue-500/20 border-blue-400/40 cursor-pointer active:scale-95'
      : 'bg-white/5 text-gray-600 border-white/5 cursor-not-allowed opacity-50'}"
    title="Enviar para o servidor remoto (Upload)"
  >
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
      <line x1="5" y1="12" x2="19" y2="12"></line>
      <polyline points="12 5 19 12 12 19"></polyline>
    </svg>
  </button>

  <!-- Botão Download (Remoto -> Local) -->
  <button
    onclick={onDownload}
    disabled={!canDownload || isTransferring}
    class="p-2.5 rounded-lg border transition-all flex items-center justify-center {canDownload && !isTransferring
      ? 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-lg shadow-indigo-500/20 border-indigo-400/40 cursor-pointer active:scale-95'
      : 'bg-white/5 text-gray-600 border-white/5 cursor-not-allowed opacity-50'}"
    title="Baixar para a máquina local (Download)"
  >
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
      <line x1="19" y1="12" x2="5" y2="12"></line>
      <polyline points="12 19 5 12 12 5"></polyline>
    </svg>
  </button>
</div>
