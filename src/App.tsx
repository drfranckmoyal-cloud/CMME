// Coquille de l'application : verrouillage, navigation, préférences d'affichage.
import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { call, errMessage, inTauri } from "./api";
import type { CatalogBundle, EncounterFull, Filter, Status } from "./types";
import { LockScreen } from "./screens/Lock";
import { Home } from "./screens/Home";
import { Consultation } from "./screens/Consultation";
import { Dossiers } from "./screens/Dossiers";
import { Stats } from "./screens/Stats";
import { ImportScreen } from "./screens/Import";
import { Study } from "./screens/Study";
import { Settings } from "./screens/Settings";
import { Modal } from "./components/Modal";
import { confirmAsk } from "./lib/confirm";
import { ChartColumn, ClipboardList, FileUp, FlaskConical, HardDrive, House, Layers, Lock, type LucideIcon } from "lucide-react";

export interface Prefs { theme: "iris" | "lagune"; scheme: "system" | "light" | "dark"; lockMinutes: number }
type Screen = "home" | "consultation" | "dossiers" | "stats" | "import" | "study" | "settings";

const loadPrefs = (): Prefs => {
  try { return { theme: "iris", scheme: "system", lockMinutes: 10, ...JSON.parse(localStorage.getItem("cmme.prefs") ?? "{}") }; } catch { return { theme: "iris", scheme: "system", lockMinutes: 10 }; }
};

const NAV: [Screen, string, LucideIcon][] = [
  ["home", "Accueil", House], ["consultation", "Consultation", ClipboardList], ["dossiers", "Dossiers", Layers], ["stats", "Statistiques", ChartColumn],
  ["import", "Reprise", FileUp], ["study", "Étude et exports", FlaskConical], ["settings", "Sauvegarde", HardDrive],
];

export default function App() {
  const [status, setStatus] = useState<Status | null>(null);
  const [bundle, setBundle] = useState<CatalogBundle | null>(null);
  const [screen, setScreen] = useState<Screen>("home");
  const [encounterId, setEncounterId] = useState<string | null>(null);
  const [filter, setFilter] = useState<Filter>({});
  const [prefs, setPrefsState] = useState<Prefs>(loadPrefs);
  const [newOpen, setNewOpen] = useState(false);
  const [fatal, setFatal] = useState<string | null>(null);
  const flushRef = useRef<(() => Promise<boolean>) | null>(null);
  const scrollKey = useRef(0);

  const setPrefs = (p: Prefs) => { setPrefsState(p); try { localStorage.setItem("cmme.prefs", JSON.stringify(p)); } catch { /* préférences non conservées */ } };
  useEffect(() => {
    document.documentElement.dataset.theme = prefs.theme;
    document.documentElement.dataset.scheme = prefs.scheme;
  }, [prefs]);

  const refreshStatus = useCallback(async () => {
    try { setStatus(await call<Status>("app_status")); } catch (e) { setFatal(errMessage(e)); }
  }, []);
  useEffect(() => { void refreshStatus(); }, [refreshStatus]);

  useEffect(() => {
    if (status?.unlocked && !bundle) call<CatalogBundle>("get_catalog").then(setBundle).catch((e) => setFatal(errMessage(e)));
  }, [status?.unlocked, bundle]);

  const registerFlush = useCallback((fn: (() => Promise<boolean>) | null) => { flushRef.current = fn; }, []);

  /** Toute navigation enregistre d'abord la saisie en cours ; en cas d'échec, on demande. */
  const leave = useCallback(async (): Promise<boolean> => {
    if (!flushRef.current) return true;
    const ok = await flushRef.current();
    if (ok) return true;
    return confirmAsk("Des saisies n'ont pas pu être enregistrées (voir le message en haut de la consultation). Quitter quand même et les perdre ?", "Quitter");
  }, []);

  const go = useCallback(async (s: Screen) => {
    if (s === screen) return;
    if (!(await leave())) return;
    if (s === "consultation" && !encounterId) { setNewOpen(true); return; }
    setScreen(s);
    if (s !== "dossiers") window.scrollTo({ top: 0 });
  }, [screen, leave, encounterId]);

  const openEncounter = useCallback(async (id: string) => {
    if (!(await leave())) return;
    setEncounterId(id);
    setScreen("consultation");
    window.scrollTo({ top: 0 });
  }, [leave]);

  const lock = useCallback(async () => {
    if (flushRef.current) await flushRef.current();
    try { await call("lock"); } catch { /* déjà verrouillée */ }
    setBundle(null); setEncounterId(null); setScreen("home");
    await refreshStatus();
  }, [refreshStatus]);

  // Verrouillage après inactivité et au retour de veille (saut d'horloge).
  useEffect(() => {
    if (!status?.unlocked || (status.profile === "clinique" && !status.password_required)) return;
    let last = Date.now();
    let tick = Date.now();
    const bump = () => { last = Date.now(); };
    const evs = ["mousemove", "keydown", "mousedown", "wheel"];
    evs.forEach((e) => window.addEventListener(e, bump, { passive: true }));
    const t = window.setInterval(() => {
      const now = Date.now();
      const slept = now - tick > 60_000;
      tick = now;
      if (slept || now - last > prefs.lockMinutes * 60_000) void lock();
    }, 15_000);
    return () => { window.clearInterval(t); evs.forEach((e) => window.removeEventListener(e, bump)); };
  }, [status?.unlocked, status?.profile, status?.password_required, prefs.lockMinutes, lock]);

  // Fermeture de la fenêtre : enregistrer d'abord.
  useEffect(() => {
    if (!inTauri()) return;
    let unlisten: (() => void) | undefined;
    getCurrentWindow().onCloseRequested(async (e) => {
      if (flushRef.current) {
        const ok = await flushRef.current();
        if (!ok && !(await confirmAsk("Des saisies ne sont pas enregistrées. Fermer quand même ?", "Fermer"))) e.preventDefault();
      }
    }).then((u) => { unlisten = u; }).catch(() => undefined);
    return () => unlisten?.();
  }, []);

  if (fatal) return <div className="lock-screen"><div className="lock-card"><div className="brand">cmme<b>.</b></div><div className="banner error">{fatal}</div></div></div>;
  if (!status) return <div className="lock-screen" />;
  if (!status.unlocked) return <LockScreen status={status} onUnlocked={refreshStatus} />;
  if (!bundle) return <div className="lock-screen" />;

  const demo = status.profile === "demo";
  const clinical = !demo;
  const initials = (status.practitioner ?? "Démo").split(/\s+/).map((x) => x[0]).join("").slice(0, 2).toUpperCase();

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="brand">cmme<b>.</b><small>Recueil dentaire</small></div>
        <nav className="appnav" aria-label="Navigation principale">
          {NAV.map(([s, label, Icon]) => (
            <button key={s} type="button" aria-current={screen === s ? "page" : undefined} onClick={() => go(s)}>
              <Icon size={18} strokeWidth={1.8} aria-hidden="true" />{label}
            </button>
          ))}
        </nav>
        <div className="account">
          <div className="avatar" aria-hidden="true">{initials}</div>
          <div className="who">{demo ? "Démonstration" : status.practitioner}<small>{demo ? "données fictives" : "Sainte-Anne · CMME"}</small></div>
          <button type="button" className="lockbtn" onClick={lock} title="Verrouiller" aria-label="Verrouiller l'application"><Lock size={17} strokeWidth={1.8} /></button>
        </div>
      </aside>
      <div className="workspace">
        <div className="topbar">
          <span>{demo ? "Démonstration" : "Espace clinique"} / <strong>{NAV.find((n) => n[0] === screen)?.[1]}</strong></span>
          <span className={`profile-dot ${demo ? "demo" : ""}`}>{demo ? "Profil de démonstration · données fictives" : "Local · chiffré · hors ligne"}</span>
        </div>
        {screen === "home" && <Home practitioner={demo ? null : status.practitioner} demo={demo} onNew={() => setNewOpen(true)} onSearch={() => go("dossiers")} onImport={() => go("import")} onOpen={openEncounter} onBackup={() => go("settings")} />}
        {screen === "consultation" && encounterId && (
          <Consultation key={encounterId} bundle={bundle} encounterId={encounterId} clinical={clinical} registerFlush={registerFlush}
            onOpenDossiers={() => go("dossiers")} onNewDossier={() => setNewOpen(true)} onOpenEncounter={openEncounter} />
        )}
        {screen === "dossiers" && <Dossiers filter={filter} setFilter={setFilter} clinical={clinical} onOpen={openEncounter} onNew={() => setNewOpen(true)} scrollKey={scrollKey} />}
        {screen === "stats" && <Stats filter={filter} setFilter={setFilter} demo={demo} />}
        {screen === "import" && <ImportScreen bundle={bundle} clinical={clinical} />}
        {screen === "study" && <Study demo={demo} />}
        {screen === "settings" && <Settings status={status} prefs={prefs} setPrefs={setPrefs} onLocked={lock} refreshStatus={refreshStatus} />}
      </div>
      {newOpen && <NewDossier clinical={clinical} onClose={() => setNewOpen(false)} onCreated={(id) => { setNewOpen(false); void openEncounter(id); }} />}
    </div>
  );
}

function NewDossier({ clinical, onClose, onCreated }: { clinical: boolean; onClose: () => void; onCreated: (encounterId: string) => void }) {
  const [ln, setLn] = useState("");
  const [fn, setFn] = useState("");
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const create = async () => {
    setBusy(true); setErr(null);
    try {
      const identity = clinical && (ln.trim() || fn.trim()) ? { last_name: ln.trim() || null, first_name: fn.trim() || null, hospital_id: null } : null;
      const e = await call<EncounterFull>("create_dossier", { code: null, identity });
      onCreated(e.meta.id);
    } catch (e) { setErr(errMessage(e)); } finally { setBusy(false); }
  };
  return (
    <Modal title="Nouveau dossier" onClose={onClose} actions={<>
      <button type="button" className="btn" onClick={onClose}>Annuler</button>
      <button type="button" className="btn primary" disabled={busy} onClick={create}>Créer et commencer la consultation</button>
    </>}>
      <p className="small muted">Le code du dossier est attribué automatiquement. La consultation démarre à la date du jour (modifiable), sans aucune réponse clinique présélectionnée.</p>
      {clinical && (
        <div className="fields">
          <div className="field"><div className="label"><span>Nom</span></div><input value={ln} onChange={(e) => setLn(e.target.value)} autoFocus /></div>
          <div className="field"><div className="label"><span>Prénom</span></div><input value={fn} onChange={(e) => setFn(e.target.value)} /></div>
        </div>
      )}
      {err && <div className="banner error" style={{ marginTop: 12 }}>{err}</div>}
    </Modal>
  );
}
