<script lang="ts">
  import { SftpService, type FileItem } from '../../../core/services';

  interface Props {
    item: FileItem;
    isSelected: boolean;
    activeTransfer: {
      file_name: string;
      percentage: number;
      transferred_bytes: number;
      total_bytes: number;
      direction: 'upload' | 'download';
    } | null;
    onSelect: (item: FileItem) => void;
    onDoubleClick: (item: FileItem) => void;
  }

  let {
    item,
    isSelected,
    activeTransfer,
    onSelect,
    onDoubleClick,
  }: Props = $props();

  let isCurrentTransfer = $derived(activeTransfer && activeTransfer.file_name === item.name);
  let transferProgress = $derived(isCurrentTransfer ? activeTransfer!.percentage : 0);
</script>

<div
  role="button"
  tabindex="0"
  onclick={() => onSelect(item)}
  ondblclick={() => onDoubleClick(item)}
  onkeydown={(e) => {
    if (e.key === 'Enter') onDoubleClick(item);
    else if (e.key === ' ') {
      e.preventDefault();
      onSelect(item);
    }
  }}
  class="relative flex items-center justify-between px-2.5 py-1.5 rounded cursor-pointer transition-all text-xs overflow-hidden {isSelected
    ? 'border-b-2 ' + (isCurrentTransfer ? 'border-purple-500 text-purple-200' : 'border-blue-500 bg-blue-600/30 text-blue-200') + ' font-medium'
    : (isCurrentTransfer ? 'border-b-2 border-purple-500 text-purple-200 font-medium' : 'hover:bg-white/5 text-gray-300')}"
  style={isCurrentTransfer
    ? (isSelected
        ? `background: linear-gradient(to right, rgba(147, 51, 234, 0.7) 0%, rgba(168, 85, 247, 0.6) ${transferProgress}%, rgba(37, 99, 235, 0.35) ${transferProgress}%);`
        : `background: linear-gradient(to right, rgba(147, 51, 234, 0.6) 0%, rgba(168, 85, 247, 0.5) ${transferProgress}%, rgba(255, 255, 255, 0.03) ${transferProgress}%);`)
    : ''}
>
  <!-- Ícone + Nome do Arquivo -->
  <div class="relative z-10 flex items-center gap-2 truncate flex-1 min-w-0">
    {#if item.is_dir}
      <svg class="w-4 h-4 text-amber-400 shrink-0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor">
        <path d="M19.5 21a3 3 0 0 0 3-3v-4.5a3 3 0 0 0-3-3h-1.5V9a3 3 0 0 0-3-3h-4.5l-2-2H4.5A3 3 0 0 0 1.5 7v11a3 3 0 0 0 3 3h15z" />
      </svg>
    {:else}
      <svg class="w-4 h-4 shrink-0 transition-colors {isCurrentTransfer ? 'text-purple-300' : (isSelected ? 'text-blue-300' : 'text-gray-400')}" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
        <polyline points="14 2 14 8 20 8"></polyline>
      </svg>
    {/if}
    <span
      class="truncate font-medium transition-colors {isCurrentTransfer ? 'text-purple-100 font-semibold drop-shadow-[0_1px_2px_rgba(0,0,0,0.8)]' : (isSelected ? 'text-blue-200' : '')}"
    >
      {item.name}
    </span>
  </div>

  <!-- Porcentagem roxa ou Tamanho do Arquivo -->
  {#if isCurrentTransfer}
    <div class="relative z-10 shrink-0 flex items-center gap-2 font-mono text-[11px] font-bold text-purple-200 drop-shadow">
      <span>{transferProgress.toFixed(0)}%</span>
    </div>
  {:else}
    <!-- Tamanho do Arquivo -->
    <div class="relative z-10 shrink-0 flex items-center gap-2 font-mono text-[10px]">
      <span class={isSelected ? 'text-blue-300/80' : 'text-gray-500'}>
        {item.is_dir ? 'Pasta' : SftpService.formatFileSize(item.size)}
      </span>
    </div>
  {/if}
</div>
