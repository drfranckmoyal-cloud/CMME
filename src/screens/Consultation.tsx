// Consultation sur une page continue : Contexte, Habitudes, Expositions, Examen, Prévention,
// Observations. Les raccourcis font défiler la page ; aucune rubrique n'est masquée.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { call, errMessage } from "../api";
import type { CatalogBundle, EncounterFull, Field, FieldInput, Identity, Recap, Section, StoredValue } from "../types";
import { useEncounter, type SaveState } from "../lib/useEncounter";
import { FieldView } from "../components/FieldInput";
import { BeweChart, BeweSummary } from "../components/Bewe";
import { ExposureGroup } from "../components/Exposures";
import { PreventionPanel } from "../components/Prevention";
import { Modal } from "../components/Modal";
import { fmtClinicalDate, fmtDateTime, fmtTime, PROTOCOL_LABELS, SERVICE_LABELS, STATUS_LABELS, VOMITING_LABELS } from "../lib/format";

const PROSPECTIVE_SECTIONS = ["contexte", "habitudes", "expositions", "examen", "prevention", "observations"];

export function SaveIndicator({ save, onRetry }: { save: SaveState; onRetry: () => void }) {
  if (save.kind === "error") {
    return (
      <span className="savestate error" role="alert">
        {save.errKind === "conflict" ? "Conflit de version" : "Échec de l'enregistrement"}
        <button type="button" className="link" onClick={onRetry}>{save.errKind === "conflict" ? "Recharger" : "Réessayer"}</button>
      </span>
    );
  }
  const label = save.kind === "saved" ? `Enregistré à ${fmtTime(save.at)}` : save.kind === "saving" ? "Enregistrement…" : save.kind === "dirty" ? "Modifications en cours" : "Aucune modification";
  return <span className={`savestate ${save.kind}`} aria-live="polite">{label}</span>;
}

function conditionState(f: Field, values: Map<string, StoredValue>): "met" | "unmet" {
  if (!f.show_if) return "met";
  const dep = values.get(f.show_if.field)?.value_text;
  return dep && f.show_if.values.includes(dep) ? "met" : "unmet";
}

interface Props {
  bundle: CatalogBundle;
  encounterId: string;
  clinical: boolean;
  registerFlush: (fn: (() => Promise<boolean>) | null) => void;
  onOpenDossiers: () => void;
  onNewDossier: () => void;
  onOpenEncounter: (id: string) => void;
}

export function Consultation({ bundle, encounterId, clinical, registerFlush, onOpenDossiers, onNewDossier, onOpenEncounter }: Props) {
  const E = useEncounter(encounterId);
  const { enc, values, save } = E;
  const [current, setCurrent] = useState("contexte");
  const [recap, setRecap] = useState<Recap | null>(null);
  const [amendOpen, setAmendOpen] = useState(false);
  const [amendReason, setAmendReason] = useState("");
  const [historyOpen, setHistoryOpen] = useState<null | { revisions: { revision: number; reason: string | null; created_at: string; author: string | null }[]; audit: { at: string; kind: string; field: string | null; old_value: string | null; new_value: string | null; reason: string | null; author: string | null }[] }>(null);
  const [identityOpen, setIdentityOpen] = useState(false);
  const [notice, setNotice] = useState<{ kind: "ok" | "error" | "warn" | "info"; text: string } | null>(null);
  const [keptOutside, setKeptOutside] = useState<string[]>([]);
  const [others, setOthers] = useState<{ encounter_id: string; visit_date: string | null; status: string }[]>([]);

  useEffect(() => {
    registerFlush(E.flush);
    return () => registerFlush(null);
  }, [E.flush, registerFlush]);

  useEffect(() => {
    if (!enc) return;
    call<{ encounter_id: string; visit_date: string | null; status: string }[]>("patient_encounters", { patientId: enc.meta.patient_id }).then(setOthers).catch(() => setOthers([]));
  }, [enc?.meta.patient_id, enc?.meta.status]);

  const legacy = enc?.meta.collection_mode === "legacy_retrospective";
  const readOnly = !enc || enc.meta.status === "validated";
  const sections: Section[] = useMemo(() => bundle.catalog.sections.filter((s) => (legacy ? s.id === "historique" : PROSPECTIVE_SECTIONS.includes(s.id))), [bundle, legacy]);

  // Rubrique courante selon le défilement.
  const refs = useRef<Record<string, HTMLElement | null>>({});
  useEffect(() => {
    const obs = new IntersectionObserver((entries) => {
      const vis = entries.filter((e) => e.isIntersecting).sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top);
      if (vis[0]) setCurrent(vis[0].target.id.replace("sec-", ""));
    }, { rootMargin: "-160px 0px -55% 0px" });
    Object.values(refs.current).forEach((el) => el && obs.observe(el));
    // En bas de page, la dernière rubrique est la rubrique courante.
    const onScroll = () => {
      if (window.innerHeight + window.scrollY >= document.body.scrollHeight - 4 && sections.length) setCurrent(sections[sections.length - 1].id);
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => { obs.disconnect(); window.removeEventListener("scroll", onScroll); };
  }, [sections, enc?.meta.id]);

  const setField = useCallback((input: FieldInput, immediate?: boolean) => { if (!readOnly) E.setField(input, immediate); }, [E, readOnly]);

  const guard = async (p: Promise<unknown>) => {
    try { await p; } catch (e) { setNotice({ kind: "error", text: errMessage(e) }); }
  };

  const finish = async () => {
    setNotice(null);
    if (!(await E.flush())) { setNotice({ kind: "error", text: "Des saisies ne sont pas enregistrées : réessayez avant de terminer." }); return; }
    try { setRecap(await call<Recap>("recap", { id: encounterId })); } catch (e) { setNotice({ kind: "error", text: errMessage(e) }); }
  };

  const doValidate = async () => {
    try {
      const full = await E.op<EncounterFull>("validate_encounter", {}, (r) => r.meta.version);
      setRecap(null);
      setNotice({ kind: "ok", text: `Saisie terminée et verrouillée (révision ${full.meta.revision}). Aucun document n'a été produit.` });
      window.scrollTo({ top: 0 });
    } catch (e) { setNotice({ kind: "error", text: errMessage(e) }); }
  };

  const doAmend = async () => {
    try {
      await E.op<EncounterFull>("start_amendment", { reason: amendReason }, (r) => r.meta.version);
      setAmendOpen(false); setAmendReason("");
      setNotice({ kind: "warn", text: "Correction en cours : chaque modification est tracée avec le motif. Terminez la saisie pour valider l'amendement." });
    } catch (e) { setNotice({ kind: "error", text: errMessage(e) }); }
  };

  if (E.loadError) return <div className="page"><div className="banner error">{E.loadError}</div></div>;
  if (!enc) return <div className="page"><p className="muted">Chargement de la consultation…</p></div>;

  const m = enc.meta;
  const valueText = (f: string) => values.get(f)?.value_text ?? null;
  const vom = values.get("vomiting_lifetime");
  const statusBadge = m.status === "validated" ? (m.revision > 1 ? `Validée · amendée (rév. ${m.revision})` : "Validée") : STATUS_LABELS[m.status];
  const name = enc.identity ? [enc.identity.last_name, enc.identity.first_name].filter(Boolean).join(" ") : "";

  const genericFields = (block: string, exclude: string[] = []) =>
    bundle.catalog.fields.filter((f) => f.block === block && !f.custom && !exclude.includes(f.code)).map((f) => {
      const cond = conditionState(f, values);
      const v = values.get(f.code);
      if (cond === "unmet" && !v) return null;
      return <FieldView key={f.code} field={f} value={v} onChange={setField} readOnly={readOnly} conditionUnmet={cond === "unmet"} />;
    });

  const blockContent = (blockId: string) => {
    if (blockId === "examen_bewe") {
      return (
        <>
          <div className="block-title">BEWE — score le plus élevé de chaque sextant</div>
          <BeweChart bundle={bundle} rows={enc.bewe} readOnly={readOnly}
            onSet={(sextant, score, missing, reason) => guard(E.op("set_sextant", { input: { sextant, score, missing_reason: missing, unassessable_reason: reason } }))} />
          <p className="small muted" style={{ marginTop: 10 }}>Clavier : Tab jusqu'à un sextant, puis 0 à 3, N (non évaluable) ou Effacement. Le total n'apparaît que lorsque les six sextants sont scorés ; aucun zéro n'est supposé.</p>
        </>
      );
    }
    if (blockId === "expositions_main") {
      return (
        <>
          <div className="fields">{genericFields(blockId, ["exposure_note"])}</div>
          <div className="expo-grid">
            <ExposureGroup group="drink" title="Boissons" noneLabel="Aucune boisson acide rapportée" options={bundle.drinks} status={values.get("acidic_drinks_status")}
              rows={enc.exposures.filter((x) => x.grp === "drink")} freeText={values.get("acidic_drinks_free_text")} freeTextField="acidic_drinks_free_text" readOnly={readOnly}
              onSelect={(g, none, cats) => guard(E.op("set_exposure_group", { group: g, noneReported: none, categories: cats }))}
              onDetail={(g, c, patch) => guard(E.op("update_exposure", { group: g, category: c, patch }))}
              onText={(f, v) => setField({ field: f, value: v })} />
            <ExposureGroup group="food" title="Aliments" noneLabel="Aucun aliment acide rapporté" options={bundle.foods} status={values.get("acidic_foods_status")}
              rows={enc.exposures.filter((x) => x.grp === "food")} freeText={values.get("acidic_foods_free_text")} freeTextField="acidic_foods_free_text" readOnly={readOnly}
              onSelect={(g, none, cats) => guard(E.op("set_exposure_group", { group: g, noneReported: none, categories: cats }))}
              onDetail={(g, c, patch) => guard(E.op("update_exposure", { group: g, category: c, patch }))}
              onText={(f, v) => setField({ field: f, value: v })} />
          </div>
          <div className="fields">{bundle.catalog.fields.filter((f) => f.code === "exposure_note").map((f) => <FieldView key={f.code} field={f} value={values.get(f.code)} onChange={setField} readOnly={readOnly} />)}</div>
        </>
      );
    }
    if (blockId === "prevention_main") {
      return (
        <PreventionPanel bundle={bundle} values={values} rows={enc.prevention} readOnly={readOnly} keptOutside={keptOutside}
          onField={(f, v, imm) => setField({ field: f, value: v }, imm)}
          onProtocol={async (p) => {
            try {
              const r = await E.op<{ save: { version: number }; kept_confirmed_outside: string[]; removed_unconfirmed: string[] }>("apply_protocol", { protocol: p }, (x) => x.save.version);
              setKeptOutside(r.kept_confirmed_outside);
              if (r.kept_confirmed_outside.length) setNotice({ kind: "warn", text: "Des mesures déjà confirmées ne font pas partie du protocole choisi : elles sont conservées et signalées, à retirer si besoin." });
            } catch (e) { setNotice({ kind: "error", text: errMessage(e) }); }
          }}
          onAction={(a, patch) => guard(E.op("update_prevention_action", { action: a, patch }))}
          onConfirm={() => guard(E.op("confirm_prevention", {}))} />
      );
    }
    return <div className="fields">{genericFields(blockId)}</div>;
  };

  const filledIn = (blockId: string) => bundle.catalog.fields.filter((f) => f.block === blockId && values.has(f.code)).length;

  return (
    <div className="page">
      <div className="heading">
        <div>
          <div className="kicker">{new Date().toLocaleDateString("fr-FR", { weekday: "long", day: "numeric", month: "long" })}</div>
          <h1>{legacy ? "Consultation reprise." : "Votre consultation."}</h1>
        </div>
        <div className="row">
          <button type="button" className="btn" onClick={onOpenDossiers}>Voir les dossiers</button>
          <button type="button" className="btn" onClick={onNewDossier}>Nouveau dossier</button>
        </div>
      </div>

      <div className="sticky-head">
        <div className="patient">
          <div className="patient-avatar" aria-hidden="true">{clinical && name ? name.split(/\s+/).map((w) => w[0]).join("").slice(0, 2).toUpperCase() : m.patient_code.slice(-2)}</div>
          <div>
            <div className="patient-name">{clinical && name ? name : `Dossier ${m.patient_code}`}</div>
            <div className="meta">
              {clinical && name ? `Dossier ${m.patient_code} · ` : ""}{fmtClinicalDate(m.visit_date)} · {m.service_code ? SERVICE_LABELS[m.service_code] : "service non renseigné"}
              {m.service_label_source ? ` (source : ${m.service_label_source})` : ""} · {legacy ? "ancien recueil" : "recueil structuré"} · {m.examiner ?? ""}
            </div>
          </div>
          <div className="row" style={{ marginLeft: "auto" }}>
            {legacy && <span className="badge legacy">Historique</span>}
            <span className={`badge ${m.status}`}>{statusBadge}</span>
            <SaveIndicator save={save} onRetry={() => (save.kind === "error" && save.errKind === "conflict" ? E.discardAndReload() : E.retry())} />
          </div>
        </div>
        {!legacy && (
          <nav className="sectionnav" aria-label="Rubriques de la consultation">
            {sections.map((s) => (
              <button key={s.id} type="button" aria-current={current === s.id} onClick={() => { document.getElementById(`sec-${s.id}`)?.scrollIntoView({ block: "start" }); setCurrent(s.id); }}>
                <span className="step">{s.number}</span>{s.title}
              </button>
            ))}
          </nav>
        )}
      </div>

      {m.is_demo && <div className="banner warn">Profil de démonstration : données entièrement fictives, base séparée du profil clinique.</div>}
      {notice && <div className={`banner ${notice.kind}`} role="status"><span className="grow">{notice.text}</span><button type="button" className="link" onClick={() => setNotice(null)}>Fermer</button></div>}
      {m.status === "validated" && (
        <div className="banner info">
          <span className="grow">Consultation validée le {fmtDateTime(m.validated_at)} et verrouillée. Une correction crée un amendement motivé ; l'état antérieur reste consultable.</span>
          <button type="button" className="btn small" onClick={() => setAmendOpen(true)}>Corriger</button>
          <button type="button" className="btn small" onClick={async () => setHistoryOpen(await call("history", { id: m.id }))}>Historique</button>
        </div>
      )}
      {m.status === "amending" && (
        <div className="banner warn"><span className="grow">Correction en cours (amendement). Chaque modification est tracée avec son motif.</span>
          <button type="button" className="btn small" onClick={async () => setHistoryOpen(await call("history", { id: m.id }))}>Historique</button></div>
      )}
      {save.kind === "error" && (
        <div className="banner error" role="alert">
          <span className="grow">{save.message} Vos dernières saisies restent à l'écran et ne sont pas enregistrées.</span>
          {save.errKind === "conflict"
            ? <button type="button" className="btn small" onClick={E.discardAndReload}>Recharger la version enregistrée</button>
            : <button type="button" className="btn small" onClick={E.retry}>Réessayer</button>}
        </div>
      )}

      <div className="content">
        <div className="main">
          {legacy && (
            <div className="banner info"><span className="grow">Ligne reprise de l'ancien recueil. Les valeurs sont conservées telles quelles ; rien n'a été reconstruit (ni sextants, ni composantes de prévention, ni âge exact).</span></div>
          )}
          {sections.map((s) => (
            <section key={s.id} id={`sec-${s.id}`} ref={(el) => { refs.current[s.id] = el; }} className="panel" aria-labelledby={`h-${s.id}`}>
              <span className="section-number">{String(s.number).padStart(2, "0")} · {s.title}</span>
              <h2 id={`h-${s.id}`}>{s.heading}</h2>
              <p className="description">{s.subtitle}</p>
              {s.blocks.map((b) => b.collapsible ? (
                <details key={b.id} className="more" open={filledIn(b.id) > 0}>
                  <summary>{b.title}<span className="count">{filledIn(b.id) ? `${filledIn(b.id)} renseigné(s)` : "facultatif"}</span></summary>
                  {blockContent(b.id)}
                </details>
              ) : (
                <div key={b.id}>
                  {b.title && <div className="block-title">{b.title}</div>}
                  {blockContent(b.id)}
                </div>
              ))}
              {s.id === "historique" && (
                <div className="histbox">
                  <strong>BEWE historique</strong>
                  <div className="kv">
                    <span>Texte source</span><span>{m.bewe_legacy_raw ?? "—"}</span>
                    <span>Total historique exact retenu</span><span>{m.bewe_total_historical ?? "aucun"}</span>
                    <span>Tranche d'origine (ancienne classe)</span><span>{m.bewe_historical_band ? `${m.bewe_historical_band[0]}–${m.bewe_historical_band[1]}` : "—"}</span>
                    <span>Sextants</span><span>non documentés (aucun reconstruit)</span>
                  </div>
                  {enc.flags.length > 0 && (
                    <>
                      <div className="block-title">Anomalies de reprise</div>
                      <div className="chips" style={{ marginTop: 8 }}>
                        {enc.flags.map((f) => <span key={f.id} className={`chip ${f.status === "open" ? "warn" : f.status === "accepted" ? "" : "muted"}`} title={f.detail ?? ""}>{f.label} · {f.status === "accepted" ? "exception validée" : f.status === "resolved" ? "résolue" : "ouverte"}</span>)}
                      </div>
                    </>
                  )}
                </div>
              )}
            </section>
          ))}

          <div className="footer-actions">
            <span className="small muted">{m.status === "validated" ? "Aucun document n'est produit par l'application." : "Les manques n'empêchent pas le brouillon. Terminer ne produit aucun document."}</span>
            {m.status !== "validated" && <button type="button" className="btn primary" onClick={finish}>Terminer la saisie ✓</button>}
          </div>
        </div>

        <aside className="aside" aria-label="Résumé de la consultation">
          <BeweSummary rows={enc.bewe} meta={m} />
          <div className="asidenote">
            <strong>Dans cette consultation</strong>
            <div className="minirow"><span>Vomissements</span><span>{vom ? VOMITING_LABELS[vom.value_text ?? vom.missing_reason ?? ""] ?? "—" : valueText("vomiting_legacy_code") ? VOMITING_LABELS[`legacy_${valueText("vomiting_legacy_code")}`] : "Non renseigné"}</span></div>
            <div className="minirow"><span>Protocole</span><span>{valueText("prevention_protocol") ? PROTOCOL_LABELS[valueText("prevention_protocol")!] : valueText("prevention_legacy") ? PROTOCOL_LABELS[valueText("prevention_legacy")!] : "Non renseigné"}</span></div>
            <div className="minirow"><span>HBD</span><span>{valueText("hbd_teaching") === "done" ? "Réalisé" : valueText("hbd_teaching") === "not_done" ? "Non réalisé" : "Non renseigné"}</span></div>
            <div className="minirow"><span>Boissons</span><span>{valueText("acidic_drinks_status") === "none_reported" ? "Aucune" : enc.exposures.filter((x) => x.grp === "drink").length || "Non renseigné"}</span></div>
            <div className="minirow"><span>Aliments</span><span>{valueText("acidic_foods_status") === "none_reported" ? "Aucun" : enc.exposures.filter((x) => x.grp === "food").length || "Non renseigné"}</span></div>
          </div>
          {clinical && <button type="button" className="btn small" onClick={() => setIdentityOpen(true)}>Identité clinique…</button>}
          {others.length > 1 && (
            <div className="asidenote">
              <strong>Autres consultations du dossier</strong>
              {others.filter((o) => o.encounter_id !== m.id).map((o) => (
                <div key={o.encounter_id} className="minirow"><button type="button" className="link" onClick={() => onOpenEncounter(o.encounter_id)}>{fmtClinicalDate(o.visit_date)}</button><span>{STATUS_LABELS[o.status]}</span></div>
              ))}
            </div>
          )}
          {m.status === "validated" && !legacy && (
            <button type="button" className="btn small" onClick={async () => { try { const f = await call<EncounterFull>("new_encounter", { patientId: m.patient_id }); onOpenEncounter(f.meta.id); } catch (e) { setNotice({ kind: "error", text: errMessage(e) }); } }}>Nouvelle consultation pour ce dossier</button>
          )}
        </aside>
      </div>

      {recap && (
        <Modal title="Terminer la saisie" onClose={() => setRecap(null)} actions={<>
          <button type="button" className="btn" onClick={() => setRecap(null)}>Compléter la saisie</button>
          <button type="button" className="btn primary" onClick={doValidate}>{recap.missing_essentials.length || recap.unconfirmed_prevention.length ? "Terminer avec ces manques" : "Terminer la saisie"}</button>
        </>}>
          <p className="small muted">Récapitulatif interne. Aucun compte rendu, courrier, ordonnance, PDF ni impression ne sera produit.</p>
          <div className="kv">
            <span>BEWE</span><span>{recap.bewe_total !== null ? `${recap.bewe_total}/18 · classe ${recap.bewe_category?.replace("-", "–")}` : `${recap.bewe_filled}/6 sextants, pas de total`}</span>
            <span>Protocole</span><span>{recap.protocol ? PROTOCOL_LABELS[recap.protocol] : "Non renseigné"}</span>
            <span>Enseignement HBD</span><span>{recap.hbd === "done" ? "Réalisé" : recap.hbd === "not_done" ? "Non réalisé" : "Non renseigné"}</span>
          </div>
          {recap.missing_essentials.length > 0 && (<>
            <h3 style={{ marginTop: 16 }}>Éléments essentiels non renseignés</h3>
            <ul className="recap-list">{recap.missing_essentials.map((x) => <li key={x}>{x}</li>)}</ul>
            <p className="small muted">Ils resteront « non renseignés » : aucune valeur ne sera supposée.</p>
          </>)}
          {recap.unconfirmed_prevention.length > 0 && (
            <div className="banner warn" style={{ marginTop: 14 }}>
              <span className="grow">{recap.unconfirmed_prevention.length} mesure(s) proposée(s) non confirmée(s) : elles seront enregistrées comme propositions, pas comme conseils.</span>
              <button type="button" className="btn small" onClick={async () => { await guard(E.op("confirm_prevention", {})); setRecap(await call<Recap>("recap", { id: encounterId })); }}>Confirmer les mesures</button>
            </div>
          )}
        </Modal>
      )}

      {amendOpen && (
        <Modal title="Corriger une consultation validée" onClose={() => setAmendOpen(false)} actions={<>
          <button type="button" className="btn" onClick={() => setAmendOpen(false)}>Annuler</button>
          <button type="button" className="btn primary" disabled={amendReason.trim().length < 3} onClick={doAmend}>Commencer la correction</button>
        </>}>
          <p className="small muted">La version validée est conservée. Le motif accompagne chaque modification dans le journal.</p>
          <div className="field" style={{ marginTop: 12 }}>
            <div className="label"><span>Motif de la correction</span></div>
            <textarea value={amendReason} onChange={(e) => setAmendReason(e.target.value)} placeholder="Ex. erreur de saisie de l'âge" />
          </div>
        </Modal>
      )}

      {historyOpen && (
        <Modal wide title="Historique des modifications" onClose={() => setHistoryOpen(null)} actions={<button type="button" className="btn" onClick={() => setHistoryOpen(null)}>Fermer</button>}>
          <h3>Révisions validées</h3>
          <ul className="recap-list">{historyOpen.revisions.map((r) => <li key={r.revision}>Révision {r.revision} — {fmtDateTime(r.created_at)} — {r.author}{r.reason ? ` — motif : ${r.reason}` : ""}</li>)}</ul>
          <h3 style={{ marginTop: 14 }}>Journal</h3>
          <div className="tablewrap" style={{ maxHeight: 360, overflowY: "auto", boxShadow: "none" }}>
            <table>
              <thead><tr><th>Date</th><th>Action</th><th>Champ</th><th>Avant</th><th>Après</th><th>Motif</th><th>Auteur</th></tr></thead>
              <tbody>{historyOpen.audit.slice().reverse().map((a, i) => (
                <tr key={i}><td>{fmtDateTime(a.at)}</td><td>{a.kind}</td><td>{a.field ?? ""}</td><td className="small">{short(a.old_value)}</td><td className="small">{short(a.new_value)}</td><td>{a.reason ?? ""}</td><td>{a.author}</td></tr>
              ))}</tbody>
            </table>
          </div>
        </Modal>
      )}

      {identityOpen && <IdentityModal patientId={m.patient_id} identity={enc.identity} onClose={() => setIdentityOpen(false)} onSaved={() => { setIdentityOpen(false); void E.reload(); }} />}
    </div>
  );
}

function short(v: string | null): string {
  if (!v) return "";
  try {
    const o = JSON.parse(v);
    if (o && typeof o === "object" && "field" in o) return String(o.value_text ?? o.value_num ?? (o.missing_reason ? `manque : ${o.missing_reason}` : ""));
  } catch { /* texte brut */ }
  return v.length > 60 ? `${v.slice(0, 60)}…` : v;
}

function IdentityModal({ patientId, identity, onClose, onSaved }: { patientId: string; identity: Identity | null; onClose: () => void; onSaved: () => void }) {
  const [ln, setLn] = useState(identity?.last_name ?? "");
  const [fn, setFn] = useState(identity?.first_name ?? "");
  const [ipp, setIpp] = useState(identity?.hospital_id ?? "");
  const [err, setErr] = useState<string | null>(null);
  return (
    <Modal title="Identité clinique (espace restreint)" onClose={onClose} actions={<>
      <button type="button" className="btn" onClick={onClose}>Annuler</button>
      <button type="button" className="btn primary" onClick={async () => { try { await call("update_identity", { patientId, identity: { last_name: ln, first_name: fn, hospital_id: ipp } }); onSaved(); } catch (e) { setErr(errMessage(e)); } }}>Enregistrer</button>
    </>}>
      <p className="small muted">Facultative, chiffrée, jamais exportée. Sert au rapprochement avec les anciens dossiers ; aucune fusion automatique sur le nom.</p>
      <div className="fields">
        <div className="field"><div className="label"><span>Nom</span></div><input value={ln} onChange={(e) => setLn(e.target.value)} /></div>
        <div className="field"><div className="label"><span>Prénom</span></div><input value={fn} onChange={(e) => setFn(e.target.value)} /></div>
        <div className="field wide"><div className="label"><span>Identifiant hospitalier (si autorisé)</span></div><input value={ipp} onChange={(e) => setIpp(e.target.value)} /></div>
      </div>
      {err && <div className="banner error" style={{ marginTop: 12 }}>{err}</div>}
    </Modal>
  );
}
