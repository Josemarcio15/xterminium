import { SftpService, type FileItem } from '../../../core/services';

export interface SftpFileSystemState {
  localPath: string;
  localFiles: FileItem[];
  selectedLocal: FileItem | null;
  loadingLocal: boolean;
  remotePath: string;
  remoteFiles: FileItem[];
  selectedRemote: FileItem | null;
  loadingRemote: boolean;
}

export function createSftpFileSystem(setStatus: (msg: string) => void) {
  let localPath = $state('');
  let localFiles = $state<FileItem[]>([]);
  let selectedLocal = $state<FileItem | null>(null);
  let loadingLocal = $state(false);

  let remotePath = $state('');
  let remoteFiles = $state<FileItem[]>([]);
  let selectedRemote = $state<FileItem | null>(null);
  let loadingRemote = $state(false);

  async function loadLocal(path?: string) {
    loadingLocal = true;
    try {
      const items = await SftpService.listLocal(path);
      localFiles = items;
      if (path) {
        localPath = path;
      } else if (items.length > 0) {
        const first = items[0].path;
        localPath = first.substring(0, first.lastIndexOf('/')) || '/';
      }
      selectedLocal = null;
    } catch (err: any) {
      setStatus(`Erro local: ${err}`);
    } finally {
      loadingLocal = false;
    }
  }

  function goUpLocal() {
    if (!localPath || localPath === '/') return;
    const parent = localPath.substring(0, localPath.lastIndexOf('/')) || '/';
    loadLocal(parent);
  }

  async function loadRemote(path: string) {
    loadingRemote = true;
    try {
      const items = await SftpService.listRemote(path);
      remoteFiles = items;
      remotePath = path;
      selectedRemote = null;
    } catch (err: any) {
      setStatus(`Erro remoto: ${err}`);
    } finally {
      loadingRemote = false;
    }
  }

  function goUpRemote() {
    if (!remotePath || remotePath === '/') return;
    const parent = remotePath.substring(0, remotePath.lastIndexOf('/')) || '/';
    loadRemote(parent);
  }

  function clearRemote() {
    remoteFiles = [];
    remotePath = '';
    selectedRemote = null;
  }

  return {
    get localPath() { return localPath; },
    set localPath(v) { localPath = v; },
    get localFiles() { return localFiles; },
    get selectedLocal() { return selectedLocal; },
    set selectedLocal(v) { selectedLocal = v; },
    get loadingLocal() { return loadingLocal; },

    get remotePath() { return remotePath; },
    set remotePath(v) { remotePath = v; },
    get remoteFiles() { return remoteFiles; },
    get selectedRemote() { return selectedRemote; },
    set selectedRemote(v) { selectedRemote = v; },
    get loadingRemote() { return loadingRemote; },

    loadLocal,
    goUpLocal,
    loadRemote,
    goUpRemote,
    clearRemote,
  };
}
