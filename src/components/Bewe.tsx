// BEWE : petit odontogramme et six sextants anatomiques. Chaque score est lié à une clé permanente
// (jamais à une position d'écran). Un sextant non rempli (—) se distingue d'un 0.
import { useState } from "react";
import type { CatalogBundle, EncounterMeta, SextantRow } from "../types";

// Disposition à l'écran (contrat 02) : droite du patient à gauche de l'écran.
const UPPER = ["upper_right", "upper_anterior", "upper_left"];
const LOWER = ["lower_right", "lower_anterior", "lower_left"];

// Dents FDI de gauche à droite de l'écran, par arcade.
const UPPER_TEETH = [17, 16, 15, 14, 13, 12, 11, 21, 22, 23, 24, 25, 26, 27];
const LOWER_TEETH = [47, 46, 45, 44, 43, 42, 41, 31, 32, 33, 34, 35, 36, 37];

export function sextantOf(tooth: number): string {
  const q = Math.floor(tooth / 10), n = tooth % 10, ant = n <= 3;
  if (q === 1) return ant ? "upper_anterior" : "upper_right";
  if (q === 2) return ant ? "upper_anterior" : "upper_left";
  if (q === 3) return ant ? "lower_anterior" : "lower_left";
  return ant ? "lower_anterior" : "lower_right";
}

export function category(total: number): string {
  return total <= 2 ? "0–2" : total <= 8 ? "3–8" : total <= 13 ? "9–13" : "14–18";
}

interface Props {
  bundle: CatalogBundle;
  rows: SextantRow[];
  readOnly: boolean;
  onSet: (sextant: string, score: number | null, missing: string | null, reason: string | null) => void;
}

export function BeweChart({ bundle, rows, readOnly, onSet }: Props) {
  const [active, setActive] = useState<string | null>(null);
  const row = (k: string) => rows.find((r) => r.sextant === k);
  const cls = (k: string) => {
    const r = row(k);
    if (!r) return "empty";
    if (r.score !== null) return `s${r.score}`;
    return "na";
  };
  const focusTile = (k: string) => {
    setActive(k);
    document.getElementById(`sextant-${k}`)?.querySelector("button")?.focus();
  };
  const arch = (teeth: number[], y: number, upper: boolean) =>
    teeth.map((t, i) => {
      const x = 14 + i * 38;
      const k = sextantOf(t);
      const curve = Math.abs(i - 6.5) * Math.abs(i - 6.5) * 0.55;
      const yy = upper ? y + curve : y - curve;
      return (
        <g key={t} onClick={() => !readOnly && focusTile(k)}>
          <rect className={`tooth ${cls(k)} ${active === k ? "focus" : ""}`} x={x} y={yy} width={32} height={30} rx={9}>
            <title>{`Dent ${t} — ${bundle.sextants.find((s) => s.key === k)?.label}`}</title>
          </rect>
          <text x={x + 16} y={upper ? yy - 4 : yy + 42} textAnchor="middle">{t}</text>
        </g>
      );
    });

  const tile = (k: string, lower: boolean) => {
    const s = bundle.sextants.find((x) => x.key === k)!;
    const r = row(k);
    const onKey = (e: React.KeyboardEvent) => {
      if (readOnly) return;
      if (["0", "1", "2", "3"].includes(e.key)) { onSet(k, Number(e.key), null, null); e.preventDefault(); }
      else if (e.key.toLowerCase() === "n") { onSet(k, null, "not_assessable", null); e.preventDefault(); }
      else if (e.key === "Backspace" || e.key === "Delete") { onSet(k, null, null, null); e.preventDefault(); }
    };
    return (
      <div key={k} id={`sextant-${k}`} className={`sextant ${lower ? "lower" : ""} ${active === k ? "active" : ""}`} role="group"
        aria-label={`Sextant ${s.label}, dents ${s.teeth}`} onKeyDown={onKey} onFocus={() => setActive(k)}>
        <span className="sx-name">{s.label}</span>
        <span className="sx-teeth">{s.teeth}</span>
        <div className={`sx-score ${r?.score == null ? "empty" : ""}`} aria-live="polite">
          {r?.score != null ? r.score : r?.missing_reason === "not_assessable" ? "N.É." : "—"}
        </div>
        <div className="sx-btns">
          {[0, 1, 2, 3].map((n) => (
            <button key={n} type="button" disabled={readOnly} aria-pressed={r?.score === n} aria-label={`Score ${n} pour ${s.label}`}
              onClick={() => onSet(k, r?.score === n ? null : n, null, null)}>{n}</button>
          ))}
          <button type="button" className="na" disabled={readOnly} aria-pressed={r?.missing_reason === "not_assessable"} title="Non évaluable"
            onClick={() => onSet(k, null, r?.missing_reason === "not_assessable" ? null : "not_assessable", null)}>N.É.</button>
        </div>
        {r?.missing_reason === "not_assessable" && (
          <select value={r.unassessable_reason ?? ""} disabled={readOnly} aria-label={`Motif de non-évaluation, ${s.label}`}
            onChange={(e) => onSet(k, null, "not_assessable", e.target.value || null)}>
            <option value="">Motif non précisé</option>
            {bundle.unassessable_reasons.map((u) => <option key={u.code} value={u.code}>{u.label}</option>)}
          </select>
        )}
      </div>
    );
  };

  return (
    <div className="bewe">
      <div className="arch-labels"><span>← Droite du patient</span><span>Gauche du patient →</span></div>
      <svg className="odonto" viewBox="0 0 560 170" role="img" aria-label="Schéma des deux arcades ; cliquer sur une dent sélectionne son sextant">
        <defs>
          <pattern id="hatch" width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
            <rect width="6" height="6" fill="var(--panel)" /><line x1="0" y1="0" x2="0" y2="6" stroke="var(--muted)" strokeWidth="1.5" />
          </pattern>
        </defs>
        {arch(UPPER_TEETH, 14, true)}
        {arch(LOWER_TEETH, 126, false)}
      </svg>
      <div className="sextants">
        {UPPER.map((k) => tile(k, false))}
        {LOWER.map((k) => tile(k, true))}
      </div>
    </div>
  );
}

export function BeweSummary({ rows, meta }: { rows: SextantRow[]; meta: EncounterMeta }) {
  const filled = rows.length;
  const total = meta.bewe_total_derived;
  const hasNA = rows.some((r) => r.missing_reason);
  return (
    <div className="scorecard" aria-live="polite">
      <div className="scorelabel">Score BEWE</div>
      {total !== null ? (
        <>
          <div className="total">{total}<small>/18</small></div>
          <span className="scoreclass">Classe {category(total)}</span>
        </>
      ) : (
        <div className="total none" title="Total disponible seulement quand les six sextants sont scorés">—<small>/18</small></div>
      )}
      <div className="scorecaption">
        {filled}/6 sextants renseignés
        {total === null && filled === 6 && hasNA && <><br />Sextant non évaluable : pas de total</>}
        {total === null && filled < 6 && <><br />Total affiché quand les six sont scorés</>}
      </div>
      {meta.bewe_total_historical !== null && (
        <div className="scorecaption">Total historique : <strong>{meta.bewe_total_historical}</strong>
          {total !== null && total !== meta.bewe_total_historical && <><br /><span style={{ color: "var(--danger)" }}>Divergence à arbitrer</span></>}
        </div>
      )}
    </div>
  );
}
