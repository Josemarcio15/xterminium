export interface SshHost {
  id: string;
  label?: string;
  user: string;
  ip: string;
  port?: string;
  key?: string;
}
