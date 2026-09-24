// Statistiques descriptives : N affiché partout, aucune imputation, filtres identiques au tableau.
import { useEffect, useState } from "react";
import { call, errMessage } from "../api";
import type { Filter, Proportion, Stats as S } from "../types";
import { fmtNum, fmtPct, ORIGIN_LABELS, PROTOCOL_LABELS, SERVICE_LABELS, VOMITING_LABELS } from "../lib/format";
import { FilterBar } from "./Dossiers";

const MISSING: Record<string, string> = { not_asked: "non renseigné", not_recorded: "non consigné (ancien)", unknown: "inconnu", declined: "refus", not_assessable: "non évaluable", not_applicable: "non applicable", ambiguous_source: "source ambiguë" };

function Prop({ p }: { p: Proportion }) {
  if (p.n === 0) return <span className="muted">aucune donnée analysable</span>;
  return <span>{p.k}/{p.n} · {fmtPct(p.pct)} <span className="muted small">(IC 95 % {fmtPct(p.ci_low)}–{fmtPct(p.ci_high)})</span></span>;
}

function Counts({ title, data, labels, note }: { title: string; data: Record<string, number>; labels: Record<string, string>; note?: string }) {
  const total = Object.values(data).reduce((a, b) => a + b, 0);
  return (
    <div className="panel">
      <h3>{title}</h3>
      {note && <p className="small muted" style={{ marginTop: 4 }}>{note}</p>}
      {total === 0 ? <div className="empty-state">Aucune consultation concernée.</div> : (
        <div className="kv">{Object.entries(data).map(([k, v]) => [<span key={k}>{labels[k] ?? MISSING[k] ?? k}</span>, <span key={`${k}v`}>{v}/{total}</span>])}</div>
      )}
    </div>
  );
}

export function Stats({ filter, setFilter, demo }: { filter: Filter; setFilter: (f: Filter) => void; demo: boolean }) {
  const [s, setS] = useState<S | null>(null);
  const [err, setErr] = useState<string | null>(null);
  useEffect(() => {
    let alive = true;
    call<S>("stats", { filter }).then((x) => alive && (setS(x), setErr(null))).catch((e) => alive && setErr(errMessage(e)));
    return () => { alive = false; };
  }, [filter]);

  const b = s?.bewe;
  const d = b?.distribution;
  return (
    <div className="page">
      <div className="heading">
        <div><div className="kicker">Prendre du recul</div><h1>Vos observations.</h1>
          <p className="sub">Statistiques descriptives du recueil{demo ? " de démonstration (données fictives)" : ""}. Chaque proportion est rapportée à son propre dénominateur.</p></div>
      </div>
      <FilterBar filter={filter} setFilter={setFilter} withText={false} />
      {err && <div className="banner error">{err}</div>}
      {s && (
        <>
          <p className="small muted" style={{ margin: "0 2px 10px" }}>Filtres appliqués : {s.filters.length ? s.filters.join(" · ") : "aucun"}. Ces chiffres décrivent la base filtrée ; l'analyse d'une étude (une visite index par patient) se fait dans l'espace Étude.</p>
          <div className="metrics">
            <div className="metric">Consultations<strong>{s.n_visits}</strong>{s.n_legacy} historiques · {s.n_prospective} prospectives</div>
            <div className="metric">Dossiers (patients)<strong>{s.n_patients}</strong>Distinct du nombre de consultations</div>
            <div className="metric">BEWE analysable<strong>{b!.n_analysable}/{b!.n_visits}</strong>{Object.entries(b!.origins).map(([k, v]) => `${v} ${ORIGIN_LABELS[k]}`).join(" · ") || "—"}</div>
            <div className="metric">BEWE médian [Q1–Q3]<strong>{d!.n ? `${fmtNum(d!.median)}` : "—"}</strong>{d!.n ? `[${fmtNum(d!.q1)}–${fmtNum(d!.q3)}] · étendue ${fmtNum(d!.min)}–${fmtNum(d!.max)}` : "aucune donnée analysable"}</div>
          </div>

          <div className="panel">
            <h2>Distribution des scores BEWE</h2>
            <p className="description">Classes de référence (0–2, 3–8, 9–13, 14–18), sur les seuls scores disponibles (N = {b!.n_analysable}). Les classes décrivent les lésions ; elles ne constituent pas un pronostic individuel.</p>
            {b!.n_analysable === 0 ? <div className="empty-state">Aucune donnée analysable pour ces critères.</div> : (
              <div className="bars">
                {b!.categories.map(([c, n]) => (
                  <div key={c} className="bar">
                    <span>{c.replace("-", "–")}</span>
                    <div className="track" role="img" aria-label={`${n} sur ${b!.n_analysable}`}><div className="fill" style={{ width: `${(100 * n) / b!.n_analysable}%` }} /></div>
                    <span>{n}/{b!.n_analysable} · {fmtPct((100 * n) / b!.n_analysable)}</span>
                  </div>
                ))}
              </div>
            )}
            <div className="kv" style={{ marginTop: 18 }}>
              <span>BEWE &gt; 0</span><Prop p={b!.gt0} />
              <span>BEWE ≥ 9</span><Prop p={b!.ge9} />
              <span>BEWE ≥ 14</span><Prop p={b!.ge14} />
              <span>Moyenne ± écart-type</span><span>{d!.n ? `${fmtNum(d!.mean, 2)} ± ${fmtNum(d!.sd, 2)}` : "—"}</span>
            </div>
          </div>

          <div className="two-col" style={{ marginTop: 18 }}>
            <div className="panel">
              <h3>Âge</h3>
              <div className="kv">
                <span>Âge exact disponible</span><span>{s.age_exact.n}/{s.n_visits}</span>
                <span>Médiane [Q1–Q3]</span><span>{s.age_exact.n ? `${fmtNum(s.age_exact.median)} [${fmtNum(s.age_exact.q1)}–${fmtNum(s.age_exact.q3)}]` : "—"}</span>
                <span>Classe d'âge seule (non convertie)</span><span>{s.age_band_only}</span>
                <span>Âge manquant</span><span>{s.age_missing}</span>
              </div>
            </div>
            <Counts title="Services" data={s.services} labels={{ ...SERVICE_LABELS, non_renseigne: "non renseigné" }} />
            <Counts title="Vomissements — recueil structuré" data={s.vomiting_prospective} labels={VOMITING_LABELS} note="Jamais / anciens / actuels selon l'histoire explicitement recueillie." />
            <Counts title="Vomissements — ancien codage oui/non" data={s.vomiting_legacy} labels={VOMITING_LABELS} note="Temporalité inconnue : « non » ne signifie pas « jamais »." />
            <Counts title="Protocole de prévention — recueil structuré" data={s.protocol_prospective} labels={PROTOCOL_LABELS} />
            <Counts title="Prévention — anciens libellés" data={s.prevention_legacy} labels={PROTOCOL_LABELS} note="Libellés conservés ; composantes inconnues." />
          </div>

          <div className="panel" style={{ marginTop: 18 }}>
            <h3>Enseignement HBD (recueil structuré)</h3>
            <div className="kv"><span>Réalisé</span><Prop p={s.hbd_done} /></div>
          </div>

          <div className="panel" style={{ marginTop: 18 }}>
            <h2>Complétude des éléments essentiels</h2>
            <p className="description">Disponible / N, puis raisons de manque. Un manque n'est jamais compté comme « non ».</p>
            <div className="tablewrap" style={{ boxShadow: "none", marginTop: 12 }}>
              <table>
                <thead><tr><th>Variable</th><th>Disponible</th><th>Manques par raison</th></tr></thead>
                <tbody>{s.completeness.map((c) => (
                  <tr key={c.field}><td>{c.label}</td><td>{c.available}/{c.n}</td><td className="small">{Object.entries(c.missing).map(([k, v]) => `${MISSING[k] ?? k} : ${v}`).join(" · ") || "—"}</td></tr>
                ))}</tbody>
              </table>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
