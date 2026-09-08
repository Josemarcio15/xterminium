import {
  SftpService,
  type SftpConnectionConfig,
  ConfigService,
} from '../../../core/services';
import type { SshHost } from '../types';

export function createSftpConnection(
  setStatus: (msg: string) => void,
  onConnected: (homeDir: string) => void,
  onDisconnected: () => void,
) {
  let savedHosts = $state<SshHost[]>([]);
  let isConnected = $state(false);
  let isConnecting = $state(false);
  let showPasswordModal = $state(false);
  let initialPassphraseOnly = $state(false);
  let requireVpsPassword = $state(false);
  let modalErrorMessage = $state('');

  let sshHost = $state('');
  let sshUser = $state('');
  let sshPort = $state(22);
  let sshKey = $state<string | undefined>(undefined);
  let currentPassphrase = $state<string | undefined>(undefined);

  async function loadHosts() {
    savedHosts = await ConfigService.loadSshHosts();
  }

  function openPasswordModal(options?: { passphraseOnly?: boolean; needVpsPassword?: boolean; error?: string }) {
    isConnecting = false;
    if (options?.passphraseOnly !== undefined) {
      initialPassphraseOnly = options.passphraseOnly;
    }
    if (options?.needVpsPassword !== undefined) {
      requireVpsPassword = options.needVpsPassword;
    }
    if (options?.error !== undefined) {
      modalErrorMessage = options.error;
    }
    showPasswordModal = true;
  }

  function closePasswordModal() {
    showPasswordModal = false;
    isConnecting = false;
    initialPassphraseOnly = false;
    requireVpsPassword = false;
    modalErrorMessage = '';
    currentPassphrase = undefined;
  }

  async function handlePasswordSubmit(data: { passphrase?: string; password?: string }) {
    if (data.passphrase !== undefined) {
      currentPassphrase = data.passphrase;
    }
    return await connect(data.password, currentPassphrase);
  }

  async function connect(passwordToUse?: string, passphraseToUse?: string) {
    if (!sshHost || !sshUser) {
      setStatus('Informe Host e Usuário');
      return false;
    }

    isConnecting = true;
    modalErrorMessage = '';
    setStatus(`Conectando a ${sshUser}@${sshHost}...`);
    try {
      const config: SftpConnectionConfig = {
        host: sshHost,
        user: sshUser,
        port: sshPort,
        key_path: sshKey,
        key_passphrase: passphraseToUse || currentPassphrase || undefined,
        password: passwordToUse || undefined,
      };
      const home = await SftpService.connect(config);
      isConnected = true;
      setStatus('Conectado com sucesso!');
      closePasswordModal();
      onConnected(home);
      return true;
    } catch (err: any) {
      const errStr = String(err || '');
      isConnected = false;

      // 1. A chave privada requer passphrase
      if (errStr.includes('PASSPHRASE_REQUIRED') || errStr.toLowerCase().includes('passphrase')) {
        setStatus('A chave SSH privada possui senha (passphrase)');
        openPasswordModal({
          passphraseOnly: true,
          needVpsPassword: false,
          error: passphraseToUse ? 'Passphrase incorreta. Tente novamente.' : '',
        });
        return false;
      }

      // 2. Passphrase foi aceita (ou não precisou), mas a VPS agora exige senha SSH
      if (
        !passwordToUse &&
        (errStr.includes('AUTH_FAILED') ||
          errStr.includes('Falha na autenticação') ||
          errStr.includes('autenticação') ||
          errStr.includes('authentication') ||
          errStr.includes('Auth'))
      ) {
        setStatus('Senha necessária para autenticar no servidor SFTP');
        openPasswordModal({
          passphraseOnly: false,
          needVpsPassword: true,
          error: passphraseToUse ? 'Chave validada! Agora informe a senha SSH da VPS:' : '',
        });
        return false;
      }

      // 3. Se já estava fornecendo senha e falhou
      if (passwordToUse) {
        modalErrorMessage = 'Senha da VPS incorreta. Tente novamente.';
        setStatus(`Falha na autenticação: Senha incorreta.`);
      } else {
        setStatus(`Falha na conexão: ${err}`);
      }
      return false;
    } finally {
      isConnecting = false;
    }
  }

  async function connectToHost(host: SshHost) {
    sshHost = host.ip;
    sshUser = host.user;
    sshPort = host.port ? parseInt(host.port, 10) || 22 : 22;
    sshKey = host.key;
    currentPassphrase = undefined;
    initialPassphraseOnly = false;
    requireVpsPassword = false;
    modalErrorMessage = '';
    await connect();
  }

  async function disconnect() {
    try {
      await SftpService.disconnect();
    } catch (_) {}
    isConnected = false;
    setStatus('Desconectado');
    onDisconnected();
  }

  return {
    get savedHosts() { return savedHosts; },
    get isConnected() { return isConnected; },
    get isConnecting() { return isConnecting; },
    get showPasswordModal() { return showPasswordModal; },
    get initialPassphraseOnly() { return initialPassphraseOnly; },
    get requireVpsPassword() { return requireVpsPassword; },
    get modalErrorMessage() { return modalErrorMessage; },
    get sshHost() { return sshHost; },
    get sshUser() { return sshUser; },
    get sshPort() { return sshPort; },
    get sshKey() { return sshKey; },

    loadHosts,
    connect,
    connectToHost,
    disconnect,
    openPasswordModal,
    closePasswordModal,
    handlePasswordSubmit,
    setConnectionInfo(info: { ip: string; user: string; port?: number; key?: string }) {
      sshHost = info.ip;
      sshUser = info.user;
      sshPort = info.port || 22;
      sshKey = info.key;
    },
  };
}
