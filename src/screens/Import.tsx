// Reprise de l'historique : aperçu → correspondance des colonnes → anomalies → validation → bilan.
// La source brute reste consultable ; chaque décision est tracée et réversible avant validation.
import { useCallback, useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { call, errMessage } from "../api";
import type { CatalogBundle } from "../types";
import { fmtDateTime } from "../lib/format";

interface SheetPreview { name: string; n_rows: number; header_row: number; headers: string[]; sample: string[][]; suggested_kind: string; suggested_mapping: string[]; merged_cells: number }
interface Preview { filename: string; sha256: string; format: string; size_bytes: number; already_imported: { imported_at: string; filename: string } | null; other_documents: number; sheets: SheetPreview[] }
interface SheetConfig { name: string; kind: string; header_row: number; mapping: string[]; service_from_sheet: boolean }
interface Doc { id: string; filename: string; sha256: string; format: string; imported_at: string; size_bytes: number; counts: Record<string, number>; needs_review: number }
interface Proposal { target: string; label: string; proposed: unknown; rule: string; decision: string; decided_value: unknown; justification: string | null }
interface Rec {
  id: string; sheet: string; row_number: number; cells: [string, string][]; status: string; exclusion_reason: string | null;
  flags: [string, string, boolean][]; unresolved: string[]; proposals: Proposal[]; diff: { same_file?: { record_id: string; sheet: string }[]; existing?: { patient_id: string; code: string }[]; changes?: { column: string; old: string | null; new: string }[] } | null;
  link_decision: string | null; link_patient_id: string | null; encounter_id: string | null;
}

const EXCLUSION: Record<string, string> = { repeated_header: "ligne d'en-tête répétée", separator_row: "ligne séparatrice (date ou titre seul) — non propagée" };
const RULES: Record<string, string> = {
  legacy_bewe_grammar: "grammaire BEWE : nombre entre parenthèses prioritaire", age_exact: "âge numérique", age_band_bounds: "bornes de la classe", age_band_below: "classe « < N »", age_band_above: "classe « > N »",
  age_exact_in_parentheses: "âge exact entre parenthèses", legacy_yes_no: "oui/non de l'ancienne colonne", legacy_prevention_label: "libellé ancien, composantes inconnues",
  raw_kept: "texte source conservé", raw_code: "code source", raw_identity: "identité (espace clinique, jamais exportée)", date_parsed: "date lue", legacy_service_label: "libellé de service",
  service_from_sheet_name: "nom de la feuille (proposition)", legacy_hygiene_label: "libellé d'hygiène", legacy_care_label: "libellé de soins", legacy_diet_label: "codage alimentaire", legacy_sex_code: "code sexe",
  legacy_dmft_numeric: "CAO numérique", no_exact_total: "pas de total exact", no_exact_age: "pas d'âge exact", unrecognized: "valeur non reconnue",
};

const show = (v: unknown): string => v === null || v === undefined ? "—" : Array.isArray(v) ? v.join("–") : String(v);

export function ImportScreen({ bundle, clinical }: { bundle: CatalogBundle; clinical: boolean }) {
  const [tab, setTab] = useState<"docs" | "wizard" | "review" | "duplicates">("docs");
  const [docs, setDocs] = useState<Doc[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const [path, setPath] = useState<string | null>(null);
  const [preview, setPreview] = useState<Preview | null>(null);
  const [config, setConfig] = useState<SheetConfig[]>([]);
  const [docId, setDocId] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const loadDocs = useCallback(() => call<Doc[]>("import_documents").then(setDocs).catch((e) => setErr(errMessage(e))), []);
  useEffect(() => { void loadDocs(); }, [loadDocs]);

  const choose = async () => {
    setErr(null); setNotice(null);
    const f = await open({ multiple: false, filters: [{ name: "Classeur ou CSV", extensions: ["xlsx", "xlsm", "xls", "ods", "csv"] }] });
    if (typeof f !== "string") return;
    try {
      const p = await call<Preview>("import_preview", { path: f });
      setPath(f); setPreview(p);
      setConfig(p.sheets.map((s) => ({ name: s.name, kind: s.suggested_kind, header_row: s.header_row, mapping: s.suggested_mapping, service_from_sheet: p.format !== "csv" })));
      setTab("wizard");
    } catch (e) { setErr(errMessage(e)); }
  };

  const stage = async () => {
    try {
      const sum = await call<{ document_id: string; rows_read: number; pending: number; ready: number; needs_review: number; unchanged: number; excluded: Record<string, number>; summary_sheets: string[] }>("import_stage", { path, config });
      setNotice(`Import préparé : ${sum.rows_read} lignes lues · ${sum.ready} sans anomalie · ${sum.needs_review} à revoir · ${sum.unchanged} déjà présentes (version précédente) · ${Object.values(sum.excluded).reduce((a, b) => a + b, 0)} exclues d'office${sum.summary_sheets.length ? ` · feuilles de synthèse ignorées : ${sum.summary_sheets.join(", ")}` : ""}. Rien n'est encore validé.`);
      setDocId(sum.document_id); setPreview(null); await loadDocs(); setTab("review");
    } catch (e) { setErr(errMessage(e)); }
  };

  return (
    <div className="page">
      <div className="heading">
        <div><div className="kicker">Reprise des anciennes données</div><h1>L'historique, sans rien inventer.</h1>
          <p className="sub">XLSX ou CSV exportés de Numbers. La source reste intacte et chiffrée ; chaque valeur proposée se valide ou se corrige avec justification.</p></div>
        <div className="row">
          <button type="button" className="btn" onClick={() => setTab("duplicates")}>Doublons et fusions</button>
          <button type="button" className="btn primary" onClick={choose}>Importer un fichier…</button>
        </div>
      </div>
      {!clinical && <div className="banner warn">Profil de démonstration : importez uniquement des fichiers fictifs (par exemple specs/data/import_synthetique.csv).</div>}
      {err && <div className="banner error" role="alert"><span className="grow">{err}</span><button className="link" type="button" onClick={() => setErr(null)}>Fermer</button></div>}
      {notice && <div className="banner ok" role="status"><span className="grow">{notice}</span><button className="link" type="button" onClick={() => setNotice(null)}>Fermer</button></div>}

      {tab === "docs" && (
        docs.length === 0 ? (
          <div className="panel empty-state">
            Aucun fichier importé. Exportez d'abord le classeur Numbers en XLSX (Fichier › Exporter vers › Excel), conservez l'original, puis importez l'export ici.
          </div>
        ) : (
          <div className="tablewrap">
            <table>
              <thead><tr><th>Fichier</th><th>Importé le</th><th>Empreinte</th><th>En attente</th><th>À revoir</th><th>Validées</th><th>Inchangées</th><th>Exclues</th><th /></tr></thead>
              <tbody>{docs.map((d) => (
                <tr key={d.id}>
                  <td>{d.filename}</td><td>{fmtDateTime(d.imported_at)}</td><td className="small muted">{d.sha256.slice(0, 12)}…</td>
                  <td>{d.counts.pending ?? 0}</td><td>{d.needs_review}</td><td>{d.counts.validated ?? 0}</td><td>{d.counts.unchanged ?? 0}</td><td>{d.counts.excluded ?? 0}</td>
                  <td><button type="button" className="btn small" onClick={() => { setDocId(d.id); setTab("review"); }}>Revoir</button></td>
                </tr>
              ))}</tbody>
            </table>
          </div>
        )
      )}

      {tab === "wizard" && preview && (
        <Wizard bundle={bundle} preview={preview} config={config} setConfig={setConfig} onCancel={() => { setPreview(null); setTab("docs"); }} onStage={stage} />
      )}
      {tab === "review" && docId && <Review docId={docId} clinical={clinical} onChanged={loadDocs} onBack={() => setTab("docs")} />}
      {tab === "duplicates" && <Duplicates onBack={() => setTab("docs")} />}
    </div>
  );
}

function Wizard({ bundle, preview, config, setConfig, onCancel, onStage }: { bundle: CatalogBundle; preview: Preview; config: SheetConfig[]; setConfig: (c: SheetConfig[]) => void; onCancel: () => void; onStage: () => void }) {
  const upd = (i: number, patch: Partial<SheetConfig>) => setConfig(config.map((c, j) => (j === i ? { ...c, ...patch } : c)));
  const individual = config.filter((c) => c.kind === "individual");
  return (
    <>
      <div className="panel">
        <h2>1. Aperçu de « {preview.filename} »</h2>
        <div className="kv">
          <span>Format</span><span>{preview.format.toUpperCase()} · {(preview.size_bytes / 1024).toFixed(1)} Ko</span>
          <span>Empreinte SHA-256</span><span className="small">{preview.sha256}</span>
          <span>Feuilles</span><span>{preview.sheets.length}</span>
        </div>
        {preview.already_imported && <div className="banner error" style={{ marginTop: 12 }}>Ce fichier exact a déjà été importé le {fmtDateTime(preview.already_imported.imported_at)} : il sera refusé (aucune duplication).</div>}
        {!preview.already_imported && preview.other_documents > 0 && <div className="banner info" style={{ marginTop: 12 }}>D'autres fichiers ont déjà été importés : si celui-ci en est une nouvelle version, les lignes identiques seront reconnues et les lignes modifiées soumises à revue.</div>}
      </div>
      {preview.sheets.map((s, i) => (
        <div className="panel" key={s.name}>
          <div className="row">
            <h3 className="grow">Feuille « {s.name} » · {s.n_rows} lignes non vides{s.merged_cells ? ` · ${s.merged_cells} zones fusionnées (non propagées)` : ""}</h3>
            <select value={config[i].kind} onChange={(e) => upd(i, { kind: e.target.value })} style={{ width: "auto" }} aria-label={`Nature de la feuille ${s.name}`}>
              <option value="individual">Une ligne = une consultation</option>
              <option value="summary">Tableau de synthèse (jamais de patient)</option>
              <option value="ignore">Ignorer</option>
            </select>
          </div>
          {config[i].kind === "individual" && (
            <>
              <div className="inline-fields" style={{ marginTop: 10 }}>
                <label>Ligne d'en-tête n° <input type="number" min={1} value={config[i].header_row + 1} onChange={(e) => upd(i, { header_row: Math.max(0, Number(e.target.value) - 1) })} style={{ width: 80, minWidth: 80 }} /></label>
                {preview.format !== "csv" && <label className="check"><input type="checkbox" checked={config[i].service_from_sheet} onChange={(e) => upd(i, { service_from_sheet: e.target.checked })} />Proposer le nom de la feuille comme service si la colonne manque</label>}
              </div>
              <div className="tablewrap" style={{ marginTop: 12, boxShadow: "none", border: "1px solid var(--line)" }}>
                <table className="mapping-table">
                  <thead><tr>{s.headers.map((h, c) => <th key={c}>{h || `Colonne ${c + 1}`}</th>)}</tr>
                    <tr>{s.headers.map((_, c) => (
                      <td key={c}><select value={config[i].mapping[c] ?? "ignore"} aria-label={`Correspondance de la colonne ${c + 1}`} onChange={(e) => { const m = [...config[i].mapping]; m[c] = e.target.value; upd(i, { mapping: m }); }}>
                        {bundle.import_targets.map((t) => <option key={t.code} value={t.code}>{t.label}</option>)}
                      </select></td>
                    ))}</tr>
                  </thead>
                  <tbody>{s.sample.map((r, ri) => <tr key={ri}>{s.headers.map((_, c) => <td key={c} className="small">{r[c] ?? ""}</td>)}</tr>)}</tbody>
                </table>
              </div>
              <p className="small muted" style={{ marginTop: 8 }}>Aperçu des 8 premières lignes. La rubrique FDI de l'ancien recueil n'est pas reprise (elle reste dans la source brute).</p>
            </>
          )}
        </div>
      ))}
      <div className="footer-actions">
        <button type="button" className="btn" onClick={onCancel}>Annuler</button>
        <button type="button" className="btn primary" disabled={!!preview.already_imported || individual.length === 0} onClick={onStage}>Préparer l'import (rien n'est encore validé)</button>
      </div>
    </>
  );
}

function Review({ docId, clinical, onChanged, onBack }: { docId: string; clinical: boolean; onChanged: () => void; onBack: () => void }) {
  const [recs, setRecs] = useState<Rec[]>([]);
  const [filter, setFilter] = useState("review");
  const [sel, setSel] = useState<string | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);
  const [correct, setCorrect] = useState<{ target: string; value: string; just: string } | null>(null);
  const [exclReason, setExclReason] = useState("");

  const load = useCallback(async () => {
    try { setRecs(await call<Rec[]>("import_records", { documentId: docId })); } catch (e) { setErr(errMessage(e)); }
  }, [docId]);
  useEffect(() => { void load(); }, [load]);

  const list = useMemo(() => recs.filter((r) => {
    if (filter === "review") return r.status === "pending" && r.unresolved.length > 0;
    if (filter === "ready") return r.status === "pending" && r.unresolved.length === 0;
    if (filter.startsWith("flag:")) return r.flags.some((f) => f[0] === filter.slice(5));
    return filter === "all" || r.status === filter;
  }), [recs, filter]);
  const allFlags = useMemo(() => [...new Map(recs.flatMap((r) => r.flags.map((f) => [f[0], f[1]] as [string, string]))).entries()], [recs]);
  const rec = recs.find((r) => r.id === sel) ?? list[0];
  const ready = recs.filter((r) => r.status === "pending" && r.unresolved.length === 0 && !r.flags.some((f) => f[2]));

  const act = async (fn: () => Promise<unknown>, done?: string) => {
    setErr(null);
    try { await fn(); await load(); onChanged(); if (done) setMsg(done); } catch (e) { setErr(errMessage(e)); }
  };

  return (
    <>
      <div className="row" style={{ marginBottom: 12 }}>
        <button type="button" className="btn ghost" onClick={onBack}>← Fichiers importés</button>
        <span className="grow" />
        <button type="button" className="btn primary" disabled={ready.length === 0} onClick={() => act(async () => {
          const r = await call<{ encounters_created: number; left_pending: number }>("import_commit", { documentId: docId, recordIds: null });
          setMsg(`${r.encounters_created} consultation(s) historique(s) créée(s) par validation en lot (lignes sans anomalie, règles déterministes). ${r.left_pending} ligne(s) restent à revoir.`);
        })}>Valider les {ready.length} lignes sans anomalie</button>
      </div>
      {err && <div className="banner error" role="alert">{err}</div>}
      {msg && <div className="banner ok" role="status"><span className="grow">{msg}</span><button className="link" type="button" onClick={() => setMsg(null)}>Fermer</button></div>}
      <div className="split">
        <div className="panel" style={{ padding: 0 }}>
          <div style={{ padding: 12 }}>
            <select value={filter} onChange={(e) => { setFilter(e.target.value); setSel(null); }} aria-label="Filtrer les lignes">
              <option value="review">À revoir ({recs.filter((r) => r.status === "pending" && r.unresolved.length > 0).length})</option>
              <option value="ready">Prêtes ({recs.filter((r) => r.status === "pending" && r.unresolved.length === 0).length})</option>
              <option value="validated">Validées ({recs.filter((r) => r.status === "validated").length})</option>
              <option value="unchanged">Inchangées depuis la version précédente ({recs.filter((r) => r.status === "unchanged").length})</option>
              <option value="excluded">Exclues ({recs.filter((r) => r.status === "excluded").length})</option>
              <option value="all">Toutes ({recs.length})</option>
              <optgroup label="Par anomalie">{allFlags.map(([f, l]) => <option key={f} value={`flag:${f}`}>{l}</option>)}</optgroup>
            </select>
          </div>
          <div className="reclist">
            {list.length === 0 && <div className="empty-state">Aucune ligne.</div>}
            {list.map((r) => (
              <button key={r.id} type="button" className="rec" aria-current={rec?.id === r.id} onClick={() => { setSel(r.id); setCorrect(null); }}>
                <strong>{r.sheet} · ligne {r.row_number}</strong>
                <div className="small muted">{r.cells.slice(0, 3).map((c) => c[1]).filter(Boolean).join(" · ")}</div>
                <div className="chips" style={{ marginTop: 4 }}>{r.unresolved.length > 0 ? <span className="chip warn">{r.unresolved.length} à trancher</span> : r.status === "pending" ? <span className="chip ok">prête</span> : <span className="chip muted">{r.status === "validated" ? "validée" : r.status === "excluded" ? "exclue" : "inchangée"}</span>}</div>
              </button>
            ))}
          </div>
        </div>

        {rec ? (
          <div className="panel">
            <div className="row"><h2 className="grow">{rec.sheet} · ligne {rec.row_number}</h2>
              {rec.status === "pending" && <button type="button" className="btn primary" disabled={rec.unresolved.length > 0} onClick={() => act(() => call("import_commit", { documentId: docId, recordIds: [rec.id] }), "Ligne validée : consultation historique créée.")}>Valider cette ligne</button>}
            </div>
            {rec.status === "excluded" && <div className="banner warn" style={{ marginTop: 10 }}><span className="grow">Exclue : {EXCLUSION[rec.exclusion_reason ?? ""] ?? rec.exclusion_reason}</span><button type="button" className="btn small" onClick={() => act(() => call("import_reinclude", { recordId: rec.id }))}>Réintégrer</button></div>}
            {rec.status === "unchanged" && <div className="banner info" style={{ marginTop: 10 }}>Identique à la ligne déjà importée d'une version précédente : aucune nouvelle consultation.</div>}
            <div className="two-col" style={{ marginTop: 14 }}>
              <div>
                <h3>Source brute (immuable)</h3>
                <dl className="source-cells" style={{ marginTop: 8 }}>
                  {rec.cells.filter(([h, v]) => v !== "" || h).flatMap(([h, v], i) => [<dt key={`h${i}`}>{h}</dt>, <dd key={`v${i}`}>{v === "" ? <span className="muted">(vide)</span> : clinical || !/nom|prénom|prenom/i.test(h) ? v : "•••"}</dd>])}
                </dl>
                {rec.flags.length > 0 && (<><h3 style={{ marginTop: 14 }}>Anomalies</h3><div className="chips" style={{ marginTop: 6 }}>{rec.flags.map((f) => <span key={f[0]} className={`chip ${rec.unresolved.includes(f[0]) ? "warn" : f[2] ? "" : "muted"}`}>{f[1]}{f[2] ? "" : " (information)"}</span>)}</div></>)}
                {rec.diff?.changes && (<><h3 style={{ marginTop: 14 }}>Différences avec la version précédente</h3>
                  <ul className="recap-list">{rec.diff.changes.map((c, i) => <li key={i}>{c.column} : « {c.old ?? ""} » → « {c.new} »</li>)}</ul></>)}
              </div>
              <div>
                <h3>Propositions</h3>
                {rec.proposals.filter((p) => !["last_name", "first_name", "full_name", "hospital_id"].includes(p.target) || clinical).map((p) => {
                  const blocking = rec.unresolved.some((f) => (f === "band_mismatch" || f === "band_only" || f === "ambiguous_exact_value" || f === "out_of_range" || f === "unparseable" || f === "not_recorded") ? p.target === "bewe_total_historical" : false) || (p.proposed === null && p.decision === "pending");
                  return (
                    <div key={p.target} className={`proposal ${blocking && p.decision === "pending" ? "pending-blocking" : ""}`}>
                      <div className="row"><strong className="grow">{p.label}</strong><span className="chip muted">{p.decision === "pending" ? "en attente" : p.decision === "accepted" ? "acceptée" : p.decision === "corrected" ? "corrigée" : "indisponible"}</span></div>
                      <div>Proposé : <strong>{show(p.proposed)}</strong> <span className="small muted">— {RULES[p.rule] ?? p.rule}</span></div>
                      {p.decision === "corrected" && <div className="small">Valeur retenue : <strong>{show(p.decided_value)}</strong> — {p.justification}</div>}
                      {rec.status === "pending" && (
                        <div className="row">
                          <button type="button" className="btn small" disabled={p.proposed === null} onClick={() => act(() => call("import_decide", { recordId: rec.id, target: p.target, decision: "accepted" }))}>Accepter</button>
                          <button type="button" className="btn small" onClick={() => setCorrect({ target: p.target, value: "", just: "" })}>Corriger…</button>
                          <button type="button" className="btn small" onClick={() => act(() => call("import_decide", { recordId: rec.id, target: p.target, decision: "unavailable" }))}>Indisponible</button>
                          {p.decision !== "pending" && <button type="button" className="btn ghost small" onClick={() => act(() => call("import_decide", { recordId: rec.id, target: p.target, decision: "pending" }))}>Annuler la décision</button>}
                        </div>
                      )}
                      {correct?.target === p.target && (
                        <div className="inline-fields">
                          <input placeholder="Valeur" value={correct.value} onChange={(e) => setCorrect({ ...correct, value: e.target.value })} aria-label="Valeur corrigée" />
                          <input placeholder="Justification (obligatoire)" value={correct.just} onChange={(e) => setCorrect({ ...correct, just: e.target.value })} aria-label="Justification" style={{ flex: 1 }} />
                          <button type="button" className="btn small primary" onClick={() => act(async () => {
                            const n = Number(correct.value.replace(",", "."));
                            const value = correct.value.trim() !== "" && !Number.isNaN(n) && /^[\d.,\s]+$/.test(correct.value) ? n : correct.value;
                            await call("import_decide", { recordId: rec.id, target: p.target, decision: "corrected", value, justification: correct.just });
                            setCorrect(null);
                          })}>Enregistrer</button>
                        </div>
                      )}
                    </div>
                  );
                })}
                {rec.status === "pending" && (rec.flags.some((f) => ["duplicate_candidate", "duplicate_exact", "no_identity", "changed_since_previous_version"].includes(f[0]))) && (
                  <div className="proposal">
                    <strong>Rattachement du dossier</strong>
                    {rec.diff?.existing && <div className="small">Dossiers existants à identité proche : {rec.diff.existing.map((x) => x.code).join(", ")}</div>}
                    {rec.diff?.same_file && <div className="small">Autres lignes de ce fichier à identité identique : {rec.diff.same_file.length}</div>}
                    <div className="row">
                      {rec.diff?.existing?.map((x) => (
                        <button key={x.patient_id} type="button" className="btn small" aria-pressed={rec.link_patient_id === x.patient_id} onClick={() => act(() => call("import_link", { recordId: rec.id, decision: "same_patient", patientId: x.patient_id }))}>Même patient que {x.code}</button>
                      ))}
                      <button type="button" className="btn small" aria-pressed={rec.link_decision === "different_person"} onClick={() => act(() => call("import_link", { recordId: rec.id, decision: "different_person" }))}>Personne différente (homonyme)</button>
                      <button type="button" className="btn small" aria-pressed={rec.link_decision === "new"} onClick={() => act(() => call("import_link", { recordId: rec.id, decision: "new" }))}>Créer un dossier distinct</button>
                    </div>
                    {rec.link_decision && <div className="small">Décision : {rec.link_decision === "same_patient" ? "même patient" : rec.link_decision === "different_person" ? "personne différente" : "dossier distinct"}</div>}
                  </div>
                )}
                {rec.status === "pending" && (
                  <div className="inline-fields" style={{ marginTop: 14 }}>
                    <input placeholder="Motif d'exclusion (ex. copie d'un bloc)" value={exclReason} onChange={(e) => setExclReason(e.target.value)} aria-label="Motif d'exclusion" style={{ flex: 1 }} />
                    <button type="button" className="btn small danger" disabled={exclReason.trim().length < 3} onClick={() => act(async () => { await call("import_exclude", { recordId: rec.id, reason: exclReason }); setExclReason(""); })}>Exclure cette ligne</button>
                  </div>
                )}
              </div>
            </div>
          </div>
        ) : <div className="panel empty-state">Sélectionnez une ligne.</div>}
      </div>
    </>
  );
}

function Duplicates({ onBack }: { onBack: () => void }) {
  const [data, setData] = useState<{ merges: { id: string; source_patient_id: string; target_patient_id: string; reason: string; at: string; undone_at: string | null; moved: string[] }[]; candidates: [string, string][][] } | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [reason, setReason] = useState("");
  const load = useCallback(() => call<typeof data>("merges").then(setData).catch((e) => setErr(errMessage(e))), []);
  useEffect(() => { void load(); }, [load]);
  const act = async (fn: () => Promise<unknown>) => { setErr(null); try { await fn(); await load(); } catch (e) { setErr(errMessage(e)); } };
  return (
    <>
      <button type="button" className="btn ghost" onClick={onBack}>← Fichiers importés</button>
      {err && <div className="banner error">{err}</div>}
      <div className="panel" style={{ marginTop: 12 }}>
        <h2>Dossiers à identité identique</h2>
        <p className="description">Candidats au rapprochement repérés par identité normalisée. Aucune fusion n'est automatique ; une fusion reste annulable.</p>
        <div className="field" style={{ marginTop: 12 }}><div className="label"><span>Motif de fusion (obligatoire)</span></div><input value={reason} onChange={(e) => setReason(e.target.value)} placeholder="Ex. même identifiant hospitalier vérifié" /></div>
        {data?.candidates.length === 0 && <div className="empty-state">Aucun candidat.</div>}
        {data?.candidates.map((g, i) => (
          <div key={i} className="proposal">
            <div>{g.map((x) => x[1]).join(" · ")}</div>
            <div className="row">{g.slice(1).map((x) => (
              <button key={x[0]} type="button" className="btn small" disabled={reason.trim().length < 3} onClick={() => act(() => call("merge_patients", { source: x[0], target: g[0][0], reason }))}>Fusionner {x[1]} dans {g[0][1]}</button>
            ))}</div>
          </div>
        ))}
      </div>
      <div className="panel">
        <h2>Fusions effectuées</h2>
        {data?.merges.length === 0 && <div className="empty-state">Aucune fusion.</div>}
        {data?.merges.map((m) => (
          <div key={m.id} className="proposal">
            <div>{fmtDateTime(m.at)} — {m.moved.length} consultation(s) déplacée(s) — motif : {m.reason}</div>
            {m.undone_at ? <span className="chip muted">annulée le {fmtDateTime(m.undone_at)}</span> : <button type="button" className="btn small" onClick={() => act(() => call("undo_merge", { mergeId: m.id }))}>Annuler cette fusion</button>}
          </div>
        ))}
      </div>
    </>
  );
}
