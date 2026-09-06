import { SftpService } from '../../../core/services';

export interface SudoPromptState {
  isOpen: boolean;
  title: string;
  description: string;
  isSubmitting: boolean;
  errorMessage: string;
  command: string;
  isRemote: boolean;
  onSuccess: () => Promise<void>;
}

export function isPermissionError(err: any): boolean {
  if (!err) return false;
  const msg = String(err).toLowerCase();
  return (
    msg.includes('permission denied') ||
    msg.includes('permissão negada') ||
    msg.includes('eacces') ||
    msg.includes('status: failure') ||
    msg.includes('status: 4') ||
    msg.includes('operation not permitted')
  );
}

export function createSftpSudo(setStatus: (msg: string) => void) {
  let sudoPrompt = $state<SudoPromptState>({
    isOpen: false,
    title: 'Acesso Privilegiado (sudo)',
    description: '',
    isSubmitting: false,
    errorMessage: '',
    command: '',
    isRemote: true,
    onSuccess: async () => {},
  });

  function closeSudoPrompt() {
    sudoPrompt.isOpen = false;
    sudoPrompt.errorMessage = '';
    sudoPrompt.isSubmitting = false;
  }

  function requestRemoteSudo(
    actionName: string,
    command: string,
    onSuccess: () => Promise<void>,
  ) {
    sudoPrompt = {
      isOpen: true,
      title: `Permissão Negada: ${actionName}`,
      description: `A tentativa via SFTP falhou por permissão. Digite a senha de usuário para executar com sudo no servidor:`,
      isSubmitting: false,
      errorMessage: '',
      command,
      isRemote: true,
      onSuccess,
    };
  }

  function requestLocalSudo(
    actionName: string,
    command: string,
    onSuccess: () => Promise<void>,
  ) {
    sudoPrompt = {
      isOpen: true,
      title: `Permissão Negada Local: ${actionName}`,
      description: `A tentativa local falhou por falta de permissão. Digite sua senha sudo para executar como root:`,
      isSubmitting: false,
      errorMessage: '',
      command,
      isRemote: false,
      onSuccess,
    };
  }

  async function handleSudoSubmit(password: string) {
    if (!password) return;
    sudoPrompt.isSubmitting = true;
    sudoPrompt.errorMessage = '';

    try {
      if (sudoPrompt.isRemote) {
        await SftpService.execRemoteSudo(password, sudoPrompt.command);
      } else {
        await SftpService.execLocalSudo(password, sudoPrompt.command);
      }
      closeSudoPrompt();
      setStatus('Operação concluída com sucesso com privilégios sudo.');
      await sudoPrompt.onSuccess();
    } catch (err: any) {
      sudoPrompt.errorMessage = String(err);
      sudoPrompt.isSubmitting = false;
    }
  }

  return {
    get sudoPrompt() {
      return sudoPrompt;
    },
    closeSudoPrompt,
    requestRemoteSudo,
    requestLocalSudo,
    handleSudoSubmit,
  };
}
