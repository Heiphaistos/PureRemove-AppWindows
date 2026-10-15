import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

const KEY = "pureremove.autoUpdate";

/** Réglage « vérifier automatiquement » : actif par défaut. */
export function isAutoUpdateEnabled(): boolean {
  try { return localStorage.getItem(KEY) !== "off"; } catch { return true; }
}

export function setAutoUpdateEnabled(on: boolean): void {
  try { localStorage.setItem(KEY, on ? "on" : "off"); } catch { /* stockage indisponible */ }
}

/** Vérification silencieuse au démarrage : jamais en dev, seulement si le réglage est actif. */
export function shouldCheckAtStartup(dev: boolean, enabled: boolean): boolean {
  return !dev && enabled;
}

export async function findUpdate(): Promise<Update | null> {
  return check();
}

/** Télécharge (progression 0-100, ou null si taille inconnue), installe puis relance. */
export async function installUpdate(update: Update, onProgress: (pct: number | null) => void): Promise<void> {
  let total = 0;
  let done = 0;
  await update.downloadAndInstall((ev) => {
    if (ev.event === "Started") total = ev.data.contentLength ?? 0;
    else if (ev.event === "Progress") {
      done += ev.data.chunkLength;
      onProgress(total > 0 ? Math.min(100, Math.round((done / total) * 100)) : null);
    } else onProgress(100);
  });
  await relaunch();
}
