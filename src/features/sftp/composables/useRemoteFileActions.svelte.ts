import { SftpService, type FileItem } from '../../../core/services';
import { isPermissionError } from './useSftpSudo.svelte';

export interface ConfirmDialogState {
  isOpen: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  variant?: 'danger' | 'warning' | 'info' | 'primary';
  inputMode?: boolean;
  inputLabel?: string;
  inputValue?: string;
  inputPlaceholder?: string;
  onConfirm: (val?: string) => void;
}

export function createRemoteFileActions(
  setStatus: (msg: string) => void,
  refreshRemote: () => Promise<void>,
  onRequestSudo?: (actionName: string, command: string, onSuccess: () => Promise<void>) => void,
) {
  let confirmDialog = $state<ConfirmDialogState>({
    isOpen: false,
    title: '',
    message: '',
    onConfirm: () => {},
  });

  function closeDialog() {
    confirmDialog.isOpen = false;
  }

  function handleCreateFile(isConnected: boolean, remotePath: string) {
    if (!isConnected || !remotePath) return;
    confirmDialog = {
      isOpen: true,
      title: 'Novo Arquivo',
      message: 'Informe o nome do arquivo que deseja criar no servidor:',
      inputMode: true,
      inputLabel: 'Nome do arquivo',
      inputPlaceholder: 'exemplo.txt',
      confirmText: 'Criar Arquivo',
      variant: 'primary',
      onConfirm: async (name) => {
        if (!name) return;
        const target = remotePath.endsWith('/') ? `${remotePath}${name}` : `${remotePath}/${name}`;
        try {
          await SftpService.createFile(target);
          setStatus(`Arquivo "${name}" criado com sucesso.`);
          await refreshRemote();
        } catch (err: any) {
          if (isPermissionError(err) && onRequestSudo) {
            onRequestSudo('Criar Arquivo', `touch ${JSON.stringify(target)}`, refreshRemote);
          } else {
            setStatus(`Erro ao criar arquivo: ${err}`);
          }
        }
      },
    };
  }

  function handleCreateFolder(isConnected: boolean, remotePath: string) {
    if (!isConnected || !remotePath) return;
    confirmDialog = {
      isOpen: true,
      title: 'Nova Pasta',
      message: 'Informe o nome do diretório que deseja criar:',
      inputMode: true,
      inputLabel: 'Nome da pasta',
      inputPlaceholder: 'minha-pasta',
      confirmText: 'Criar Pasta',
      variant: 'primary',
      onConfirm: async (name) => {
        if (!name) return;
        const target = remotePath.endsWith('/') ? `${remotePath}${name}` : `${remotePath}/${name}`;
        try {
          await SftpService.createDir(target);
          setStatus(`Pasta "${name}" criada com sucesso.`);
          await refreshRemote();
        } catch (err: any) {
          if (isPermissionError(err) && onRequestSudo) {
            onRequestSudo('Criar Pasta', `mkdir -p ${JSON.stringify(target)}`, refreshRemote);
          } else {
            setStatus(`Erro ao criar pasta: ${err}`);
          }
        }
      },
    };
  }

  function handleRename(isConnected: boolean, remotePath: string, item: FileItem) {
    if (!isConnected || !item) return;
    confirmDialog = {
      isOpen: true,
      title: item.is_dir ? 'Renomear Pasta' : 'Renomear Arquivo',
      message: `Digite o novo nome para "${item.name}":`,
      inputMode: true,
      inputLabel: 'Novo nome',
      inputValue: item.name,
      confirmText: 'Renomear',
      variant: 'primary',
      onConfirm: async (newName) => {
        if (!newName || newName === item.name) return;
        const parentDir = item.path.substring(0, item.path.lastIndexOf('/')) || '/';
        const newPath = parentDir.endsWith('/') ? `${parentDir}${newName}` : `${parentDir}/${newName}`;

        try {
          await SftpService.rename(item.path, newPath);
          setStatus(`"${item.name}" renomeado para "${newName}".`);
          await refreshRemote();
        } catch (err: any) {
          if (isPermissionError(err) && onRequestSudo) {
            onRequestSudo('Renomear/Mover', `mv ${JSON.stringify(item.path)} ${JSON.stringify(newPath)}`, refreshRemote);
          } else {
            setStatus(`Erro ao renomear: ${err}`);
          }
        }
      },
    };
  }

  function handleDelete(isConnected: boolean, item: FileItem) {
    if (!isConnected || !item) return;
    confirmDialog = {
      isOpen: true,
      title: item.is_dir ? 'Excluir Pasta' : 'Excluir Arquivo',
      message: `Tem certeza que deseja excluir "${item.name}"? Esta ação não pode ser desfeita.`,
      confirmText: 'Excluir',
      cancelText: 'Cancelar',
      variant: 'danger',
      inputMode: false,
      onConfirm: async () => {
        try {
          if (item.is_dir) {
            await SftpService.removeDir(item.path);
          } else {
            await SftpService.removeFile(item.path);
          }
          setStatus(`"${item.name}" excluído com sucesso.`);
          await refreshRemote();
        } catch (err: any) {
          if (isPermissionError(err) && onRequestSudo) {
            const cmd = item.is_dir ? `rm -rf ${JSON.stringify(item.path)}` : `rm -f ${JSON.stringify(item.path)}`;
            onRequestSudo('Excluir', cmd, refreshRemote);
          } else {
            setStatus(`Erro ao excluir: ${err}`);
          }
        }
      },
    };
  }

  return {
    get confirmDialog() { return confirmDialog; },
    setDialog: (dialog: ConfirmDialogState) => { confirmDialog = dialog; },
    closeDialog,
    handleCreateFile,
    handleCreateFolder,
    handleRename,
    handleDelete,
  };
}
