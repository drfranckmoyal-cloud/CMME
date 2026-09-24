// Chargement et sauvegarde d'une consultation.
// Les saisies partent après une courte pause (ou à la sortie du champ), dans une file ordonnée :
// l'état « Enregistré à… » n'apparaît qu'après la réponse positive du cœur natif.
import { useCallback, useEffect, useRef, useState } from "react";
import { call, errKind, errMessage } from "../api";
import type { EncounterFull, FieldInput, SaveResult, StoredValue } from "../types";

export type SaveState =
  | { kind: "idle" }
  | { kind: "dirty" }
  | { kind: "saving" }
  | { kind: "saved"; at: string }
  | { kind: "error"; message: string; errKind: string };

const DEBOUNCE_MS = 700;

function toStored(input: FieldInput): StoredValue | null {
  const base: StoredValue = { field: input.field, value_text: null, value_num: null, value_num_max: null, precision: null, date_precision: null, missing_reason: null, source_type: null, certainty: null };
  if (input.missing_reason) return { ...base, missing_reason: input.missing_reason };
  const v = input.value;
  if (v === null || v === undefined) return null;
  if (Array.isArray(v)) return v.length ? { ...base, value_text: JSON.stringify(v) } : null;
  if (typeof v === "object") {
    const o = v as { min: number; max?: number | null };
    return { ...base, value_num: o.min, value_num_max: o.max ?? null, precision: o.max != null ? "range" : input.precision ?? "exact" };
  }
  if (typeof v === "number") return { ...base, value_num: v, precision: input.precision ?? "exact" };
  return { ...base, value_text: String(v) };
}

export function useEncounter(id: string | null) {
  const [enc, setEnc] = useState<EncounterFull | null>(null);
  const [values, setValues] = useState<Map<string, StoredValue>>(new Map());
  const [loadError, setLoadError] = useState<string | null>(null);
  const [save, setSave] = useState<SaveState>({ kind: "idle" });
  const version = useRef(0);
  const pending = useRef(new Map<string, FieldInput>());
  const queue = useRef<Promise<unknown>>(Promise.resolve());
  const timer = useRef<number | undefined>(undefined);
  const idRef = useRef(id);
  idRef.current = id;

  const apply = useCallback((full: EncounterFull) => {
    setEnc(full);
    version.current = full.meta.version;
    const m = new Map<string, StoredValue>();
    full.values.forEach((v) => m.set(v.field, v));
    // Les saisies non encore enregistrées restent affichées.
    pending.current.forEach((inp) => {
      const s = toStored(inp);
      if (s) m.set(inp.field, s);
      else m.delete(inp.field);
    });
    setValues(m);
  }, []);

  const reload = useCallback(async () => {
    if (!idRef.current) return;
    try {
      const full = await call<EncounterFull>("load_encounter", { id: idRef.current });
      apply(full);
      setLoadError(null);
    } catch (e) {
      setLoadError(errMessage(e));
    }
  }, [apply]);

  useEffect(() => {
    pending.current.clear();
    setEnc(null);
    setValues(new Map());
    setSave({ kind: "idle" });
    if (id) reload();
  }, [id, reload]);

  /** Exécute une opération native dans la file, avec la version courante. */
  const run = useCallback(<T,>(op: (v: number) => Promise<T>, pickVersion: (r: T) => number | undefined, reloadAfter = false): Promise<T> => {
    const p = queue.current.then(async () => {
      setSave({ kind: "saving" });
      try {
        const r = await op(version.current);
        const nv = pickVersion(r);
        if (nv !== undefined) version.current = nv;
        if (reloadAfter) await reload();
        const at = (r as unknown as SaveResult)?.saved_at ?? new Date().toISOString();
        setSave(pending.current.size ? { kind: "dirty" } : { kind: "saved", at });
        return r;
      } catch (e) {
        setSave({ kind: "error", message: errMessage(e), errKind: errKind(e) });
        throw e;
      }
    });
    queue.current = p.catch(() => undefined);
    return p;
  }, [reload]);

  const flush = useCallback(async (): Promise<boolean> => {
    window.clearTimeout(timer.current);
    if (!idRef.current || pending.current.size === 0) {
      await queue.current;
      return true;
    }
    const inputs = [...pending.current.values()];
    pending.current.clear();
    try {
      await run((v) => call<SaveResult>("save_fields", { id: idRef.current, version: v, inputs }), (r) => r.version);
      return true;
    } catch {
      // Remettre en attente ce qui n'a pas été remplacé entre-temps : rien n'est perdu côté écran.
      inputs.forEach((i) => { if (!pending.current.has(i.field)) pending.current.set(i.field, i); });
      return false;
    }
  }, [run]);

  const setField = useCallback((input: FieldInput, immediate = false) => {
    pending.current.set(input.field, input);
    setValues((prev) => {
      const m = new Map(prev);
      const s = toStored(input);
      if (s) m.set(input.field, s);
      else m.delete(input.field);
      return m;
    });
    setSave({ kind: "dirty" });
    window.clearTimeout(timer.current);
    if (immediate) void flush();
    else timer.current = window.setTimeout(() => void flush(), DEBOUNCE_MS);
  }, [flush]);

  /** Opération structurante (BEWE, expositions, prévention…) : enregistre d'abord la saisie en cours. */
  const op = useCallback(async <T,>(cmd: string, args: Record<string, unknown>, pick: (r: T) => number | undefined = (r) => (r as unknown as SaveResult).version): Promise<T> => {
    const ok = await flush();
    if (!ok) throw { kind: "pending", message: "Des saisies précédentes ne sont pas enregistrées." };
    return run((v) => call<T>(cmd, { id: idRef.current, version: v, ...args }), pick, true);
  }, [flush, run]);

  const retry = useCallback(async () => {
    if (save.kind === "error" && save.errKind === "storage") {
      try { await call("reconnect"); } catch { /* l'erreur réapparaîtra à l'enregistrement */ }
    }
    if (pending.current.size) await flush();
    else await reload();
  }, [flush, reload, save]);

  const discardAndReload = useCallback(async () => {
    pending.current.clear();
    await reload();
    setSave({ kind: "idle" });
  }, [reload]);

  return { enc, values, save, loadError, setField, flush, op, reload, retry, discardAndReload, hasPending: () => pending.current.size > 0, setEncDirect: apply };
}
