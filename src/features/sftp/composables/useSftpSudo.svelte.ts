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
    msg.includes('operation not permitted') ||
    msg.includes('código 15') ||
    msg.includes('código 13') ||
    msg.includes('cannot create') ||
    msg.includes('could not create')
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

  // Estado da sessão root/sudo persistente
  let isRootModeActive = $state(false);
  let savedPassword = $state<string | null>(null);

  function deactivateRootMode() {
    isRootModeActive = false;
    savedPassword = null;
    setStatus('Modo Root desativado.');
  }

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
    // Se o modo root já está ativo com senha salva, executa diretamente sem abrir o modal!
    if (isRootModeActive && savedPassword) {
      executeWithSavedPassword(savedPassword, command, onSuccess, true);
      return;
    }

    sudoPrompt = {
      isOpen: true,
      title: `Permissão Negada: ${actionName}`,
      description: `A tentativa via SFTP falhou por permissão. Digite a senha para autorizar com sudo:`,
      isSubmitting: false,
      errorMessage: '',
      command,
      isRemote: true,
      onSuccess,
    };
  }

  function requestEnableRootMode(onSuccess?: () => Promise<void>) {
    sudoPrompt = {
      isOpen: true,
      title: 'Ativar Modo Root (Sudo)',
      description: 'Digite sua senha sudo para ativar a elevação contínua nesta sessão SFTP:',
      isSubmitting: false,
      errorMessage: '',
      command: 'echo ok',
      isRemote: true,
      onSuccess: async () => {
        if (onSuccess) await onSuccess();
      },
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

  async function executeWithSavedPassword(
    password: string,
    command: string,
    onSuccess: () => Promise<void>,
    isRemote: boolean,
  ) {
    try {
      setStatus('Executando ação com permissões de root...');
      if (isRemote) {
        await SftpService.execRemoteSudo(password, command);
      } else {
        await SftpService.execLocalSudo(password, command);
      }
      setStatus('Operação concluída com sucesso com privilégios sudo.');
      await onSuccess();
    } catch (err: any) {
      // Se a senha falhar, desativa o modo root para pedir novamente
      const errStr = String(err);
      if (errStr.toLowerCase().includes('senha sudo incorreta')) {
        deactivateRootMode();
      }
      setStatus(`Erro ao executar como root: ${errStr}`);
      // Abre o modal informando o erro
      sudoPrompt = {
        isOpen: true,
        title: 'Falha na Autorização Sudo',
        description: 'A credencial de root não foi aceita ou expirou. Por favor, reintroduza a senha:',
        isSubmitting: false,
        errorMessage: errStr,
        command,
        isRemote,
        onSuccess,
      };
    }
  }

  async function handleSudoSubmit(password: string, rememberForSession: boolean = true) {
    if (!password) return;
    sudoPrompt.isSubmitting = true;
    sudoPrompt.errorMessage = '';

    try {
      if (sudoPrompt.isRemote) {
        await SftpService.execRemoteSudo(password, sudoPrompt.command);
      } else {
        await SftpService.execLocalSudo(password, sudoPrompt.command);
      }

      // Se for remoto e marcado para lembrar (padrão true), ativa o modo root contínuo
      if (sudoPrompt.isRemote && rememberForSession) {
        isRootModeActive = true;
        savedPassword = password;
        setStatus('Modo Root ativado: operações com permissão negada serão elevadas automaticamente.');
      } else {
        setStatus('Operação concluída com sucesso com privilégios sudo.');
      }

      closeSudoPrompt();
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
    get isRootModeActive() {
      return isRootModeActive;
    },
    deactivateRootMode,
    requestEnableRootMode,
    closeSudoPrompt,
    requestRemoteSudo,
    requestLocalSudo,
    handleSudoSubmit,
  };
}
