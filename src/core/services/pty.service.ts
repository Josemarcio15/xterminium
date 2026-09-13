import { invoke } from "@tauri-apps/api/core";
import type { ShellProfile } from "../types";

export class PtyService {
  static async spawnPty(options: {
    id: string;
    cols?: number;
    rows?: number;
    command?: string;
    args?: string[];
  }): Promise<void> {
    return invoke("spawn_pty", options);
  }

  /** Lista os shells instalados no sistema (PowerShell, CMD, WSL, zsh, bash, ...). */
  static async listShells(): Promise<ShellProfile[]> {
    return invoke<ShellProfile[]>("list_shells");
  }

  static async writePty(id: string, data: string): Promise<void> {
    return invoke("write_pty", { id, data });
  }

  static async resizePty(
    id: string,
    cols: number,
    rows: number,
  ): Promise<void> {
    return invoke("resize_pty", { id, cols, rows });
  }

  static async closePty(id: string): Promise<void> {
    return invoke("close_pty", { id });
  }
}
