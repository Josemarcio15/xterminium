import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';

export interface GithubReleaseInfo {
  tag_name: string;
  name: string;
  body: string;
  published_at: string;
  html_url: string;
}

export interface UpdateCheckResult {
  hasUpdate: boolean;
  currentVersion: string;
  latestVersion: string;
  releaseNotes: string;
  releaseUrl: string;
}

const REPO_OWNER = 'Josemarcio15';
const REPO_NAME = 'xterminium';

function parseVersionNumbers(v: string): number[] {
  // Remove 'v' inicial e sufixos como '-alpha', '-beta', etc.
  const clean = v.replace(/^v/i, '').split('-')[0].trim();
  const parts = clean.split('.').map((p) => parseInt(p, 10) || 0);
  while (parts.length < 3) parts.push(0);
  return parts;
}

export function isNewerVersion(current: string, latest: string): boolean {
  const c = parseVersionNumbers(current);
  const l = parseVersionNumbers(latest);

  for (let i = 0; i < 3; i++) {
    if (l[i] > c[i]) return true;
    if (l[i] < c[i]) return false;
  }
  return false;
}

export class UpdateService {
  static async getCurrentVersion(): Promise<string> {
    try {
      return await getVersion();
    } catch {
      try {
        return await invoke<string>('get_app_version');
      } catch {
        return '0.0.0';
      }
    }
  }

  static async checkForUpdates(): Promise<UpdateCheckResult | null> {
    try {
      const currentVersion = await this.getCurrentVersion();
      // Consulta a lista completa de releases para incluir pre-releases (alpha/beta)
      const res = await fetch(`https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases`, {
        headers: {
          Accept: 'application/vnd.github.v3+json',
        },
      });

      if (!res.ok) {
        return null;
      }

      const releases: GithubReleaseInfo[] = await res.json();
      if (!Array.isArray(releases) || releases.length === 0) {
        return null;
      }

      // Pega a primeira release pública publicada (mesmo sendo pre-release)
      const release = releases[0];
      const latestTag = release.tag_name || '';
      const hasUpdate = isNewerVersion(currentVersion, latestTag);

      // Remove blocos legados de instruções de terminal caso a release antiga ainda tenha
      let cleanNotes = (release.body || '').split('---')[0].trim();
      cleanNotes = cleanNotes.replace(/```[\s\S]*?```/g, '').trim();

      return {
        hasUpdate,
        currentVersion,
        latestVersion: latestTag.replace(/^v/i, ''),
        releaseNotes: cleanNotes,
        releaseUrl: release.html_url || `https://github.com/${REPO_OWNER}/${REPO_NAME}/releases`,
      };
    } catch (err) {
      console.error('Falha ao verificar atualizações do xterminium:', err);
      return null;
    }
  }

  static async runUpdate(password?: string): Promise<void> {
    await invoke('run_update_installer', { password: password || null });
  }
}
