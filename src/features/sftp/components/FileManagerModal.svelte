<script lang="ts">
  import type { SshHost, FileItem, SftpTransferProgress } from '../types';
  import { createSftpConnection } from '../composables/useSftpConnection.svelte';
  import { createSftpFileSystem } from '../composables/useSftpFileSystem.svelte';
  import { createSftpTransfer } from '../composables/useSftpTransfer.svelte';
  import { createRemoteFileActions } from '../composables/useRemoteFileActions.svelte';
  import { createLocalFileActions } from '../composables/useLocalFileActions.svelte';
  import { createSftpNotifications } from '../composables/useSftpNotifications.svelte';
  import { createSftpSudo } from '../composables/useSftpSudo.svelte';

  import PasswordModal from './PasswordModal.svelte';
  import SudoPasswordModal from './SudoPasswordModal.svelte';
  import ServerListPanel from './ServerListPanel.svelte';
  import FileListPanel from './FileListPanel.svelte';
  import TransferControls from './TransferControls.svelte';
  import FileManagerHeader from './FileManagerHeader.svelte';
  import ChecksumModal from './ChecksumModal.svelte';
  import NotificationsModal from './NotificationsModal.svelte';
  import { ConfirmModal } from '../../../shared/components';

  interface Props {
    isOpen?: boolean;
    isViewMode?: boolean;
    initialLocalPath?: string;
    currentSshInfo?: SshHost;
    onClose?: () => void;
  }

  let {
    isOpen = true,
    isViewMode = false,
    initialLocalPath,
    currentSshInfo,
    onClose,
  }: Props = $props();

  // Estados Globais de UI
  let showHiddenFiles = $state(false);
  let showChecksumModal = $state(false);

  // Composable de Notificações
  const notif = createSftpNotifications();

  function setStatus(msg: string) {
    notif.addNotification(msg);
  }

  // Composables por Responsabilidade
  const fs = createSftpFileSystem(setStatus);

  const conn = createSftpConnection(
    setStatus,
    (homeDir) => {
      fs.remotePath = homeDir;
      fs.loadRemote(homeDir);
    },
    () => {
      fs.clearRemote();
      notif.clearNotifications();
    },
  );

  const transfer = createSftpTransfer(setStatus);
  const sudo = createSftpSudo(setStatus);

  const actions = createRemoteFileActions(
    setStatus,
    async () => {
      await fs.loadRemote(fs.remotePath);
    },
    (actionName, cmd, onSuccess) => sudo.requestRemoteSudo(actionName, cmd, onSuccess),
  );

  const localActions = createLocalFileActions(
    setStatus,
    async () => {
      await fs.loadLocal(fs.localPath);
    },
    (dialog) => actions.setDialog(dialog),
    (actionName, cmd, onSuccess) => sudo.requestLocalSudo(actionName, cmd, onSuccess),
  );

  // Filtros de Arquivos Ocultos
  let filteredLocalFiles = $derived(
    showHiddenFiles ? fs.localFiles : fs.localFiles.filter((f) => !f.name.startsWith('.'))
  );

  let filteredRemoteFiles = $derived(
    showHiddenFiles ? fs.remoteFiles : fs.remoteFiles.filter((f) => !f.name.startsWith('.'))
  );

  $effect(() => {
    if (isOpen) {
      conn.loadHosts();

      if (initialLocalPath) {
        fs.loadLocal(initialLocalPath);
      } else {
        fs.loadLocal();
      }

      if (currentSshInfo?.ip && currentSshInfo?.user) {
        conn.setConnectionInfo({
          ip: currentSshInfo.ip,
          user: currentSshInfo.user,
          port: currentSshInfo.port ? parseInt(currentSshInfo.port, 10) || 22 : 22,
          key: currentSshInfo.key,
        });
        conn.connect();
      }
    }
  });

  function handleCloseModal() {
    conn.disconnect();
    onClose?.();
  }
</script>

{#if isOpen}
  {#snippet content()}
    <div class="w-full h-full flex flex-col overflow-hidden bg-[#0f111a] text-sm text-gray-200">
      <!-- Cabeçalho Principal Modularizado com Sininho e Notificações -->
      <FileManagerHeader
        showHiddenFiles={showHiddenFiles}
        notificationCount={notif.unreadCount}
        activeToast={notif.activeToast}
        isViewMode={isViewMode}
        onToggleHiddenFiles={(val) => (showHiddenFiles = val)}
        onOpenNotifications={notif.openHistory}
        onCloseModal={handleCloseModal}
      />

      <!-- Área de Conteúdo dos Painéis -->
      <div class="flex-1 flex overflow-hidden p-3 gap-2">
        <!-- PAINEL LOCAL (ESQUERDA) -->
        <FileListPanel
          title="Local"
          path={fs.localPath}
          files={filteredLocalFiles}
          selectedFile={fs.selectedLocal}
          loading={fs.loadingLocal}
          tagColor="purple"
          isRemote={false}
          activeTransfer={transfer.activeTransfer?.direction === 'upload' ? transfer.activeTransfer : null}
          onSelect={(item) => (fs.selectedLocal = item)}
          onDoubleClick={(item) => {
            if (item.is_dir) fs.loadLocal(item.path);
          }}
          onGoUp={fs.goUpLocal}
          onNavigate={(newPath) => fs.loadLocal(newPath)}
          onCreateFile={() => localActions.handleCreateFile(fs.localPath)}
          onCreateFolder={() => localActions.handleCreateFolder(fs.localPath)}
          onRename={(item) => localActions.handleRename(fs.localPath, item)}
          onDelete={(item) => localActions.handleDelete(item)}
        />

        <!-- CONTROLES CENTRAIS (ENVIAR / BAIXAR / CHECKSUM) -->
        {#if conn.isConnected}
          <TransferControls
            canUpload={!!fs.selectedLocal && !fs.selectedLocal.is_dir}
            canDownload={!!fs.selectedRemote && !fs.selectedRemote.is_dir}
            canChecksum={(!!fs.selectedLocal && !fs.selectedLocal.is_dir) || (!!fs.selectedRemote && !fs.selectedRemote.is_dir)}
            isTransferring={transfer.isTransferring}
            onUpload={() => transfer.upload(fs.selectedLocal, fs.remotePath, () => fs.loadRemote(fs.remotePath))}
            onDownload={() => transfer.download(fs.selectedRemote, fs.localPath, () => fs.loadLocal(fs.localPath))}
            onChecksum={() => (showChecksumModal = true)}
          />
        {/if}

        <!-- PAINEL REMOTO (DIREITA) -->
        {#if !conn.isConnected}
          <div class="flex-1 flex flex-col bg-[#12141d] border border-white/5 rounded-lg overflow-hidden">
            <ServerListPanel
              hosts={conn.savedHosts}
              isConnecting={conn.isConnecting}
              activeConnectingHost={conn.sshHost}
              onSelectHost={conn.connectToHost}
            />
          </div>
        {:else}
          {#snippet remoteActions()}
            <button
              onclick={conn.disconnect}
              class="p-1 text-red-400 hover:text-red-200 hover:bg-red-500/20 rounded border border-red-500/30 transition-all cursor-pointer flex items-center justify-center"
              title="Desconectar do SFTP"
              aria-label="Desconectar"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path>
                <polyline points="16 17 21 12 16 7"></polyline>
                <line x1="21" y1="12" x2="9" y2="12"></line>
              </svg>
            </button>
          {/snippet}

          <FileListPanel
            title="Remoto"
            path={fs.remotePath}
            files={filteredRemoteFiles}
            selectedFile={fs.selectedRemote}
            loading={fs.loadingRemote}
            tagColor="blue"
            isRemote={true}
            activeTransfer={transfer.activeTransfer?.direction === 'download' ? transfer.activeTransfer : null}
            onSelect={(item) => (fs.selectedRemote = item)}
            onDoubleClick={(item) => {
              if (item.is_dir) fs.loadRemote(item.path);
            }}
            onGoUp={fs.goUpRemote}
            onNavigate={(newPath) => fs.loadRemote(newPath)}
            onCreateFile={() => actions.handleCreateFile(conn.isConnected, fs.remotePath)}
            onCreateFolder={() => actions.handleCreateFolder(conn.isConnected, fs.remotePath)}
            onRename={(item) => actions.handleRename(conn.isConnected, fs.remotePath, item)}
            onDelete={(item) => actions.handleDelete(conn.isConnected, item)}
            extraActionsSnippet={remoteActions}
          />
        {/if}
      </div>
    </div>
  {/snippet}

  {#if isViewMode}
    {@render content()}
  {:else}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 select-none"
      onclick={(e) => {
        if (e.target === e.currentTarget) handleCloseModal();
      }}
    >
      <div
        class="bg-[#181a24] border border-white/10 rounded-xl shadow-2xl w-[92vw] max-w-5xl h-[80vh] flex flex-col overflow-hidden text-sm text-gray-200"
      >
        {@render content()}
      </div>
    </div>
  {/if}

  <!-- Mini Modal de Senha -->
  <PasswordModal
    isOpen={conn.showPasswordModal}
    sshUser={conn.sshUser}
    sshHost={conn.sshHost}
    isConnecting={conn.isConnecting}
    onSubmit={conn.handlePasswordSubmit}
    onClose={conn.closePasswordModal}
  />

  <!-- Modal de Verificação SHA-256 -->
  <ChecksumModal
    isOpen={showChecksumModal}
    localFile={fs.selectedLocal}
    remoteFile={fs.selectedRemote}
    onClose={() => (showChecksumModal = false)}
  />

  <!-- Modal de Histórico de Notificações (Limpa ao fechar) -->
  <NotificationsModal
    isOpen={notif.showHistoryModal}
    notifications={notif.notifications}
    onClose={notif.closeHistory}
  />

  <!-- Modal de Confirmação / Criação / Renomeação nativo do app -->
  <ConfirmModal
    isOpen={actions.confirmDialog.isOpen}
    title={actions.confirmDialog.title}
    message={actions.confirmDialog.message}
    confirmText={actions.confirmDialog.confirmText}
    cancelText={actions.confirmDialog.cancelText}
    variant={actions.confirmDialog.variant}
    inputMode={actions.confirmDialog.inputMode}
    inputLabel={actions.confirmDialog.inputLabel}
    inputValue={actions.confirmDialog.inputValue}
    inputPlaceholder={actions.confirmDialog.inputPlaceholder}
    onConfirm={actions.confirmDialog.onConfirm}
    onClose={actions.closeDialog}
  />

  <!-- Modal de Senha Sudo para operações com permissão negada -->
  <SudoPasswordModal
    isOpen={sudo.sudoPrompt.isOpen}
    title={sudo.sudoPrompt.title}
    description={sudo.sudoPrompt.description}
    isSubmitting={sudo.sudoPrompt.isSubmitting}
    errorMessage={sudo.sudoPrompt.errorMessage}
    onSubmit={(pwd) => sudo.handleSudoSubmit(pwd)}
    onClose={sudo.closeSudoPrompt}
  />
{/if}
