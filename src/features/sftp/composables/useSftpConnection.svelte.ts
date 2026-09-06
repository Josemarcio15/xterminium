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

  let sshHost = $state('');
  let sshUser = $state('');
  let sshPort = $state(22);
  let sshKey = $state<string | undefined>(undefined);

  async function loadHosts() {
    savedHosts = await ConfigService.loadSshHosts();
  }

  function openPasswordModal() {
    isConnecting = false;
    showPasswordModal = true;
  }

  function closePasswordModal() {
    showPasswordModal = false;
    isConnecting = false;
  }

  async function handlePasswordSubmit(password: string) {
    await connect(password);
  }

  async function connect(passwordToUse?: string) {
    if (!sshHost || !sshUser) {
      setStatus('Informe Host e Usuário');
      return;
    }

    isConnecting = true;
    setStatus(`Conectando a ${sshUser}@${sshHost}...`);
    try {
      const config: SftpConnectionConfig = {
        host: sshHost,
        user: sshUser,
        port: sshPort,
        key_path: sshKey,
        password: passwordToUse || undefined,
      };
      const home = await SftpService.connect(config);
      isConnected = true;
      setStatus('Conectado com sucesso!');
      showPasswordModal = false;
      onConnected(home);
    } catch (err: any) {
      const errStr = String(err || '');
      if (
        !passwordToUse &&
        (errStr.includes('Falha na autenticação') ||
          errStr.includes('autenticação') ||
          errStr.includes('authentication') ||
          errStr.includes('Auth'))
      ) {
        setStatus('Senha necessária para autenticar no servidor SFTP');
        openPasswordModal();
      } else {
        setStatus(`Falha na conexão: ${err}`);
      }
      isConnected = false;
    } finally {
      isConnecting = false;
    }
  }

  async function connectToHost(host: SshHost) {
    sshHost = host.ip;
    sshUser = host.user;
    sshPort = host.port ? parseInt(host.port, 10) || 22 : 22;
    sshKey = host.key;
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
