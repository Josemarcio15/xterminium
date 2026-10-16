<script lang="ts">
  import { onMount } from "svelte";
  import Modal from "../../../shared/components/Modal.svelte";
  import Button from "@/shared/components/Button.svelte";
  import { UpdateService } from "@/core/services/update.service";

  interface Props {
    show: boolean;
    onClose: () => void;
  }

  let { show = false, onClose }: Props = $props();

  let version = $state("Carregando...");
  const author = "Josemarcio15";
  const repoUrl = "https://github.com/Josemarcio15/xterminium";
  const license = "MIT";

  onMount(async () => {
    try {
      version = await UpdateService.getCurrentVersion();
    } catch {
      version = "0.0.26-alpha";
    }
  });

  $effect(() => {
    if (show) {
      UpdateService.getCurrentVersion().then((v) => {
        if (v) version = v;
      }).catch(() => {});
    }
  });
</script>

<Modal {show} title="Sobre o xterminium" widthClass="w-[320px]" {onClose}>
  {#snippet icon()}
    <svg
      class="text-sky-400"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="12" cy="12" r="10"></circle>
      <line x1="12" y1="16" x2="12" y2="12"></line>
      <line x1="12" y1="8" x2="12.01" y2="8"></line>
    </svg>
  {/snippet}

  <div class="flex flex-col items-center text-center py-2">
    <!-- App Logo -->
    <div class="w-12 h-12 rounded-xl bg-white/5 border border-(--border-subtle) flex items-center justify-center p-2 mb-2 shadow-inner">
      <img src="/terminal.svg" alt="xterminium" class="w-8 h-8 object-contain" />
    </div>

    <h3 class="text-sm font-semibold text-(--text-base) tracking-wide">xterminium</h3>
    <p class="text-[11px] text-(--text-muted) mb-3">Terminal Emulator & SSH Client</p>

    <div class="w-full bg-black/20 dark:bg-white/5 rounded-lg border border-(--border-subtle) p-2.5 flex flex-col gap-2 text-xs text-left mb-3">
      <div class="flex items-center justify-between">
        <span class="text-(--text-muted)">Versão</span>
        <span class="font-mono text-[11px] px-1.5 py-0.5 rounded bg-sky-500/10 text-sky-400 border border-sky-500/20 font-medium">
          v{version.replace(/^v/i, '')}
        </span>
      </div>

      <div class="flex items-center justify-between">
        <span class="text-(--text-muted)">Licença</span>
        <span class="text-(--text-base) font-medium">{license}</span>
      </div>

      <div class="flex items-center justify-between">
        <span class="text-(--text-muted)">Autor</span>
        <span class="text-(--text-base) font-medium">{author}</span>
      </div>
    </div>

    <div class="w-full flex justify-end">
      <Button variant="secondary" size="sm" onclick={onClose}>
        Fechar
      </Button>
    </div>
  </div>
</Modal>
