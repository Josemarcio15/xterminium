import { invoke } from '@tauri-apps/api/core';

export interface FileItem {
  name: string;
  path: string;
  is_dir: boolean;
  is_symlink: boolean;
  size: number;
  modified?: number;
}

export interface SftpConnectionConfig {
  host: string;
  port?: number;
  user: string;
  password?: string;
  key_path?: string;
}

export class SftpService {
  static async connect(config: SftpConnectionConfig): Promise<string> {
    return await invoke<string>('sftp_connect', {
      host: config.host,
      port: config.port || 22,
      user: config.user,
      password: config.password || null,
      keyPath: config.key_path || null,
    });
  }

  static async disconnect(): Promise<void> {
    return await invoke<void>('sftp_disconnect');
  }

  static async listRemote(path: string): Promise<FileItem[]> {
    return await invoke<FileItem[]>('sftp_list_remote', { path });
  }

  static async listLocal(path?: string): Promise<FileItem[]> {
    return await invoke<FileItem[]>('sftp_list_local', { path: path || null });
  }

  static async getLocalHome(): Promise<string> {
    return await invoke<string>('sftp_get_local_home');
  }

  static async upload(localPath: string, remoteDir: string): Promise<void> {
    return await invoke<void>('sftp_upload_item', {
      localPath,
      remoteDir,
    });
  }

  static async download(remotePath: string, localDir: string): Promise<void> {
    return await invoke<void>('sftp_download_item', {
      remotePath,
      localDir,
    });
  }

  static formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }
}
