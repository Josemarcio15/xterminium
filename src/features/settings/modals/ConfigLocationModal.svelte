<script lang="ts">
  import Modal from "../../../shared/components/Modal.svelte";
  import Button from "@/shared/components/Button.svelte";
  import IconButton from "@/shared/components/IconButton.svelte";
  import { ConfigService } from "../../../core/services/config.service";
  import { configStore } from "../../../core/stores/config.svelte";

  interface Props {
    show: boolean;
    onClose: () => void;
  }

  let { show = false, onClose }: Props = $props();

  let currentPath = $state("");
  let defaultPath = $state("");
  let isCustom = $state(false);
  let targetPath = $state("");

  let loading = $state(false);
  let errorMessage = $state<string | null>(null);
  let successMessage = $state<string | null>(null);

  // Estado para diálogo de confirmação de conflitos/sobrescrita
  let showConflictDialog = $state(false);
  let conflictingFiles = $state<string[]>([]);

  $effect(() => {
    if (show) {
      loadLocation();
    }
  });

  async function loadLocation() {
    errorMessage = null;
    successMessage = null;
    try {
      const loc = await ConfigService.getConfigLocation();
      currentPath = loc.current_path;
      defaultPath = loc.default_path;
      isCustom = loc.is_custom;
      targetPath = loc.current_path;
    } catch (e: any) {
      errorMessage = typeof e === "string" ? e : e.message || "Erro ao carregar pasta atual";
    }
  }

  async function handlePickFolder() {
    errorMessage = null;
    try {
      const picked = await ConfigService.pickConfigFolder();
      if (picked) {
        targetPath = picked;
      }
    } catch (e: any) {
      errorMessage = typeof e === "string" ? e : e.message || "Erro ao abrir seletor de pasta";
    }
  }

  function handleResetDefault() {
    targetPath = defaultPath;
  }

  async function handleApply() {
    const dest = targetPath.trim();
    if (!dest) {
      errorMessage = "Informe o caminho da nova pasta.";
      return;
    }

    if (dest === currentPath) {
      errorMessage = "O caminho selecionado é o mesmo diretório atual.";
      return;
    }

    loading = true;
    errorMessage = null;
    successMessage = null;

    try {
      // Verificar se a pasta de destino já existe e se há arquivos conflitantes
      const check = await ConfigService.checkMigrationTarget(dest);
      if (check.conflicting_files && check.conflicting_files.length > 0) {
        conflictingFiles = check.conflicting_files;
        showConflictDialog = true;
        loading = false;
        return;
      }

      // Se não houver conflitos, migra direto (sobrescrevendo se aplicável)
      await executeMigration(dest, false);
    } catch (e: any) {
      loading = false;
      errorMessage = typeof e === "string" ? e : e.message || "Erro ao verificar destino";
    }
  }

  async function executeMigration(dest: string, overwrite: boolean) {
    loading = true;
    errorMessage = null;
    try {
      const newPath = await ConfigService.changeConfigLocation(dest, overwrite);
      await configStore.reloadAll();
      await loadLocation();
      targetPath = newPath;
      showConflictDialog = false;
      successMessage = "Local dos arquivos de configuração alterado com sucesso!";
    } catch (e: any) {
      errorMessage = typeof e === "string" ? e : e.message || "Erro ao migrar configurações";
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    showConflictDialog = false;
    errorMessage = null;
    successMessage = null;
    onClose();
  }
</script>

<Modal {show} title="Pasta de Configuração" widthClass="w-[480px]" onClose={handleClose}>
  {#snippet icon()}
    <svg
      class="text-sky-400 shrink-0"
      xmlns="http://www.w3.org/2000/svg"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
      <circle cx="12" cy="13" r="2"></circle>
    </svg>
  {/snippet}

  <div class="flex flex-col gap-3.5 py-1 text-xs text-(--text-base)">
    <!-- Explicação rápida -->
    <p class="text-(--text-muted) leading-relaxed">
      Defina onde os arquivos de configuração do Xterminium (<code class="bg-black/10 dark:bg-white/10 px-1 py-0.5 rounded text-[11px]">ssh.json</code>, <code class="bg-black/10 dark:bg-white/10 px-1 py-0.5 rounded text-[11px]">theme.json</code>, etc.) são salvos. Ao alterar, os arquivos existentes serão movidos para o novo local.
    </p>

    <!-- Pasta Atual -->
    <div class="bg-black/5 dark:bg-white/5 border border-(--border-subtle) rounded-lg p-2.5 flex flex-col gap-1">
      <div class="flex items-center justify-between text-[11px] text-(--text-muted)">
        <span class="font-medium">Diretório em uso:</span>
        {#if isCustom}
          <span class="text-amber-400 font-semibold px-1.5 py-0.2 rounded bg-amber-400/10 text-[10px]">Personalizado</span>
        {:else}
          <span class="text-emerald-400 font-semibold px-1.5 py-0.2 rounded bg-emerald-400/10 text-[10px]">Padrão do Sistema</span>
        {/if}
      </div>
      <div class="font-mono text-[11px] break-all select-all text-(--text-base) bg-black/10 dark:bg-black/30 p-1.5 rounded border border-(--border-subtle)">
        {currentPath || "Carregando..."}
      </div>
    </div>

    <!-- Campo Nova Pasta -->
    <div class="flex flex-col gap-1.5">
      <label for="config-target-path" class="font-medium text-xs text-(--text-base)">
        Novo local de configuração:
      </label>
      <div class="flex items-center gap-1.5">
        <input
          id="config-target-path"
          type="text"
          class="flex-1 bg-black/5 dark:bg-white/5 border border-(--border-subtle) rounded-lg px-2.5 py-1.5 text-xs text-(--text-base) outline-none focus:border-sky-400 font-mono transition-colors"
          placeholder="Caminho completo da pasta..."
          bind:value={targetPath}
          disabled={loading}
        />
        <Button
          variant="secondary"
          size="sm"
          disabled={loading}
          onclick={handlePickFolder}
          title="Procurar pasta no computador"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-1">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          </svg>
          Procurar...
        </Button>
      </div>

      {#if targetPath !== defaultPath}
        <div class="flex justify-end mt-0.5">
          <button
            type="button"
            class="text-[11px] text-sky-400 hover:underline cursor-pointer bg-transparent border-none p-0"
            onclick={handleResetDefault}
          >
            Restaurar para pasta padrão ({defaultPath})
          </button>
        </div>
      {/if}
    </div>

    <!-- Alertas de sucesso / erro -->
    {#if successMessage}
      <div class="p-2 rounded bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 text-xs">
        {successMessage}
      </div>
    {/if}

    {#if errorMessage}
      <div class="p-2 rounded bg-rose-500/10 border border-rose-500/30 text-rose-400 text-xs">
        {errorMessage}
      </div>
    {/if}

    <!-- Modal / Aviso de Conflito de Arquivos -->
    {#if showConflictDialog}
      <div class="bg-amber-500/10 border border-amber-500/30 rounded-xl p-3 flex flex-col gap-2.5">
        <div class="flex items-center gap-2 text-amber-400 font-semibold text-xs">
          <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"></path>
            <line x1="12" y1="9" x2="12" y2="13"></line>
            <line x1="12" y1="17" x2="12.01" y2="17"></line>
          </svg>
          Arquivos já existem no local de destino!
        </div>
        <p class="text-xs text-(--text-muted) leading-relaxed">
          O diretório de destino já contém arquivos de configuração existentes que coincidem com os atuais:
        </p>
        <ul class="text-[11px] font-mono text-amber-300 list-disc list-inside max-h-24 overflow-y-auto bg-black/20 p-2 rounded">
          {#each conflictingFiles as file}
            <li>{file}</li>
          {/each}
        </ul>
        <p class="text-xs text-(--text-base) font-medium">
          Deseja sobrescrever os arquivos já existentes no destino com os seus arquivos atuais?
        </p>
        <div class="flex items-center justify-end gap-2 pt-1">
          <Button
            variant="secondary"
            size="sm"
            disabled={loading}
            onclick={() => showConflictDialog = false}
          >
            Cancelar
          </Button>
          <Button
            variant="secondary"
            size="sm"
            disabled={loading}
            onclick={() => executeMigration(targetPath.trim(), false)}
            title="Mantém os arquivos que já existirem na pasta nova e move apenas os restantes"
          >
            Não Sobrescrever
          </Button>
          <Button
            variant="warn"
            size="sm"
            disabled={loading}
            onclick={() => executeMigration(targetPath.trim(), true)}
            title="Substitui os arquivos da nova pasta pelos arquivos da pasta antiga"
          >
            Sim, Sobrescrever
          </Button>
        </div>
      </div>
    {/if}

    <!-- Botões principais de Ação -->
    {#if !showConflictDialog}
      <div class="flex items-center justify-end gap-2 pt-2 border-t border-(--border-subtle)">
        <Button variant="ghost" size="sm" onclick={handleClose} disabled={loading}>
          Fechar
        </Button>
        <Button
          variant="primary"
          size="sm"
          disabled={loading || targetPath.trim() === currentPath || !targetPath.trim()}
          loading={loading}
          onclick={handleApply}
        >
          Mover e Aplicar
        </Button>
      </div>
    {/if}
  </div>
</Modal>

