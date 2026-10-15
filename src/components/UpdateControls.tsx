import { useCallback, useEffect, useRef, useState } from "react";
import type { Update } from "@tauri-apps/plugin-updater";
import {
  findUpdate, installUpdate, isAutoUpdateEnabled, setAutoUpdateEnabled, shouldCheckAtStartup,
} from "@/lib/autoUpdate";

type Phase = "idle" | "checking" | "uptodate" | "available" | "installing" | "error";

export function UpdateControls() {
  const [auto, setAuto] = useState(isAutoUpdateEnabled);
  const [phase, setPhase] = useState<Phase>("idle");
  const [update, setUpdate] = useState<Update | null>(null);
  const [pct, setPct] = useState<number | null>(0);
  const [error, setError] = useState("");
  const [open, setOpen] = useState(false);
  const busy = useRef(false);

  const run = useCallback(async (silent: boolean) => {
    if (busy.current) return;
    busy.current = true;
    if (!silent) setPhase("checking");
    try {
      const u = await findUpdate();
      if (u) { setUpdate(u); setPhase("available"); }
      else if (!silent) setPhase("uptodate");
    } catch (e) {
      if (!silent) { setError(String(e)); setPhase("error"); }
    } finally { busy.current = false; }
  }, []);

  useEffect(() => {
    if (shouldCheckAtStartup(import.meta.env.DEV, isAutoUpdateEnabled())) void run(true);
  }, [run]);

  const install = async () => {
    if (!update) return;
    setPhase("installing");
    try { await installUpdate(update, setPct); }
    catch (e) { setError(String(e)); setPhase("error"); }
  };

  const toggle = (on: boolean) => { setAuto(on); setAutoUpdateEnabled(on); };

  return (
    <>
      <div className="relative">
        <button onClick={() => setOpen((o) => !o)}
          className="text-xs px-3 py-1.5 rounded-lg bg-secondary text-foreground hover:opacity-80">
          Mises à jour
        </button>
        {open && (
          <div className="absolute right-0 mt-2 w-72 p-4 rounded-xl bg-secondary border border-border shadow-2xl z-40 space-y-3 text-sm">
            <button onClick={() => void run(false)} disabled={phase === "checking" || phase === "installing"}
              className="w-full px-3 py-2 rounded-lg bg-primary text-white disabled:opacity-50">
              {phase === "checking" ? "Recherche en cours…" : "Rechercher une mise à jour"}
            </button>
            {phase === "uptodate" && <p className="text-muted-foreground">PureRemove est à jour.</p>}
            {phase === "error" && <p className="text-destructive">Échec : {error}</p>}
            <label className="flex items-center gap-2 cursor-pointer">
              <input type="checkbox" checked={auto} onChange={(e) => toggle(e.target.checked)} />
              Vérifier les mises à jour automatiquement
            </label>
          </div>
        )}
      </div>

      {(phase === "available" || phase === "installing") && update && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
          <div className="w-[28rem] max-w-[90vw] p-6 rounded-2xl bg-secondary border border-border space-y-4">
            <h2 className="text-lg font-bold text-foreground">Mise à jour disponible</h2>
            <p className="text-sm text-foreground">
              La version {update.version} est disponible (vous avez la {update.currentVersion}).
            </p>
            {update.body && (
              <pre className="text-xs text-muted-foreground whitespace-pre-wrap max-h-48 overflow-auto">{update.body}</pre>
            )}
            {phase === "installing" ? (
              <div>
                <div className="h-2 rounded bg-background overflow-hidden">
                  <div className="h-full bg-primary transition-all" style={{ width: `${pct ?? 100}%` }} />
                </div>
                <p className="text-xs text-muted-foreground mt-2">
                  {pct === 100 ? "Installation puis redémarrage…" : `Téléchargement ${pct ?? ""}${pct != null ? " %" : "…"}`}
                </p>
              </div>
            ) : (
              <div className="flex justify-end gap-2">
                <button onClick={() => setPhase("idle")} className="px-4 py-2 text-sm rounded-lg bg-background text-foreground">Plus tard</button>
                <button onClick={() => void install()} className="px-4 py-2 text-sm rounded-lg bg-primary text-white">Installer et redémarrer</button>
              </div>
            )}
          </div>
        </div>
      )}
    </>
  );
}
