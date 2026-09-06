import { onMount } from 'svelte';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SftpService, type SftpTransferProgress, type FileItem } from '../../../core/services';

export function createSftpTransfer(setStatus: (msg: string) => void) {
  let activeTransfer = $state<SftpTransferProgress | null>(null);
  let isTransferring = $state(false);

  let unlistenProgress: UnlistenFn | null = null;

  onMount(() => {
    let active = true;
    listen<SftpTransferProgress>('sftp://progress', (event) => {
      if (!active) return;
      activeTransfer = event.payload;
      if (event.payload.is_done) {
        setTimeout(() => {
          if (activeTransfer?.file_name === event.payload.file_name) {
            activeTransfer = null;
          }
        }, 1200);
      }
    }).then((unlisten) => {
      unlistenProgress = unlisten;
    });

    return () => {
      active = false;
      if (unlistenProgress) unlistenProgress();
    };
  });

  async function upload(
    selectedLocal: FileItem | null,
    remotePath: string,
    onSuccess: () => Promise<void>,
  ) {
    if (!selectedLocal || selectedLocal.is_dir || !remotePath || isTransferring) return;
    const targetRemotePath = remotePath.endsWith('/')
      ? `${remotePath}${selectedLocal.name}`
      : `${remotePath}/${selectedLocal.name}`;

    isTransferring = true;
    setStatus(`Enviando "${selectedLocal.name}"...`);

    try {
      await SftpService.uploadFile(selectedLocal.path, targetRemotePath);
      setStatus(`Upload de "${selectedLocal.name}" concluído!`);
      await onSuccess();
    } catch (err: any) {
      setStatus(`Erro no upload: ${err}`);
    } finally {
      isTransferring = false;
    }
  }

  async function download(
    selectedRemote: FileItem | null,
    localPath: string,
    onSuccess: () => Promise<void>,
  ) {
    if (!selectedRemote || selectedRemote.is_dir || !localPath || isTransferring) return;
    const targetLocalPath = localPath.endsWith('/')
      ? `${localPath}${selectedRemote.name}`
      : `${localPath}/${selectedRemote.name}`;

    isTransferring = true;
    setStatus(`Baixando "${selectedRemote.name}"...`);

    try {
      await SftpService.downloadFile(selectedRemote.path, targetLocalPath);
      setStatus(`Download de "${selectedRemote.name}" concluído!`);
      await onSuccess();
    } catch (err: any) {
      setStatus(`Erro no download: ${err}`);
    } finally {
      isTransferring = false;
    }
  }

  return {
    get activeTransfer() { return activeTransfer; },
    get isTransferring() { return isTransferring; },
    upload,
    download,
  };
}
