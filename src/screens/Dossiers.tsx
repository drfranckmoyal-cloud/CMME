// Tableau des consultations (une ligne par consultation) et filtres partagés avec les statistiques.
import { useEffect, useMemo, useState } from "react";
import { call, errMessage } from "../api";
import type { EncounterRow, Filter } from "../types";
import { fmtClinicalDate, ORIGIN_LABELS, PROTOCOL_LABELS, SERVICE_LABELS, STATUS_LABELS, VOMITING_LABELS } from "../lib/format";

const COLUMNS = [
  ["code", "Patient"], ["date", "Date"], ["service", "Service"], ["age", "Âge"], ["vomiting", "Vomissements"],
  ["bewe", "BEWE"], ["prevention", "Prévention"], ["mode", "Recueil"], ["status", "Statut / qualité"],
] as const;
type Col = (typeof COLUMNS)[number][0];

const loadCols = (): Col[] => {
  try { const v = JSON.parse(localStorage.getItem("cmme.columns") ?? "null"); if (Array.isArray(v)) return v; } catch { /* défaut */ }
  return COLUMNS.map((c) => c[0]);
};

export function FilterBar({ filter, setFilter, withText = true }: { filter: Filter; setFilter: (f: Filter) => void; withText?: boolean }) {
  const set = (patch: Partial<Filter>) => setFilter({ ...filter, ...patch });
  return (
    <div className="toolbar" role="search">
      {withText && (
        <div className="field search"><div className="label"><span>Rechercher (code, nom en espace clinique)</span></div>
          <input value={filter.text ?? ""} onChange={(e) => set({ text: e.target.value })} placeholder="Nom, prénom ou code…" /></div>
      )}
      <div className="field"><div className="label"><span>Du</span></div><input type="date" value={filter.date_from ?? ""} onChange={(e) => set({ date_from: e.target.value || undefined })} /></div>
      <div className="field"><div className="label"><span>Au</span></div><input type="date" value={filter.date_to ?? ""} onChange={(e) => set({ date_to: e.target.value || undefined })} /></div>
      <div className="field"><div className="label"><span>Service</span></div>
        <select value={filter.services?.[0] ?? ""} onChange={(e) => set({ services: e.target.value ? [e.target.value] : undefined })}>
          <option value="">Tous</option>{Object.entries(SERVICE_LABELS).map(([k, l]) => <option key={k} value={k}>{l}</option>)}
        </select></div>
      <div className="field"><div className="label"><span>Recueil</span></div>
        <select value={filter.collection_mode ?? ""} onChange={(e) => set({ collection_mode: e.target.value || undefined })}>
          <option value="">Historique et prospectif</option><option value="legacy_retrospective">Historique (reprise)</option><option value="structured_prospective">Prospectif (structuré)</option>
        </select></div>
      <div className="field"><div className="label"><span>Statut</span></div>
        <select value={filter.status ?? ""} onChange={(e) => set({ status: e.target.value || undefined })}>
          <option value="">Tous</option><option value="draft">Brouillon</option><option value="validated">Validée</option><option value="amending">Correction en cours</option>
        </select></div>
      <div className="field"><div className="label"><span>BEWE</span></div>
        <select value={filter.bewe ?? ""} onChange={(e) => set({ bewe: e.target.value || undefined })}>
          <option value="">Tous</option><option value="available">Disponible</option><option value="missing">Manquant / incomplet</option><option value="ge9">≥ 9</option>
        </select></div>
      <label className="check"><input type="checkbox" checked={!!filter.anomalies_only} onChange={(e) => set({ anomalies_only: e.target.checked || undefined })} />Anomalies</label>
      <button type="button" className="btn ghost" onClick={() => setFilter({})}>Réinitialiser</button>
    </div>
  );
}

export function Dossiers({ filter, setFilter, clinical, onOpen, onNew, scrollKey }: { filter: Filter; setFilter: (f: Filter) => void; clinical: boolean; onOpen: (id: string) => void; onNew: () => void; scrollKey: { current: number } }) {
  const [rows, setRows] = useState<EncounterRow[] | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [cols, setCols] = useState<Col[]>(loadCols);
  const [showIdentity, setShowIdentityState] = useState(() => { try { return localStorage.getItem("cmme.showIdentity") !== "0"; } catch { return true; } });
  const setShowIdentity = (v: boolean) => { setShowIdentityState(v); try { localStorage.setItem("cmme.showIdentity", v ? "1" : "0"); } catch { /* sans conséquence */ } };
  const [sort, setSort] = useState<{ key: Col; dir: 1 | -1 }>({ key: "code", dir: 1 });

  useEffect(() => {
    let alive = true;
    call<EncounterRow[]>("list_encounters", { filter, withIdentity: clinical && showIdentity })
      .then((r) => { if (alive) { setRows(r); setErr(null); requestAnimationFrame(() => window.scrollTo({ top: scrollKey.current })); } })
      .catch((e) => alive && setErr(errMessage(e)));
    return () => { alive = false; };
  }, [filter, showIdentity, clinical, scrollKey]);

  useEffect(() => { try { localStorage.setItem("cmme.columns", JSON.stringify(cols)); } catch { /* sans conséquence */ } }, [cols]);

  const sorted = useMemo(() => {
    if (!rows) return [];
    const key = (r: EncounterRow): string | number => {
      switch (sort.key) {
        case "code": return (r.display_name ?? r.patient_code).toLocaleLowerCase("fr");
        case "date": return r.visit_date ?? "";
        case "service": return r.service_code ?? "";
        case "age": return r.age_years ?? r.age_band?.[0] ?? -1;
        case "bewe": return r.bewe.value ?? -1;
        default: return "";
      }
    };
    // Tri stable : on garde l'ordre d'origine en cas d'égalité.
    return rows.map((r, i) => [r, i] as const).sort((a, b) => {
      const x = key(a[0]), y = key(b[0]);
      return x < y ? -sort.dir : x > y ? sort.dir : a[1] - b[1];
    }).map((x) => x[0]);
  }, [rows, sort]);

  const patients = rows ? new Set(rows.map((r) => r.patient_id)).size : 0;
  const th = (c: Col, label: string) => cols.includes(c) && (
    <th key={c} aria-sort={sort.key === c ? (sort.dir === 1 ? "ascending" : "descending") : "none"}>
      <button type="button" onClick={() => setSort({ key: c, dir: sort.key === c ? (sort.dir === 1 ? -1 : 1) : 1 })}>{label}{sort.key === c ? (sort.dir === 1 ? " ↑" : " ↓") : ""}</button>
    </th>
  );
  const age = (r: EncounterRow) => r.age_years != null ? `${r.age_years} ans` : r.age_band ? `classe ${r.age_band[0] ?? "…"}–${r.age_band[1] ?? "…"}` : r.age_missing ? VOMITING_LABELS[r.age_missing] ?? "—" : "—";

  return (
    <div className="page">
      <div className="heading">
        <div><div className="kicker">Registre clinique</div><h1>Les consultations.</h1><p className="sub">Une ligne par consultation. Retrouver, compléter, explorer.</p></div>
        <button type="button" className="btn primary" onClick={onNew}>+ Nouveau dossier</button>
      </div>
      <FilterBar filter={filter} setFilter={setFilter} />
      <div className="count-line">
        <span>{rows ? `${rows.length} consultation${rows.length > 1 ? "s" : ""} · ${patients} dossier${patients > 1 ? "s" : ""}` : "Chargement…"}</span>
        <span className="row">
          {clinical && <label className="check small"><input type="checkbox" checked={showIdentity} onChange={(e) => setShowIdentity(e.target.checked)} />Afficher l'identité</label>}
          <details className="picks" style={{ fontSize: 13 }}>
            <summary>Colonnes</summary>
            <div className="options">{COLUMNS.map(([c, l]) => (
              <label key={c} className="check"><input type="checkbox" checked={cols.includes(c)} disabled={c === "code"} onChange={(e) => setCols(e.target.checked ? [...cols, c] : cols.filter((x) => x !== c))} />{l}</label>
            ))}</div>
          </details>
        </span>
      </div>
      {err && <div className="banner error">{err}</div>}
      {rows && rows.length === 0 ? (
        <div className="panel empty-state">Aucune consultation ne correspond à ces critères. <button type="button" className="link" onClick={() => setFilter({})}>Réinitialiser les filtres</button></div>
      ) : (
        <div className="tablewrap">
          <table>
            <thead><tr>{COLUMNS.map(([c, l]) => th(c, l))}</tr></thead>
            <tbody>
              {sorted.map((r) => (
                <tr key={r.encounter_id} className="clickable" tabIndex={0}
                  onClick={() => { scrollKey.current = window.scrollY; onOpen(r.encounter_id); }}
                  onKeyDown={(e) => { if (e.key === "Enter") { scrollKey.current = window.scrollY; onOpen(r.encounter_id); } }}>
                  {cols.includes("code") && <td>{r.display_name ? <><strong style={{ color: "var(--accent)" }}>{r.display_name}</strong><div className="small muted">{r.patient_code}</div></> : <strong style={{ color: "var(--accent)" }}>{r.patient_code}</strong>}</td>}
                  {cols.includes("date") && <td>{fmtClinicalDate(r.visit_date)}{r.visit_date_precision && r.visit_date_precision !== "day" ? <span className="origin">({r.visit_date_precision === "month" ? "mois" : "année"})</span> : null}</td>}
                  {cols.includes("service") && <td>{r.service_code ? SERVICE_LABELS[r.service_code] : <span className="muted">—</span>}</td>}
                  {cols.includes("age") && <td>{age(r)}</td>}
                  {cols.includes("vomiting") && <td>{r.vomiting ? VOMITING_LABELS[r.vomiting] ?? r.vomiting : <span className="muted">Non renseigné</span>}</td>}
                  {cols.includes("bewe") && <td><span className={`scorepill ${r.bewe.value == null ? "none" : ""}`}>{r.bewe.value ?? "—"}</span><span className="origin">{ORIGIN_LABELS[r.bewe.origin]}{r.bewe.origin === "incomplete" ? ` ${r.bewe.sextants_filled}/6` : ""}</span></td>}
                  {cols.includes("prevention") && <td>{r.protocol ? PROTOCOL_LABELS[r.protocol] : <span className="muted">Non renseigné</span>}{r.hbd === "done" ? " · HBD" : ""}</td>}
                  {cols.includes("mode") && <td>{r.collection_mode === "legacy_retrospective" ? "Historique" : "Prospectif"}</td>}
                  {cols.includes("status") && <td>{STATUS_LABELS[r.status]}{r.revision > 1 ? ` (rév. ${r.revision})` : ""}{r.open_flags > 0 ? <span className="chip warn" style={{ marginLeft: 6 }}>{r.open_flags} anomalie{r.open_flags > 1 ? "s" : ""}</span> : null}{r.is_demo ? <span className="chip muted" style={{ marginLeft: 6 }}>fictif</span> : null}</td>}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
