import { SftpService, type FileItem } from '../../../core/services';
import type { ConfirmDialogState } from './useRemoteFileActions.svelte';
import { isPermissionError } from './useSftpSudo.svelte';

export function createLocalFileActions(
  setStatus: (msg: string) => void,
  refreshLocal: () => Promise<void>,
  setConfirmDialog: (dialog: ConfirmDialogState) => void,
  onRequestSudo?: (actionName: string, command: string, onSuccess: () => Promise<void>) => void,
) {
  function handleCreateFile(localPath: string) {
    if (!localPath) return;
    setConfirmDialog({
      isOpen: true,
      title: 'Novo Arquivo Local',
      message: 'Informe o nome do arquivo que deseja criar na máquina local:',
      inputMode: true,
      inputLabel: 'Nome do arquivo',
      inputPlaceholder: 'exemplo.txt',
      confirmText: 'Criar Arquivo',
      variant: 'primary',
      onConfirm: async (name) => {
        if (!name) return;
        const target = localPath.endsWith('/') ? `${localPath}${name}` : `${localPath}/${name}`;
        try {
          await SftpService.createLocalFile(target);
          setStatus(`Arquivo local "${name}" criado com sucesso.`);
          await refreshLocal();
        } catch (err: any) {
          if (isPermissionError(err) && onRequestSudo) {
            onRequestSudo('Criar Arquivo Local', `touch ${JSON.stringify(target)}`, refreshLocal);
          } else {
            setStatus(`Erro ao criar arquivo local: ${err}`);
          }
        }
      },
    });
  }

  function handleCreateFolder(localPath: string) {
    if (!localPath) return;
    setConfirmDialog({
      isOpen: true,
      title: 'Nova Pasta Local',
      message: 'Informe o nome da pasta que deseja criar na máquina local:',
      inputMode: true,
      inputLabel: 'Nome da pasta',
      inputPlaceholder: 'minha-pasta',
      confirmText: 'Criar Pasta',
      variant: 'primary',
      onConfirm: async (name) => {
        if (!name) return;
        const target = localPath.endsWith('/') ? `${localPath}${name}` : `${localPath}/${name}`;
        try {
          await SftpService.createLocalDir(target);
          setStatus(`Pasta local "${name}" criada com sucesso.`);
          await refreshLocal();
        } catch (err: any) {
          if (isPermissionError(err) && onRequestSudo) {
            onRequestSudo('Criar Pasta Local', `mkdir -p ${JSON.stringify(target)}`, refreshLocal);
          } else {
            setStatus(`Erro ao criar pasta local: ${err}`);
          }
        }
      },
    });
  }

  function handleRename(localPath: string, item: FileItem) {
    if (!item) return;
    setConfirmDialog({
      isOpen: true,
      title: item.is_dir ? 'Renomear Pasta Local' : 'Renomear Arquivo Local',
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
          await SftpService.renameLocal(item.path, newPath);
          setStatus(`"${item.name}" renomeado localmente para "${newName}".`);
          await refreshLocal();
        } catch (err: any) {
          if (isPermissionError(err) && onRequestSudo) {
            onRequestSudo('Renomear Local', `mv ${JSON.stringify(item.path)} ${JSON.stringify(newPath)}`, refreshLocal);
          } else {
            setStatus(`Erro ao renomear item local: ${err}`);
          }
        }
      },
    });
  }

  function handleDelete(item: FileItem) {
    if (!item) return;
    setConfirmDialog({
      isOpen: true,
      title: item.is_dir ? 'Excluir Pasta Local' : 'Excluir Arquivo Local',
      message: `Tem certeza que deseja excluir localmente "${item.name}"? Esta ação não pode ser desfeita.`,
      confirmText: 'Excluir',
      cancelText: 'Cancelar',
      variant: 'danger',
      inputMode: false,
      onConfirm: async () => {
        try {
          if (item.is_dir) {
            await SftpService.removeLocalDir(item.path);
          } else {
            await SftpService.removeLocalFile(item.path);
          }
          setStatus(`"${item.name}" excluído localmente com sucesso.`);
          await refreshLocal();
        } catch (err: any) {
          if (isPermissionError(err) && onRequestSudo) {
            const cmd = item.is_dir ? `rm -rf ${JSON.stringify(item.path)}` : `rm -f ${JSON.stringify(item.path)}`;
            onRequestSudo('Excluir Local', cmd, refreshLocal);
          } else {
            setStatus(`Erro ao excluir localmente: ${err}`);
          }
        }
      },
    });
  }

  return {
    handleCreateFile,
    handleCreateFolder,
    handleRename,
    handleDelete,
  };
}
