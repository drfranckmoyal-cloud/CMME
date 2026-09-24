// Champ générique piloté par le catalogue : aucune valeur par défaut, raison de manque explicite.
import { useEffect, useRef, useState } from "react";
import type { Field, FieldInput as FI, StoredValue } from "../types";
import { fmtClinicalDate, numToInput, parseFrNumber, parseFrenchDate } from "../lib/format";

const MISSING_LABELS: Record<string, string> = {
  unknown: "Inconnu", declined: "Refus de répondre", not_applicable: "Non applicable", not_assessable: "Non évaluable",
  not_recorded: "Non consigné (ancien support)", ambiguous_source: "Source ambiguë", not_asked: "Non renseigné",
};

interface Props {
  field: Field;
  value: StoredValue | undefined;
  onChange: (input: FI, immediate?: boolean) => void;
  onBlur?: () => void;
  readOnly?: boolean;
  conditionUnmet?: boolean;
}

function MissingMenu({ field, onPick, hasValue, readOnly }: { field: Field; onPick: (r: string | null) => void; hasValue: boolean; readOnly?: boolean }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent) => { if (!ref.current?.contains(e.target as Node)) setOpen(false); };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, [open]);
  if (readOnly || (field.missing.length === 0 && !hasValue)) return null;
  return (
    <div className="menu-wrap" ref={ref}>
      <button type="button" className="menu-btn" aria-haspopup="menu" aria-expanded={open} aria-label={`Autres réponses pour ${field.label}`} onClick={() => setOpen(!open)}>•••</button>
      {open && (
        <div className="menu" role="menu">
          {field.missing.map((m) => (
            <button key={m} role="menuitem" type="button" onClick={() => { onPick(m); setOpen(false); }}>{MISSING_LABELS[m] ?? m}</button>
          ))}
          {hasValue && <button role="menuitem" type="button" onClick={() => { onPick(null); setOpen(false); }}>Effacer (non renseigné)</button>}
        </div>
      )}
    </div>
  );
}

function NumberField({ field, value, onChange, onBlur, readOnly }: Props) {
  const [a, setA] = useState(numToInput(value?.value_num));
  const [b, setB] = useState(numToInput(value?.value_num_max));
  const [range, setRange] = useState(value?.value_num_max != null);
  const [estimated, setEstimated] = useState(value?.precision === "estimated");
  const [error, setError] = useState<string | null>(null);
  const focused = useRef(false);
  useEffect(() => {
    if (focused.current) return;
    setA(numToInput(value?.value_num));
    setB(numToInput(value?.value_num_max));
    if (value?.value_num_max != null) setRange(true);
    setEstimated(value?.precision === "estimated");
  }, [value?.value_num, value?.value_num_max, value?.precision]);

  const emit = (na: string, nb: string, est: boolean, useRange: boolean) => {
    const x = parseFrNumber(na);
    const y = useRange ? parseFrNumber(nb) : null;
    if (x === null && (y === null || !useRange)) { setError(null); onChange({ field: field.code, value: null }); return; }
    if (x === null || Number.isNaN(x) || (y !== null && Number.isNaN(y))) { setError("Nombre non valide : saisie non enregistrée"); return; }
    if (field.min !== null && x < field.min || field.max !== null && x > field.max || y !== null && field.max !== null && y > field.max) { setError(`Hors bornes (${field.min} à ${field.max}) : non enregistré`); return; }
    if (field.integer && (!Number.isInteger(x) || (y !== null && !Number.isInteger(y)))) { setError("Nombre entier attendu : non enregistré"); return; }
    if (y !== null && y < x) { setError("Borne haute inférieure à la borne basse"); return; }
    setError(null);
    onChange({ field: field.code, value: y !== null && y !== x ? { min: x, max: y } : x, precision: est ? "estimated" : "exact" });
  };
  return (
    <>
      <div className="input-unit">
        <input inputMode="decimal" value={a} readOnly={readOnly} aria-label={field.label} aria-invalid={!!error}
          onFocus={() => (focused.current = true)}
          onChange={(e) => { setA(e.target.value); emit(e.target.value, b, estimated, range); }}
          onBlur={() => { focused.current = false; onBlur?.(); }} />
        {range && (<><span className="to">à</span>
          <input inputMode="decimal" value={b} readOnly={readOnly} aria-label={`${field.label} (borne haute)`}
            onFocus={() => (focused.current = true)}
            onChange={(e) => { setB(e.target.value); emit(a, e.target.value, estimated, true); }}
            onBlur={() => { focused.current = false; onBlur?.(); }} /></>)}
        {field.unit && <span className="unit">{field.unit}</span>}
        {field.allow_range && !readOnly && (
          <>
            <button type="button" className="btn ghost small" onClick={() => { const r = !range; setRange(r); if (!r) { setB(""); emit(a, "", estimated, false); } }}>{range ? "valeur unique" : "plage"}</button>
            {!range && <label className="check small" title="Valeur estimée"><input type="checkbox" checked={estimated} disabled={readOnly} onChange={(e) => { setEstimated(e.target.checked); emit(a, b, e.target.checked, range); }} />estimé</label>}
          </>
        )}
      </div>
      {error && <span className="err-note" role="alert">{error}</span>}
    </>
  );
}

function DateField({ field, value, onChange, onBlur, readOnly }: Props) {
  const [text, setText] = useState(value?.value_text ? fmtClinicalDate(value.value_text) : "");
  const [error, setError] = useState<string | null>(null);
  const focused = useRef(false);
  useEffect(() => { if (!focused.current) setText(value?.value_text ? fmtClinicalDate(value.value_text) : ""); }, [value?.value_text]);
  return (
    <>
      <input value={text} readOnly={readOnly} placeholder="JJ/MM/AAAA" aria-label={field.label}
        onFocus={() => (focused.current = true)}
        onChange={(e) => {
          setText(e.target.value);
          if (!e.target.value.trim()) { setError(null); onChange({ field: field.code, value: null }); return; }
          const iso = parseFrenchDate(e.target.value);
          if (iso) { setError(null); onChange({ field: field.code, value: iso }); } else setError("Date incomplète ou invalide : non enregistrée");
        }}
        onBlur={() => { focused.current = false; onBlur?.(); }} />
      {error && text.length >= 4 && <span className="err-note" role="alert">{error}</span>}
    </>
  );
}

function TextField({ field, value, onChange, onBlur, readOnly }: Props) {
  const [text, setText] = useState(value?.value_text ?? "");
  const focused = useRef(false);
  useEffect(() => { if (!focused.current) setText(value?.value_text ?? ""); }, [value?.value_text]);
  const big = field.code === "consultation_observations" || field.code === "observations_legacy";
  const common = {
    value: text, readOnly, "aria-label": field.label, placeholder: field.hint ?? "",
    onFocus: () => (focused.current = true),
    onChange: (e: { target: { value: string } }) => { setText(e.target.value); onChange({ field: field.code, value: e.target.value.trim() ? e.target.value : null }); },
    onBlur: () => { focused.current = false; onBlur?.(); },
  };
  return field.wide ? <textarea className={big ? "big" : ""} {...common} /> : <input {...common} />;
}

function MultiField({ field, value, onChange, readOnly }: Props) {
  const selected: string[] = value?.value_text ? JSON.parse(value.value_text) : [];
  const toggle = (code: string, on: boolean) => {
    let next = on ? [...selected, code] : selected.filter((c) => c !== code);
    if (on && field.exclusive.includes(code)) next = [code];
    if (on && !field.exclusive.includes(code)) next = next.filter((c) => !field.exclusive.includes(c));
    onChange({ field: field.code, value: next.length ? next : null }, true);
  };
  const labels = field.options.filter((o) => selected.includes(o.code)).map((o) => o.label);
  return (
    <details className="picks">
      <summary>{labels.length ? labels.join(", ") : <span className="muted">{value?.missing_reason ? MISSING_LABELS[value.missing_reason] : "Non renseigné — choisir"}</span>}</summary>
      <div className="options">
        {field.options.map((o) => (
          <label key={o.code} className={`check ${field.exclusive.includes(o.code) ? "excl" : ""}`}>
            <input type="checkbox" disabled={readOnly} checked={selected.includes(o.code)} onChange={(e) => toggle(o.code, e.target.checked)} />
            {o.label}
          </label>
        ))}
      </div>
    </details>
  );
}

function ChoiceField({ field, value, onChange, readOnly }: Props) {
  const current = value?.value_text ?? null;
  const long = field.options.length > 4 || field.options.some((o) => o.label.length > 26);
  if (long) {
    return (
      <select value={current ?? (value?.missing_reason ? `missing:${value.missing_reason}` : "")} disabled={readOnly} aria-label={field.label}
        onChange={(e) => {
          const v = e.target.value;
          if (!v) onChange({ field: field.code, value: null }, true);
          else if (v.startsWith("missing:")) onChange({ field: field.code, missing_reason: v.slice(8) }, true);
          else onChange({ field: field.code, value: v }, true);
        }}>
        <option value="">Non renseigné</option>
        {field.options.map((o) => <option key={o.code} value={o.code}>{o.label}</option>)}
        {field.missing.length > 0 && (
          <optgroup label="Réponse manquante">
            {field.missing.map((m) => <option key={m} value={`missing:${m}`}>{MISSING_LABELS[m] ?? m}</option>)}
          </optgroup>
        )}
      </select>
    );
  }
  return (
    <div className="seg" role="group" aria-label={field.label}>
      {field.options.map((o) => (
        <button key={o.code} type="button" disabled={readOnly} aria-pressed={current === o.code}
          onClick={() => onChange(current === o.code ? { field: field.code, value: null } : { field: field.code, value: o.code }, true)}>
          {o.label}
        </button>
      ))}
    </div>
  );
}

export function FieldView(props: Props) {
  const { field, value, onChange, readOnly, conditionUnmet } = props;
  const missing = value?.missing_reason;
  const hasValue = !!value;
  const longChoice = field.kind === "choice" && (field.options.length > 4 || field.options.some((o) => o.label.length > 26));
  const showMissingChip = missing && !longChoice && field.kind !== "multi";
  return (
    <div className={`field ${field.wide ? "wide" : ""}`} data-field={field.code}>
      <div className="label">
        <span>{field.label}{field.hint && field.kind !== "text" ? <span className="hint"> · {field.hint}</span> : null}</span>
        <MissingMenu field={field} hasValue={hasValue} readOnly={readOnly} onPick={(r) => onChange(r ? { field: field.code, missing_reason: r } : { field: field.code, value: null }, true)} />
      </div>
      {showMissingChip ? (
        <span className="missing-chip">{MISSING_LABELS[missing] ?? missing}
          {!readOnly && <button type="button" onClick={() => onChange({ field: field.code, value: null }, true)} aria-label="Effacer la raison de manque">×</button>}
        </span>
      ) : (
        <>
          {field.kind === "choice" && <ChoiceField {...props} />}
          {field.kind === "multi" && <MultiField {...props} />}
          {field.kind === "number" && <NumberField {...props} />}
          {field.kind === "date" && <DateField {...props} />}
          {field.kind === "text" && <TextField {...props} />}
        </>
      )}
      {conditionUnmet && hasValue && <span className="warn-note">Valeur conservée alors que la réponse dont elle dépend a changé.</span>}
    </div>
  );
}
