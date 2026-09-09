import { invoke } from '@tauri-apps/api/core';

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
const CURRENT_VERSION = '0.0.7-alpha';

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
  static async checkForUpdates(): Promise<UpdateCheckResult | null> {
    try {
      const res = await fetch(`https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest`, {
        headers: {
          Accept: 'application/vnd.github.v3+json',
        },
      });

      if (!res.ok) {
        return null;
      }

      const release: GithubReleaseInfo = await res.json();
      const latestTag = release.tag_name || '';
      const hasUpdate = isNewerVersion(CURRENT_VERSION, latestTag);

      return {
        hasUpdate,
        currentVersion: CURRENT_VERSION,
        latestVersion: latestTag.replace(/^v/i, ''),
        releaseNotes: release.body || '',
        releaseUrl: release.html_url || `https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest`,
      };
    } catch (err) {
      console.error('Falha ao verificar atualizações do xterminium:', err);
      return null;
    }
  }

  static async runUpdate(): Promise<void> {
    await invoke('run_update_installer');
  }
}
