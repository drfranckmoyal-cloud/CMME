// Prévention : HBD indépendant en tête, protocole (aucun / modéré / avancé / personnalisé),
// composantes proposées à confirmer, gouttière semi-rigide indépendante. Rien n'est prescrit
// automatiquement ; un conseil n'est jamais présenté comme un usage.
import { useEffect, useRef, useState } from "react";
import type { CatalogBundle, PreventionRow, StoredValue } from "../types";

const PROTOCOLS = [
  { code: "none", label: "Aucun protocole proposé", sub: "Réponse explicite, distincte de « non renseigné »" },
  { code: "moderate", label: "Modéré", sub: "Dentifrice anti-érosion + bain de bouche après vomissement" },
  { code: "advanced", label: "Avancé", sub: "Modéré + Tooth Mousse quotidien" },
  { code: "custom", label: "Personnalisé", sub: "Mesures choisies une à une" },
];
const STATUS_TOGGLES: [keyof PreventionRow, string][] = [
  ["st_already_used", "Déjà utilisé selon le patient"],
  ["st_done_in_consultation", "Réalisé pendant la consultation"],
  ["st_given_today", "Remis aujourd'hui"],
  ["st_refused", "Refusé"],
];

interface Props {
  bundle: CatalogBundle;
  values: Map<string, StoredValue>;
  rows: PreventionRow[];
  readOnly: boolean;
  onField: (field: string, value: unknown, immediate?: boolean) => void;
  onProtocol: (p: string | null) => void;
  onAction: (action: string, patch: Record<string, unknown>) => void;
  onConfirm: () => void;
  keptOutside: string[];
}

function LazyText({ value, onCommit, placeholder, readOnly, label }: { value: string | null; onCommit: (v: string) => void; placeholder: string; readOnly: boolean; label: string }) {
  const [t, setT] = useState(value ?? "");
  const focused = useRef(false);
  useEffect(() => { if (!focused.current) setT(value ?? ""); }, [value]);
  return <input value={t} placeholder={placeholder} readOnly={readOnly} aria-label={label}
    onFocus={() => (focused.current = true)} onChange={(e) => setT(e.target.value)}
    onBlur={() => { focused.current = false; if ((value ?? "") !== t) onCommit(t); }} />;
}

function ActionCard({ row, label, readOnly, onAction, outside }: { row: PreventionRow; label: string; readOnly: boolean; onAction: Props["onAction"]; outside: boolean }) {
  const isTM = row.action_type === "tooth_mousse";
  const isRinse = row.action_type === "anti_erosion_rinse";
  return (
    <div className={`action-card ${row.confirmed ? "" : "unconfirmed"}`}>
      <div className="action-head">
        <span>{label}</span>
        <span className="chips">
          {!row.confirmed && row.decision && <span className="chip warn">Proposé — à confirmer</span>}
          {row.confirmed && row.decision === "advised" && <span className="chip ok">Conseillé ce jour (confirmé)</span>}
          {outside && <span className="chip warn">Hors du protocole choisi</span>}
        </span>
      </div>
      <div className="seg" role="group" aria-label={`Décision : ${label}`}>
        {[["advised", "Conseillé"], ["not_advised", "Non conseillé"], ...(isRinse ? [["not_applicable", "Non applicable (pas de vomissement)"]] : [])].map(([c, l]) => (
          <button key={c} type="button" disabled={readOnly} aria-pressed={row.decision === c} onClick={() => onAction(row.action_type, { decision: row.decision === c ? "" : c })}>{l}</button>
        ))}
      </div>
      {isTM && row.decision === "advised" && (
        <div className="inline-fields">
          <label>Application
            <select value={row.route ?? ""} disabled={readOnly} onChange={(e) => {
              const r = e.target.value;
              onAction(row.action_type, { route: r, tray_minutes: r === "tray" || r === "both" ? (row.tray_minutes ?? 10) : null });
            }}>
              <option value="">à renseigner</option><option value="direct">Directe</option><option value="tray">En gouttières</option><option value="both">Les deux</option><option value="unspecified">Non précisé</option>
            </select>
          </label>
          {(row.route === "tray" || row.route === "both") && (
            <label>Durée en gouttières
              <LazyText value={row.tray_minutes != null ? String(row.tray_minutes) : ""} placeholder="10" readOnly={readOnly} label="Minutes par jour"
                onCommit={(v) => onAction(row.action_type, { tray_minutes: v || null })} /> min / jour
            </label>
          )}
          <label>Fréquence
            <LazyText value={row.frequency_text} placeholder="quotidien" readOnly={readOnly} label="Fréquence conseillée" onCommit={(v) => onAction(row.action_type, { frequency_text: v })} />
          </label>
        </div>
      )}
      <div className="toggles" role="group" aria-label={`Statuts : ${label}`}>
        {STATUS_TOGGLES.map(([k, l]) => (
          <button key={k} type="button" disabled={readOnly} aria-pressed={!!row[k]} onClick={() => onAction(row.action_type, { [k]: !row[k] })}>{l}</button>
        ))}
      </div>
      <div className="inline-fields">
        <LazyText value={row.product_text} placeholder="Produit exact (facultatif)" readOnly={readOnly} label={`Produit : ${label}`} onCommit={(v) => onAction(row.action_type, { product_text: v })} />
      </div>
    </div>
  );
}

const COMPONENTS: Record<string, string[]> = {
  moderate: ["anti_erosion_toothpaste", "anti_erosion_rinse"],
  advanced: ["anti_erosion_toothpaste", "anti_erosion_rinse", "tooth_mousse"],
};
function isOutside(protocol: string | null, action: string): boolean {
  return !!protocol && protocol in COMPONENTS && action !== "other" && !COMPONENTS[protocol].includes(action);
}

export function PreventionPanel(p: Props) {
  const hbd = p.values.get("hbd_teaching");
  const protocol = p.values.get("prevention_protocol")?.value_text ?? null;
  const label = (code: string) => p.bundle.prevention_actions.find((a) => a.code === code)?.label ?? code;
  const main = p.rows.filter((r) => r.action_type !== "vomiting_semirigid_tray");
  const tray = p.rows.find((r) => r.action_type === "vomiting_semirigid_tray");
  const unconfirmed = p.rows.filter((r) => !r.confirmed && r.decision);
  const addable = ["anti_erosion_toothpaste", "anti_erosion_rinse", "tooth_mousse", "other"].filter((a) => !p.rows.some((r) => r.action_type === a));
  const [adding, setAdding] = useState("");
  const brux = p.values.get("bruxism_tray_existing")?.value_text ?? null;
  return (
    <>
      <div className="hbd">
        <label>
          <input type="checkbox" disabled={p.readOnly} checked={hbd?.value_text === "done"}
            onChange={(e) => p.onField("hbd_teaching", e.target.checked ? "done" : null, true)} />
          Enseignement HBD
        </label>
        {hbd?.value_text === "not_done" && <span className="missing-chip">Non réalisé (explicite) {!p.readOnly && <button type="button" onClick={() => p.onField("hbd_teaching", null, true)}>×</button>}</span>}
        {!hbd && !p.readOnly && <button type="button" className="btn ghost small" onClick={() => p.onField("hbd_teaching", "not_done", true)}>Noter « non réalisé »</button>}
        <span className="small muted grow" style={{ textAlign: "right" }}>Indépendant du protocole ; décocher revient à « non renseigné ».</span>
      </div>

      <h3>Protocole proposé</h3>
      <div className="protocols" role="group" aria-label="Protocole proposé">
        {PROTOCOLS.map((x) => (
          <button key={x.code} type="button" disabled={p.readOnly} aria-pressed={protocol === x.code} onClick={() => p.onProtocol(protocol === x.code ? null : x.code)}>
            {x.label}<small>{x.sub}</small>
          </button>
        ))}
      </div>
      {!protocol && <p className="small muted">Non renseigné. Choisir un protocole prépare ses mesures, à confirmer ; rien n'est enregistré comme conseillé sans confirmation.</p>}
      {protocol === "none" && main.some((r) => r.confirmed && r.decision === "advised") && (
        <div className="banner warn">« Aucun protocole proposé » coexiste avec des mesures confirmées : vérifiez ou retirez-les.</div>
      )}

      {main.map((r) => <ActionCard key={r.action_type} row={r} label={label(r.action_type)} readOnly={p.readOnly} onAction={p.onAction} outside={isOutside(protocol, r.action_type) || p.keptOutside.includes(r.action_type)} />)}

      {!p.readOnly && addable.length > 0 && (
        <div className="inline-fields" style={{ marginTop: 12 }}>
          <select value={adding} onChange={(e) => setAdding(e.target.value)} aria-label="Ajouter une mesure">
            <option value="">Ajouter une mesure…</option>
            {addable.map((a) => <option key={a} value={a}>{label(a)}</option>)}
          </select>
          <button type="button" className="btn small" disabled={!adding} onClick={() => { p.onAction(adding, { decision: "advised" }); setAdding(""); }}>Ajouter comme conseillée</button>
        </div>
      )}

      <div className="block-title">Gouttières semi-rigides pendant les vomissements</div>
      <p className="small muted" style={{ marginTop: 4 }}>Option indépendante, jamais cochée par un protocole. Distincte des gouttières d'application et de bruxisme.</p>
      <div className="seg" role="group" aria-label="Gouttières semi-rigides pendant les vomissements" style={{ marginTop: 8 }}>
        {[["advised", "Conseillées"], ["not_advised", "Non conseillées"], ["not_applicable", "Non applicable"]].map(([c, l]) => (
          <button key={c} type="button" disabled={p.readOnly} aria-pressed={tray?.decision === c} onClick={() => p.onAction("vomiting_semirigid_tray", { decision: tray?.decision === c ? "" : c })}>{l}</button>
        ))}
      </div>

      <div className="fields">
        <div className="field">
          <div className="label"><span>Gouttière de bruxisme existante</span></div>
          <div className="seg" role="group" aria-label="Gouttière de bruxisme existante">
            {[["yes", "Oui"], ["no", "Non"]].map(([c, l]) => (
              <button key={c} type="button" disabled={p.readOnly} aria-pressed={brux === c} onClick={() => p.onField("bruxism_tray_existing", brux === c ? null : c, true)}>{l}</button>
            ))}
          </div>
        </div>
        <div className="field">
          <div className="label"><span>Adaptation / précisions</span></div>
          <LazyText value={p.values.get("prevention_note")?.value_text ?? null} placeholder="Texte court facultatif" readOnly={p.readOnly} label="Adaptation" onCommit={(v) => p.onField("prevention_note", v.trim() ? v : null, true)} />
        </div>
        <div className="field wide">
          <div className="label"><span>Autre action (information, démonstration, coordination avec l'équipe)</span></div>
          <LazyText value={p.values.get("other_action")?.value_text ?? null} placeholder="Facultatif" readOnly={p.readOnly} label="Autre action" onCommit={(v) => p.onField("other_action", v.trim() ? v : null, true)} />
        </div>
      </div>

      {unconfirmed.length > 0 && !p.readOnly && (
        <div className="banner info" style={{ marginTop: 18 }}>
          <span className="grow">{unconfirmed.length} mesure{unconfirmed.length > 1 ? "s" : ""} proposée{unconfirmed.length > 1 ? "s" : ""}, non encore confirmée{unconfirmed.length > 1 ? "s" : ""}. Sans confirmation, elles ne comptent pas comme conseillées.</span>
          <button type="button" className="btn primary small" onClick={p.onConfirm}>Confirmer les mesures conseillées</button>
        </div>
      )}
    </>
  );
}
