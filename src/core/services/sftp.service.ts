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

export interface SftpTransferProgress {
  direction: 'upload' | 'download';
  file_name: string;
  file_path: string;
  transferred_bytes: number;
  total_bytes: number;
  percentage: number;
  is_done: boolean;
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

  static async createDir(path: string): Promise<void> {
    return await invoke<void>('sftp_create_dir', { path });
  }

  static async createFile(path: string): Promise<void> {
    return await invoke<void>('sftp_create_file', { path });
  }

  static async rename(oldPath: string, newPath: string): Promise<void> {
    return await invoke<void>('sftp_rename', { oldPath, newPath });
  }

  static async removeFile(path: string): Promise<void> {
    return await invoke<void>('sftp_remove_file', { path });
  }

  static async removeDir(path: string): Promise<void> {
    return await invoke<void>('sftp_remove_dir', { path });
  }

  static async createLocalDir(path: string): Promise<void> {
    return await invoke<void>('sftp_create_local_dir', { path });
  }

  static async createLocalFile(path: string): Promise<void> {
    return await invoke<void>('sftp_create_local_file', { path });
  }

  static async renameLocal(oldPath: string, newPath: string): Promise<void> {
    return await invoke<void>('sftp_rename_local', { oldPath, newPath });
  }

  static async removeLocalFile(path: string): Promise<void> {
    return await invoke<void>('sftp_remove_local_file', { path });
  }

  static async removeLocalDir(path: string): Promise<void> {
    return await invoke<void>('sftp_remove_local_dir', { path });
  }

  static async downloadFile(remotePath: string, localPath: string): Promise<void> {
    return await invoke<void>('sftp_download_file', { remotePath, localPath });
  }

  static async uploadFile(localPath: string, remotePath: string): Promise<void> {
    return await invoke<void>('sftp_upload_file', { localPath, remotePath });
  }

  static async calculateLocalHash(localPath: string): Promise<string> {
    return await invoke<string>('sftp_calculate_local_hash', { localPath });
  }

  static async calculateRemoteHash(remotePath: string): Promise<string> {
    return await invoke<string>('sftp_calculate_remote_hash', { remotePath });
  }

  static async execRemoteSudo(password: string, command: string): Promise<void> {
    return await invoke<void>('sftp_exec_remote_sudo', { password, command });
  }

  static async execLocalSudo(password: string, command: string): Promise<void> {
    return await invoke<void>('sftp_exec_local_sudo', { password, command });
  }

  static formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }
}
