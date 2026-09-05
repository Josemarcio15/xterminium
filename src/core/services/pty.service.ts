import { invoke } from '@tauri-apps/api/core';

export class PtyService {
  static async spawnPty(options: {
    id: string;
    cols?: number;
    rows?: number;
    command?: string;
    args?: string[];
  }): Promise<void> {
    return invoke('spawn_pty', options);
  }

  static async writePty(id: string, data: string): Promise<void> {
    return invoke('write_pty', { id, data });
  }

  static async resizePty(id: string, cols: number, rows: number): Promise<void> {
    return invoke('resize_pty', { id, cols, rows });
  }

  static async closePty(id: string): Promise<void> {
    return invoke('close_pty', { id });
  }
}
