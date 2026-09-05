export interface TabItem {
  id: string;
  title: string;
  type: 'local' | 'ssh' | 'sftp';
  sshInfo?: import('./ssh').SshHost;
}
