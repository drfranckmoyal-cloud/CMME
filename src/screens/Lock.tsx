// Premier lancement, déverrouillage, restauration sur un nouveau Mac.
import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { call, errMessage } from "../api";
import type { Status } from "../types";

export function LockScreen({ status, onUnlocked }: { status: Status; onUnlocked: () => void }) {
  const [mode, setMode] = useState<"unlock" | "setup" | "restore">(status.clinique_exists ? "unlock" : "setup");
  const [pw, setPw] = useState("");
  const [pw2, setPw2] = useState("");
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [file, setFile] = useState<string | null>(null);
  const [phrase, setPhrase] = useState("");

  // Mot de passe désactivé par le praticien : ouverture directe de l'espace clinique.
  useEffect(() => {
    if (status.clinique_exists && !status.password_required) void call("unlock", { profile: "clinique" }).then(onUnlocked).catch((e) => setErr(errMessage(e)));
  }, [status.clinique_exists, status.password_required, onUnlocked]);

  const go = async (fn: () => Promise<void>) => {
    setBusy(true); setErr(null);
    try { await fn(); } catch (e) { setErr(errMessage(e)); } finally { setBusy(false); }
  };

  return (
    <div className="lock-screen">
      <div className="lock-card">
        <div className="brand">cmme<b>.</b><small>Recueil dentaire · Sainte-Anne</small></div>

        {mode === "unlock" && (
          <>
            <h2>Déverrouiller</h2>
            <p className="sub">Espace clinique chiffré sur ce Mac.</p>
            <form onSubmit={(e) => { e.preventDefault(); void go(async () => { await call("unlock", { profile: "clinique", password: pw }); setPw(""); onUnlocked(); }); }}>
              <input type="password" autoFocus value={pw} onChange={(e) => setPw(e.target.value)} placeholder="Mot de passe" aria-label="Mot de passe" autoComplete="current-password" />
              <button className="btn primary" type="submit" disabled={busy || !pw}>Déverrouiller</button>
            </form>
          </>
        )}

        {mode === "setup" && (
          <>
            <h2>Créer l'espace clinique</h2>
            <p className="sub">Une base chiffrée est créée sur ce Mac ; sa clé est rangée dans le trousseau macOS. Le mot de passe protège l'ouverture de l'application.</p>
            <form onSubmit={(e) => {
              e.preventDefault();
              if (pw !== pw2) { setErr("Les deux mots de passe diffèrent."); return; }
              void go(async () => { await call("setup_clinique", { practitioner: name, password: pw }); onUnlocked(); });
            }}>
              <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Nom du praticien (ex. Dr F. Moyal)" aria-label="Nom du praticien" />
              <input type="password" value={pw} onChange={(e) => setPw(e.target.value)} placeholder="Mot de passe (8 caractères au moins)" aria-label="Mot de passe" autoComplete="new-password" />
              <input type="password" value={pw2} onChange={(e) => setPw2(e.target.value)} placeholder="Confirmer le mot de passe" aria-label="Confirmer le mot de passe" autoComplete="new-password" />
              <button className="btn primary" type="submit" disabled={busy || !name || pw.length < 8}>Créer l'espace clinique</button>
            </form>
            <p className="small muted" style={{ marginTop: 10 }}>Aucune donnée ne quitte ce Mac. Pensez ensuite à faire une première sauvegarde chiffrée.</p>
          </>
        )}

        {mode === "restore" && (
          <>
            <h2>Restaurer une sauvegarde</h2>
            <p className="sub">Nouveau Mac ou poste réinstallé : la sauvegarde s'ouvre avec sa phrase de récupération, indépendamment du trousseau de l'ancien Mac.</p>
            <form onSubmit={(e) => { e.preventDefault(); void go(async () => {
              const meta = await call<{ created_at: string; counts: { patients: number; encounters: number } }>("restore_first_run", { path: file, passphrase: phrase });
              setInfo(`Restauration réussie : sauvegarde du ${new Date(meta.created_at).toLocaleString("fr-FR")}, ${meta.counts.patients} dossiers, ${meta.counts.encounters} consultations. Déverrouillez avec le mot de passe de l'application en vigueur à cette date.`);
              setPhrase(""); setMode("unlock");
            }); }}>
              <button type="button" className="btn" onClick={async () => { const f = await open({ multiple: false, filters: [{ name: "Sauvegarde CMME", extensions: ["cmmebak"] }] }); if (typeof f === "string") setFile(f); }}>
                {file ? `Fichier : ${file.split("/").pop()}` : "Choisir le fichier de sauvegarde…"}
              </button>
              <input type="password" value={phrase} onChange={(e) => setPhrase(e.target.value)} placeholder="Phrase de récupération" aria-label="Phrase de récupération" />
              <button className="btn primary" type="submit" disabled={busy || !file || phrase.length < 10}>Restaurer</button>
            </form>
          </>
        )}

        {err && <div className="banner error" role="alert" style={{ marginTop: 14 }}>{err}</div>}
        {info && <div className="banner ok" role="status" style={{ marginTop: 14 }}>{info}</div>}

        <div className="divider">ou</div>
        <div className="row">
          {mode !== "unlock" && status.clinique_exists && <button type="button" className="btn small" onClick={() => setMode("unlock")}>Déverrouiller</button>}
          {mode !== "setup" && !status.clinique_exists && <button type="button" className="btn small" onClick={() => setMode("setup")}>Créer l'espace clinique</button>}
          {mode !== "restore" && !status.clinique_exists && <button type="button" className="btn small" onClick={() => setMode("restore")}>Restaurer une sauvegarde</button>}
          <button type="button" className="btn small" onClick={() => void go(async () => { await call("unlock", { profile: "demo" }); onUnlocked(); })}>Ouvrir la démonstration</button>
        </div>
        <p className="small muted" style={{ marginTop: 14 }}>Version {status.app_version} · stockage : {status.keychain}{status.macos_version ? ` · macOS ${status.macos_version}` : ""}</p>
      </div>
    </div>
  );
}
