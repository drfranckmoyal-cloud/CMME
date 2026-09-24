// Espace étude : projet, visite index, exclusions tracées, export pseudonymisé figé.
import { useCallback, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { call, errMessage } from "../api";
import { fmtDateTime, fmtNum } from "../lib/format";

interface Project { id: string; name: string; period_start: string | null; period_end: string | null; include_legacy: boolean; include_prospective: boolean; exclude_open_anomalies: boolean; plan_version: string | null; created_at: string }
interface Selection { patients_total: number; encounters_total: number; excluded_encounters: Record<string, number>; excluded_patients: [string, string, string][]; included: { patient_id: string; patient_code: string; index_encounter: string; eligible_encounters: string[] }[]; bewe_analysable_index: number }
interface Snapshot { id: string; created_at: string; manifest: { rows: Record<string, number>; exact_dates: boolean; summary: { index_bewe_n: number; index_bewe_median: number | null; index_bewe_ge9: number } } }

const REASONS: Record<string, string> = {
  non_validee: "consultation non validée (brouillon ou correction en cours)", historique_hors_projet: "recueil historique hors projet", prospectif_hors_projet: "recueil prospectif hors projet",
  anomalie_ouverte: "anomalie ouverte non arbitrée", date_absente_periode_non_verifiable: "date absente : période non vérifiable", date_imprecise_periode_non_verifiable: "date imprécise en bord de période", hors_periode: "hors période",
};

export function Study({ demo }: { demo: boolean }) {
  const [projects, setProjects] = useState<Project[]>([]);
  const [pid, setPid] = useState<string | null>(null);
  const [sel, setSel] = useState<Selection | null>(null);
  const [snaps, setSnaps] = useState<Snapshot[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);
  const [form, setForm] = useState({ name: "", period_start: "", period_end: "", include_legacy: true, include_prospective: true, exclude_open_anomalies: true, plan_version: "" });
  const [exact, setExact] = useState(false);
  const [excl, setExcl] = useState<{ patient: string; reason: string } | null>(null);

  const loadProjects = useCallback(async () => {
    try { const p = await call<Project[]>("projects"); setProjects(p); if (!pid && p[0]) setPid(p[0].id); } catch (e) { setErr(errMessage(e)); }
  }, [pid]);
  const loadSel = useCallback(async () => {
    if (!pid) return;
    try { setSel(await call<Selection>("selection", { projectId: pid })); setSnaps(await call<Snapshot[]>("snapshots", { projectId: pid })); } catch (e) { setErr(errMessage(e)); }
  }, [pid]);
  useEffect(() => { void loadProjects(); }, [loadProjects]);
  useEffect(() => { void loadSel(); }, [loadSel]);

  const act = async (fn: () => Promise<unknown>) => { setErr(null); try { await fn(); } catch (e) { setErr(errMessage(e)); } };
  const project = projects.find((p) => p.id === pid);

  return (
    <div className="page">
      <div className="heading">
        <div><div className="kicker">Espace étude</div><h1>Préparer l'analyse.</h1>
          <p className="sub">Une visite index par patient, des exclusions comptées, un export figé et pseudonymisé. Aucun moteur de recherche de significativité.</p></div>
      </div>
      {demo && <div className="banner warn">Profil de démonstration : les exports contiennent des données fictives.</div>}
      {err && <div className="banner error" role="alert"><span className="grow">{err}</span><button className="link" type="button" onClick={() => setErr(null)}>Fermer</button></div>}
      {msg && <div className="banner ok" role="status"><span className="grow">{msg}</span><button className="link" type="button" onClick={() => setMsg(null)}>Fermer</button></div>}

      <div className="two-col">
        <div className="panel">
          <h2>Projets</h2>
          {projects.length === 0 && <p className="small muted" style={{ marginTop: 8 }}>Aucun projet. Créez-en un pour définir le périmètre de l'analyse.</p>}
          <div style={{ marginTop: 10 }}>
            {projects.map((p) => (
              <button key={p.id} type="button" className="rec" aria-current={p.id === pid} onClick={() => setPid(p.id)}>
                <strong>{p.name}</strong><div className="small muted">{p.period_start || p.period_end ? `${p.period_start ?? "…"} → ${p.period_end ?? "…"}` : "toutes périodes"} · {p.include_legacy ? "ancien + nouveau recueil" : "nouveau recueil uniquement"}{p.plan_version ? ` · plan ${p.plan_version}` : ""}</div>
              </button>
            ))}
          </div>
        </div>
        <div className="panel">
          <h2>Nouveau projet</h2>
          <form className="fields" onSubmit={(e) => { e.preventDefault(); void act(async () => { const p = await call<Project>("create_project", { input: { ...form, period_start: form.period_start || null, period_end: form.period_end || null, plan_version: form.plan_version || null } }); setPid(p.id); await loadProjects(); setForm({ ...form, name: "" }); }); }}>
            <div className="field wide"><div className="label"><span>Nom</span></div><input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder="Étude rétrospective BEWE" /></div>
            <div className="field"><div className="label"><span>Début de période (facultatif)</span></div><input type="date" value={form.period_start} onChange={(e) => setForm({ ...form, period_start: e.target.value })} /></div>
            <div className="field"><div className="label"><span>Fin de période (facultatif)</span></div><input type="date" value={form.period_end} onChange={(e) => setForm({ ...form, period_end: e.target.value })} /></div>
            <div className="field wide"><div className="label"><span>Données analysées</span></div>
              <div className="seg" role="group" aria-label="Données analysées">
                <button type="button" aria-pressed={form.include_legacy} onClick={() => setForm({ ...form, include_legacy: true, include_prospective: true })}>Ancien + nouveau recueil</button>
                <button type="button" aria-pressed={!form.include_legacy} onClick={() => setForm({ ...form, include_legacy: false, include_prospective: true })}>Nouveau recueil uniquement</button>
              </div>
            </div>
            <label className="check wide"><input type="checkbox" checked={form.exclude_open_anomalies} onChange={(e) => setForm({ ...form, exclude_open_anomalies: e.target.checked })} />Exclure les consultations avec une anomalie encore ouverte</label>
            <div className="field"><div className="label"><span>Version du plan d'analyse</span></div><input value={form.plan_version} onChange={(e) => setForm({ ...form, plan_version: e.target.value })} placeholder="ex. PLAN-2026-10-v1" /></div>
            <div className="field" style={{ alignSelf: "end" }}><button className="btn primary" type="submit" disabled={form.name.trim().length < 3}>Créer le projet</button></div>
          </form>
          <p className="small muted" style={{ marginTop: 10 }}>Avec une période définie, une consultation sans date ne peut pas être vérifiée : elle est exclue et comptée, jamais datée par supposition.</p>
        </div>
      </div>

      {project && sel && (
        <>
          <div className="panel" style={{ marginTop: 18 }}>
            <h2>Diagramme de sélection — {project.name}</h2>
            <div className="kv" style={{ maxWidth: 720 }}>
              <span>Dossiers dans la base</span><span>{sel.patients_total}</span>
              <span>Consultations dans la base</span><span>{sel.encounters_total}</span>
              {Object.entries(sel.excluded_encounters).map(([k, v]) => [<span key={k}>— consultations exclues : {REASONS[k] ?? k}</span>, <span key={`${k}v`}>{v}</span>])}
              {Object.entries(sel.excluded_patients.reduce<Record<string, number>>((m, x) => ((m[x[2]] = (m[x[2]] ?? 0) + 1), m), {})).map(([k, v]) => [<span key={k}>— dossiers exclus : {k}</span>, <span key={`${k}v`}>{v}</span>])}
              <span><strong>Dossiers inclus (une visite index chacun)</strong></span><span><strong>{sel.included.length}</strong></span>
              <span>BEWE analysable à la visite index</span><span>{sel.bewe_analysable_index}/{sel.included.length}</span>
            </div>
          </div>

          <div className="panel">
            <h2>Figer une extraction</h2>
            <p className="description">Fichiers : patients, visites, BEWE, expositions, actions de prévention, dictionnaire, manifeste (versions, règles, empreintes SHA-256), rapport de qualité. Liste blanche de colonnes : aucun nom, date de naissance, identifiant hospitalier, texte libre ni chemin de fichier. Pseudonymisé ne veut pas dire anonymisé.</p>
            <label className="check" style={{ marginTop: 10 }}><input type="checkbox" checked={exact} onChange={(e) => setExact(e.target.checked)} />Inclure les dates exactes (par défaut : année seulement ; à n'activer que si nécessaire et autorisé)</label>
            <div className="row" style={{ marginTop: 10 }}>
              <button type="button" className="btn primary" disabled={sel.included.length === 0} onClick={() => act(async () => { await call("freeze_export", { projectId: project.id, exactDates: exact }); setMsg("Extraction figée et conservée dans la base chiffrée. Écrivez-la dans un dossier de votre choix pour l'analyser."); await loadSel(); })}>Figer une extraction</button>
            </div>
            {snaps.length > 0 && (
              <div className="tablewrap" style={{ marginTop: 14, boxShadow: "none" }}>
                <table>
                  <thead><tr><th>Figée le</th><th>Lignes (patients / visites / BEWE)</th><th>BEWE index : N · médiane · ≥ 9</th><th>Dates exactes</th><th /></tr></thead>
                  <tbody>{snaps.map((s) => (
                    <tr key={s.id}>
                      <td>{fmtDateTime(s.created_at)}</td>
                      <td>{s.manifest.rows["patients.csv"]} / {s.manifest.rows["visits.csv"]} / {s.manifest.rows["bewe.csv"]}</td>
                      <td>{s.manifest.summary.index_bewe_n} · {fmtNum(s.manifest.summary.index_bewe_median)} · {s.manifest.summary.index_bewe_ge9}/{s.manifest.summary.index_bewe_n}</td>
                      <td>{s.manifest.exact_dates ? "oui" : "non (année)"}</td>
                      <td><button type="button" className="btn small" onClick={() => act(async () => {
                        const dir = await open({ directory: true, title: "Dossier de destination de l'export (hors iCloud)" });
                        if (typeof dir !== "string") return;
                        const out = await call<string>("write_snapshot", { snapshotId: s.id, dir });
                        setMsg(`Export écrit dans ${out}. Les fichiers CSV sont en clair : conservez-les dans un emplacement autorisé.`);
                      })}>Écrire dans un dossier…</button></td>
                    </tr>
                  ))}</tbody>
                </table>
              </div>
            )}
            <p className="small muted" style={{ marginTop: 10 }}>Recalcul indépendant : <code>python3 research/scripts/rapport_descriptif.py &lt;dossier d'export&gt;</code> retrouve N, médiane, classes et ≥ 9 à partir des seuls fichiers exportés, après vérification des empreintes.</p>
          </div>

          <div className="panel">
            <h2>Dossiers inclus</h2>
            <p className="description">Retirer l'éligibilité exclut le dossier des prochains exports de ce projet, sans rien effacer des soins ni des exports déjà figés.</p>
            <div className="tablewrap" style={{ marginTop: 12, boxShadow: "none", maxHeight: 420, overflowY: "auto" }}>
              <table>
                <thead><tr><th>Dossier</th><th>Consultations éligibles</th><th /></tr></thead>
                <tbody>
                  {sel.included.map((x) => (
                    <tr key={x.patient_id}><td>{x.patient_code}</td><td>{x.eligible_encounters.length}</td>
                      <td>{excl?.patient === x.patient_id ? (
                        <span className="inline-fields"><input autoFocus placeholder="Motif (ex. opposition)" value={excl.reason} onChange={(e) => setExcl({ ...excl, reason: e.target.value })} />
                          <button type="button" className="btn small" disabled={excl.reason.trim().length < 3} onClick={() => act(async () => { await call("set_eligibility", { projectId: project.id, patientId: x.patient_id, excludedReason: excl.reason }); setExcl(null); await loadSel(); })}>Confirmer</button></span>
                      ) : <button type="button" className="btn ghost small" onClick={() => setExcl({ patient: x.patient_id, reason: "" })}>Retirer l'éligibilité</button>}</td></tr>
                  ))}
                  {sel.excluded_patients.filter((x) => x[2].startsWith("éligibilité")).map((x) => (
                    <tr key={x[0]}><td className="muted">{x[1]}</td><td className="muted small">{x[2]}</td>
                      <td><button type="button" className="btn ghost small" onClick={() => act(async () => { await call("set_eligibility", { projectId: project.id, patientId: x[0], excludedReason: null }); await loadSel(); })}>Rendre éligible</button></td></tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
