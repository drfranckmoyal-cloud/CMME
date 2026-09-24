// Sauvegarde chiffrée, restauration, thème, verrouillage, mot de passe, informations techniques.
import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { call, errMessage } from "../api";
import type { Status } from "../types";
import { fmtDateTime } from "../lib/format";
import { confirmAsk } from "../lib/confirm";
import type { Prefs } from "../App";

export function Settings({ status, prefs, setPrefs, onLocked, refreshStatus }: { status: Status; prefs: Prefs; setPrefs: (p: Prefs) => void; onLocked: () => void; refreshStatus: () => void }) {
  const [dir, setDir] = useState<string | null>(null);
  const [p1, setP1] = useState("");
  const [p2, setP2] = useState("");
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState<{ kind: "ok" | "error" | "info"; text: string } | null>(null);
  const [rFile, setRFile] = useState<string | null>(null);
  const [rPhrase, setRPhrase] = useState("");
  const [rPw, setRPw] = useState("");
  const [rMeta, setRMeta] = useState<{ created_at: string; app_version: string; schema_version: number; counts: Record<string, number> } | null>(null);
  const [cur, setCur] = useState("");
  const [npw, setNpw] = useState("");
  const demo = status.profile === "demo";

  const go = async (fn: () => Promise<void>) => {
    setBusy(true); setMsg(null);
    try { await fn(); } catch (e) { setMsg({ kind: "error", text: errMessage(e) }); } finally { setBusy(false); }
  };

  return (
    <div className="page">
      <div className="heading"><div><div className="kicker">Continuité</div><h1>Sauvegarde et réglages.</h1>
        <p className="sub">Dernière sauvegarde réussie : {status.last_backup_at ? fmtDateTime(status.last_backup_at) : "aucune"}.</p></div></div>
      {msg && <div className={`banner ${msg.kind}`} role="status"><span className="grow">{msg.text}</span><button className="link" type="button" onClick={() => setMsg(null)}>Fermer</button></div>}

      <div className="two-col">
        <div className="panel">
          <h2>Créer une sauvegarde chiffrée</h2>
          <p className="description">Copie complète et cohérente (dossiers, consultations, sources importées, journal, exports figés), chiffrée par une phrase de récupération. Elle se restaure sur un autre Mac sans le trousseau de celui-ci. Vérifiée avant d'être annoncée.</p>
          <div className="fields">
            <div className="field wide"><div className="label"><span>Destination (clé USB ou disque autorisé ; iCloud refusé)</span></div>
              <button type="button" className="btn" onClick={async () => { const d = await open({ directory: true, title: "Destination de la sauvegarde" }); if (typeof d === "string") setDir(d); }}>{dir ?? "Choisir un dossier…"}</button></div>
            <div className="field"><div className="label"><span>Phrase de récupération (10 caractères au moins)</span></div><input type="password" value={p1} onChange={(e) => setP1(e.target.value)} autoComplete="new-password" /></div>
            <div className="field"><div className="label"><span>Confirmer la phrase</span></div><input type="password" value={p2} onChange={(e) => setP2(e.target.value)} autoComplete="new-password" /></div>
          </div>
          <p className="small muted" style={{ marginTop: 8 }}>Sans cette phrase, la sauvegarde est irrécupérable. Conservez-la hors de ce Mac (papier sous clé, gestionnaire de mots de passe).</p>
          <button type="button" className="btn primary" style={{ marginTop: 12 }} disabled={busy || !dir || p1.length < 10 || p1 !== p2}
            onClick={() => go(async () => {
              const r = await call<{ path: string; size_bytes: number; meta: { counts: Record<string, number> } }>("backup_create", { dir, passphrase: p1 });
              setP1(""); setP2(""); refreshStatus();
              setMsg({ kind: "ok", text: `Sauvegarde créée et relue avec succès : ${r.path} (${(r.size_bytes / 1024).toFixed(0)} Ko, ${r.meta.counts.patients} dossiers, ${r.meta.counts.encounters} consultations).` });
            })}>{busy ? "Sauvegarde…" : "Créer la sauvegarde"}</button>
          {p1 && p2 && p1 !== p2 && <p className="err-note">Les deux phrases diffèrent.</p>}
        </div>

        <div className="panel">
          <h2>Restaurer une sauvegarde</h2>
          {demo ? <p className="description">Disponible dans le profil clinique.</p> : (
            <>
              <p className="description">L'état actuel est d'abord copié (copie de sécurité chiffrée), puis remplacé en une seule opération. Mauvaise phrase ou fichier endommagé : rien n'est modifié.</p>
              <div className="fields">
                <div className="field wide"><button type="button" className="btn" onClick={async () => { const f = await open({ multiple: false, filters: [{ name: "Sauvegarde CMME", extensions: ["cmmebak"] }] }); if (typeof f === "string") { setRFile(f); setRMeta(null); } }}>{rFile ? rFile.split("/").pop() : "Choisir le fichier…"}</button></div>
                <div className="field"><div className="label"><span>Phrase de récupération</span></div><input type="password" value={rPhrase} onChange={(e) => { setRPhrase(e.target.value); setRMeta(null); }} /></div>
                <div className="field"><div className="label"><span>Mot de passe de l'application</span></div><input type="password" value={rPw} onChange={(e) => setRPw(e.target.value)} /></div>
              </div>
              <div className="row" style={{ marginTop: 12 }}>
                <button type="button" className="btn" disabled={busy || !rFile || rPhrase.length < 10} onClick={() => go(async () => setRMeta(await call("backup_inspect", { path: rFile, passphrase: rPhrase })))}>1. Vérifier la sauvegarde</button>
                <button type="button" className="btn primary" disabled={busy || !rMeta || !rPw} onClick={() => go(async () => {
                  if (!(await confirmAsk(`Remplacer toutes les données actuelles par la sauvegarde du ${fmtDateTime(rMeta!.created_at)} ? Une copie de sécurité de l'état actuel sera conservée.`, "Restaurer"))) return;
                  const r = await call<{ safety_copy: string }>("backup_restore", { path: rFile, passphrase: rPhrase, password: rPw });
                  setMsg({ kind: "ok", text: `Restauration effectuée. Copie de sécurité de l'état précédent : ${r.safety_copy}. L'application est verrouillée : déverrouillez avec le mot de passe en vigueur dans la sauvegarde.` });
                  onLocked();
                })}>2. Restaurer</button>
              </div>
              {rMeta && (
                <div className="kv" style={{ marginTop: 12 }}>
                  <span>Sauvegarde du</span><span>{fmtDateTime(rMeta.created_at)}</span>
                  <span>Version de l'application / schéma</span><span>{rMeta.app_version} / {rMeta.schema_version}</span>
                  <span>Dossiers · consultations</span><span>{rMeta.counts.patients} · {rMeta.counts.encounters}</span>
                  <span>Fichiers sources · exports figés</span><span>{rMeta.counts.source_documents} · {rMeta.counts.export_snapshots}</span>
                  <span>Intégrité</span><span>vérifiée</span>
                </div>
              )}
            </>
          )}
        </div>

        <div className="panel">
          <h2>Apparence</h2>
          <p className="description">Palette non définitivement choisie : Iris par défaut, Lagune disponible.</p>
          <div className="field" style={{ marginTop: 14 }}><div className="label"><span>Thème</span></div>
            <div className="seg">{[["iris", "Iris (prune, lilas)"], ["lagune", "Lagune (pétrole, ivoire)"]].map(([k, l]) => <button key={k} type="button" aria-pressed={prefs.theme === k} onClick={() => setPrefs({ ...prefs, theme: k as Prefs["theme"] })}>{l}</button>)}</div></div>
          <div className="field" style={{ marginTop: 14 }}><div className="label"><span>Mode</span></div>
            <div className="seg">{[["system", "Selon macOS"], ["light", "Clair"], ["dark", "Sombre"]].map(([k, l]) => <button key={k} type="button" aria-pressed={prefs.scheme === k} onClick={() => setPrefs({ ...prefs, scheme: k as Prefs["scheme"] })}>{l}</button>)}</div></div>
        </div>

        <div className="panel">
          <h2>Sécurité</h2>
          <div className="field" style={{ marginTop: 10 }}><div className="label"><span>Verrouillage après inactivité</span></div>
            <select value={prefs.lockMinutes} onChange={(e) => setPrefs({ ...prefs, lockMinutes: Number(e.target.value) })}>
              {[5, 10, 15, 30].map((m) => <option key={m} value={m}>{m} minutes</option>)}
            </select></div>
          <p className="small muted" style={{ marginTop: 6 }}>Verrouillage aussi à la sortie de veille. La saisie en cours est enregistrée avant verrouillage.</p>
          {!demo && (
            <div className="field" style={{ marginTop: 14 }}>
              <label className="check"><input type="checkbox" checked={status.password_required} disabled={busy || (status.password_required && !cur)}
                onChange={(e) => go(async () => { await call("set_password_required", { required: e.target.checked, password: cur }); refreshStatus(); setMsg({ kind: "ok", text: e.target.checked ? "Le mot de passe sera demandé à l'ouverture." : "Le mot de passe ne sera plus demandé. La base reste chiffrée ; toute personne ayant accès à votre session macOS peut ouvrir CMME." }); })} />
                Demander le mot de passe à l'ouverture</label>
              <p className="small muted">{status.password_required ? "Pour le désactiver, saisissez d'abord le mot de passe actuel ci-dessous." : "Désactivé : ouverture directe, sans verrouillage automatique. La base reste chiffrée sur le disque."}</p>
            </div>
          )}
          {!demo && (
            <div className="fields">
              <div className="field"><div className="label"><span>Mot de passe actuel</span></div><input type="password" value={cur} onChange={(e) => setCur(e.target.value)} /></div>
              <div className="field"><div className="label"><span>Nouveau mot de passe</span></div><input type="password" value={npw} onChange={(e) => setNpw(e.target.value)} /></div>
              <button type="button" className="btn" disabled={busy || !cur || npw.length < 8} onClick={() => go(async () => { await call("change_password", { current: cur, newPassword: npw }); setCur(""); setNpw(""); setMsg({ kind: "ok", text: "Mot de passe modifié." }); })}>Changer le mot de passe</button>
            </div>
          )}
        </div>

        <div className="panel">
          <h2>Informations techniques</h2>
          <div className="kv">
            <span>Version</span><span>{status.app_version}</span>
            <span>macOS</span><span>{status.macos_version ?? "—"}</span>
            <span>Chiffrement</span><span>SQLCipher {status.cipher_version ?? ""}</span>
            <span>Clé de la base</span><span>{status.keychain}</span>
            <span>Profil</span><span>{demo ? "Démonstration (base séparée, données fictives)" : "Clinique"}</span>
            <span>Données</span><span className="small">{status.data_dir}</span>
            <span>Réseau</span><span>aucun : ni cloud, ni télémétrie, ni IA</span>
          </div>
        </div>
      </div>
    </div>
  );
}
