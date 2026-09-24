// Boissons et aliments : deux menus à cases multiples, « Aucune rapportée » exclusif,
// précisions de fréquence (exacte ou plage) et de quantité, champ libre pour chacun.
import { useEffect, useRef, useState } from "react";
import type { ExposureRow, Opt, StoredValue } from "../types";
import { numToInput, parseFrNumber } from "../lib/format";
import { confirmAsk } from "../lib/confirm";

interface GroupProps {
  group: "drink" | "food";
  title: string;
  noneLabel: string;
  options: Opt[];
  status: StoredValue | undefined;
  rows: ExposureRow[];
  freeText: StoredValue | undefined;
  freeTextField: string;
  readOnly: boolean;
  onSelect: (group: string, none: boolean, categories: string[]) => void;
  onDetail: (group: string, category: string, patch: Record<string, unknown>) => void;
  onText: (field: string, value: string | null) => void;
}

function Detail({ row, label, readOnly, onDetail }: { row: ExposureRow; label: string; readOnly: boolean; onDetail: GroupProps["onDetail"] }) {
  const [min, setMin] = useState(numToInput(row.freq_min));
  const [max, setMax] = useState(numToInput(row.freq_max));
  const [qty, setQty] = useState(row.quantity_text ?? "");
  const [err, setErr] = useState<string | null>(null);
  const focused = useRef(false);
  useEffect(() => {
    if (focused.current) return;
    setMin(numToInput(row.freq_min)); setMax(numToInput(row.freq_max)); setQty(row.quantity_text ?? "");
  }, [row.freq_min, row.freq_max, row.quantity_text]);
  const commit = (over: Partial<{ min: string; max: string; qty: string; unit: string | null; temp: string | null }> = {}) => {
    const a = parseFrNumber(over.min ?? min), b = parseFrNumber(over.max ?? max);
    if ((a !== null && Number.isNaN(a)) || (b !== null && Number.isNaN(b))) { setErr("Fréquence non valide"); return; }
    if (b !== null && a === null) { setErr("Indiquez d'abord la borne basse"); return; }
    if (a !== null && b !== null && b < a) { setErr("Borne haute inférieure"); return; }
    setErr(null);
    onDetail(row.grp, row.category, {
      freq_min: a, freq_max: b, freq_unit: over.unit !== undefined ? over.unit : row.freq_unit,
      temporality: over.temp !== undefined ? over.temp : row.temporality, quantity_text: over.qty ?? qty,
    });
  };
  const bind = { onFocus: () => (focused.current = true), onBlur: () => { focused.current = false; commit(); } };
  return (
    <div className="expo-item">
      <div className="expo-head"><span>{label}</span></div>
      <div className="expo-fields">
        <input className="num" inputMode="decimal" placeholder="fréq." aria-label={`${label} : fréquence`} value={min} readOnly={readOnly} onChange={(e) => setMin(e.target.value)} {...bind} />
        <span>à</span>
        <input className="num" inputMode="decimal" placeholder="max" aria-label={`${label} : fréquence maximale`} value={max} readOnly={readOnly} onChange={(e) => setMax(e.target.value)} {...bind} />
        <select value={row.freq_unit ?? ""} disabled={readOnly} aria-label={`${label} : par`} onChange={(e) => commit({ unit: e.target.value || null })}>
          <option value="">par…</option><option value="day">par jour</option><option value="week">par semaine</option><option value="month">par mois</option>
        </select>
        <select value={row.temporality ?? ""} disabled={readOnly} aria-label={`${label} : actuel ou ancien`} onChange={(e) => commit({ temp: e.target.value || null })}>
          <option value="">actuel / ancien ?</option><option value="current">actuel</option><option value="past">ancien</option>
        </select>
        <input placeholder="quantité (ex. 1 canette)" aria-label={`${label} : quantité`} value={qty} readOnly={readOnly} onChange={(e) => setQty(e.target.value)} {...bind} style={{ flex: 1, minWidth: 140 }} />
      </div>
      {err && <span className="err-note">{err}</span>}
    </div>
  );
}

function FreeText({ field, value, readOnly, onText, placeholder }: { field: string; value: StoredValue | undefined; readOnly: boolean; onText: GroupProps["onText"]; placeholder: string }) {
  const [t, setT] = useState(value?.value_text ?? "");
  const focused = useRef(false);
  useEffect(() => { if (!focused.current) setT(value?.value_text ?? ""); }, [value?.value_text]);
  return <input value={t} readOnly={readOnly} placeholder={placeholder} aria-label={placeholder}
    onFocus={() => (focused.current = true)} onBlur={() => (focused.current = false)}
    onChange={(e) => { setT(e.target.value); onText(field, e.target.value.trim() ? e.target.value : null); }} />;
}

export function ExposureGroup(p: GroupProps) {
  const selected = p.rows.map((r) => r.category);
  const none = p.status?.value_text === "none_reported";
  const toggle = async (code: string, on: boolean) => {
    const next = on ? [...selected, code] : selected.filter((c) => c !== code);
    const removing = !on && p.rows.find((r) => r.category === code);
    if (removing && (removing.freq_min !== null || removing.quantity_text)) {
      if (!(await confirmAsk("Retirer cette catégorie efface aussi sa fréquence et sa quantité."))) return;
    }
    p.onSelect(p.group, false, next);
  };
  const summary = none ? p.noneLabel : selected.length ? p.options.filter((o) => selected.includes(o.code)).map((o) => o.label).join(", ") : null;
  return (
    <div className="field">
      <div className="label"><span>{p.title}</span></div>
      <details className="picks">
        <summary>{summary ?? <span className="muted">Non renseigné — choisir</span>}</summary>
        <div className="options">
          {p.options.map((o) => (
            <label key={o.code} className="check">
              <input type="checkbox" disabled={p.readOnly} checked={selected.includes(o.code)} onChange={(e) => toggle(o.code, e.target.checked)} />{o.label}
            </label>
          ))}
          <label className="check excl">
            <input type="checkbox" disabled={p.readOnly} checked={none}
              onChange={async (e) => {
                const on = e.target.checked;
                if (on && selected.length && !(await confirmAsk(`« ${p.noneLabel} » retire les catégories cochées.`))) return;
                p.onSelect(p.group, on, []);
              }} />
            {p.noneLabel} (recherchées)
          </label>
        </div>
      </details>
      {p.rows.map((r) => <Detail key={r.category} row={r} label={p.options.find((o) => o.code === r.category)?.label ?? r.category} readOnly={p.readOnly} onDetail={p.onDetail} />)}
      <FreeText field={p.freeTextField} value={p.freeText} readOnly={p.readOnly} onText={p.onText} placeholder={p.group === "drink" ? "Autre boisson, précisions…" : "Autre aliment, précisions…"} />
    </div>
  );
}
