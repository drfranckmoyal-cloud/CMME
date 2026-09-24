// Accueil : trois actions principales et quelques indicateurs de travail (pas de statistiques d'étude).
import { useEffect, useState } from "react";
import { call, errMessage } from "../api";
import type { EncounterRow } from "../types";
import { fmtClinicalDate, fmtDateTime, SERVICE_LABELS, STATUS_LABELS } from "../lib/format";

interface Summary { drafts: number; visits: number; patients: number; anomalies: number; import_pending: number; last_backup_at: string | null; recent: EncounterRow[] }

export function Home({ practitioner, demo, onNew, onSearch, onImport, onOpen, onBackup }: { practitioner: string | null; demo: boolean; onNew: () => void; onSearch: () => void; onImport: () => void; onOpen: (id: string) => void; onBackup: () => void }) {
  const [s, setS] = useState<Summary | null>(null);
  const [err, setErr] = useState<string | null>(null);
  useEffect(() => { call<Summary>("home_summary").then(setS).catch((e) => setErr(errMessage(e))); }, []);
  const backupOld = !s?.last_backup_at || Date.now() - new Date(s.last_backup_at).getTime() > 7 * 86400000;
  return (
    <div className="page">
      <div className="heading">
        <div><div className="kicker">{new Date().toLocaleDateString("fr-FR", { weekday: "long", day: "numeric", month: "long", year: "numeric" })}</div>
          <h1>Bonjour{practitioner ? `, ${practitioner}` : ""}.</h1>
          <p className="sub">{demo ? "Profil de démonstration : données fictives, base séparée." : "Espace clinique chiffré, local et hors ligne."}</p></div>
      </div>
      {err && <div className="banner error">{err}</div>}
      {!demo && s && backupOld && (
        <div className="banner warn"><span className="grow">{s.last_backup_at ? `Dernière sauvegarde : ${fmtDateTime(s.last_backup_at)}.` : "Aucune sauvegarde n'a encore été faite."} Une sauvegarde chiffrée régulière protège contre la perte du Mac.</span>
          <button type="button" className="btn small" onClick={onBackup}>Sauvegarder</button></div>
      )}
      <div className="metrics" style={{ gridTemplateColumns: "repeat(3, 1fr)" }}>
        <button type="button" className="metric" style={{ textAlign: "left" }} onClick={onNew}>Nouvelle consultation<strong>+</strong>Dossier vierge, aucune réponse présélectionnée</button>
        <button type="button" className="metric" style={{ textAlign: "left" }} onClick={onSearch}>Rechercher un patient<strong>{s?.patients ?? "…"}</strong>dossiers · {s?.visits ?? "…"} consultations</button>
        <button type="button" className="metric" style={{ textAlign: "left" }} onClick={onImport}>Reprendre les anciennes données<strong>{s?.import_pending ?? "…"}</strong>lignes importées en attente de revue</button>
      </div>
      <div className="two-col">
        <div className="panel">
          <h3>À reprendre</h3>
          <div className="kv">
            <span>Brouillons non terminés</span><span>{s?.drafts ?? "…"}</span>
            <span>Consultations avec anomalie</span><span>{s?.anomalies ?? "…"}</span>
            <span>Dernière sauvegarde réussie</span><span>{s?.last_backup_at ? fmtDateTime(s.last_backup_at) : "aucune"}</span>
          </div>
        </div>
        <div className="panel">
          <h3>Consultations récentes</h3>
          {s && s.recent.length === 0 && <div className="empty-state">Aucune consultation.</div>}
          {s?.recent.map((r) => (
            <div key={r.encounter_id} className="minirow">
              <button type="button" className="link" onClick={() => onOpen(r.encounter_id)}>{r.patient_code}</button>
              <span className="small">{fmtClinicalDate(r.visit_date)} · {r.service_code ? SERVICE_LABELS[r.service_code] : "—"} · {STATUS_LABELS[r.status]}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
