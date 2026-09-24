// Types miroirs des structures du cœur Rust (crates/core). Les codes sont stables.

export type Kind = "choice" | "multi" | "number" | "date" | "text";
export interface Opt { code: string; label: string }
export interface Field {
  code: string; section: string; block: string; label: string; label_en: string; kind: Kind;
  options: Opt[]; exclusive: string[]; min: number | null; max: number | null; integer: boolean;
  unit: string | null; allow_range: boolean; export: boolean; legacy: boolean; essential: boolean;
  wide: boolean; hint: string | null; show_if: { field: string; values: string[] } | null;
  missing: string[]; custom: boolean;
}
export interface Block { id: string; title: string | null; collapsible: boolean }
export interface Section { id: string; number: number; title: string; heading: string; subtitle: string; blocks: Block[] }
export interface CatalogBundle {
  catalog: { form_version: string; protocol_version: string; sections: Section[]; fields: Field[] };
  sextants: { key: string; label: string; teeth: string }[];
  unassessable_reasons: Opt[];
  drinks: Opt[]; foods: Opt[];
  prevention_actions: Opt[];
  missing_reasons: Opt[];
  import_targets: Opt[];
}

export interface StoredValue {
  field: string; value_text: string | null; value_num: number | null; value_num_max: number | null;
  precision: string | null; date_precision: string | null; missing_reason: string | null;
  source_type: string | null; certainty: string | null;
}
export interface FieldInput {
  field: string; value?: unknown; missing_reason?: string | null; precision?: string | null;
}
export interface EncounterMeta {
  id: string; patient_id: string; patient_code: string; collection_mode: string; form_version: string;
  status: "draft" | "validated" | "amending"; revision: number; version: number;
  visit_date: string | null; visit_date_precision: string | null; service_code: string | null; service_label_source: string | null;
  examiner: string | null; bewe_total_derived: number | null; bewe_total_historical: number | null;
  bewe_historical_band: [number, number] | null; bewe_legacy_raw: string | null;
  created_at: string; updated_at: string; validated_at: string | null; is_demo: boolean;
}
export interface SextantRow { sextant: string; score: number | null; missing_reason: string | null; unassessable_reason: string | null }
export interface ExposureRow { grp: string; category: string; temporality: string | null; freq_min: number | null; freq_max: number | null; freq_unit: string | null; quantity_text: string | null }
export interface PreventionRow {
  action_type: string; decision: string | null; proposed_by: string | null; confirmed: boolean;
  st_already_used: boolean; st_done_in_consultation: boolean; st_given_today: boolean; st_refused: boolean;
  product_text: string | null; route: string | null; frequency_text: string | null; tray_minutes: number | null; note: string | null;
}
export interface FlagRow { id: number; flag: string; label: string; detail: string | null; status: string; resolution: string | null }
export interface Identity { last_name: string | null; first_name: string | null; hospital_id: string | null }
export interface EncounterFull {
  meta: EncounterMeta; identity: Identity | null; values: StoredValue[]; bewe: SextantRow[];
  exposures: ExposureRow[]; prevention: PreventionRow[]; flags: FlagRow[];
}
export interface SaveResult { version: number; saved_at: string; bewe_total_derived: number | null }
export interface Recap {
  bewe_filled: number; bewe_total: number | null; bewe_category: string | null; missing_essentials: string[];
  unconfirmed_prevention: string[]; hbd: string | null; protocol: string | null; open_flags: number;
}
export interface BeweAnalysis { value: number | null; origin: string; sextants_filled: number }
export interface EncounterRow {
  encounter_id: string; patient_id: string; patient_code: string; display_name: string | null; collection_mode: string;
  status: string; revision: number; visit_date: string | null; visit_date_precision: string | null; service_code: string | null;
  age_years: number | null; age_band: [number | null, number | null] | null; age_missing: string | null;
  vomiting: string | null; vomiting_origin: string | null; bewe: BeweAnalysis; bewe_category: string | null;
  protocol: string | null; protocol_origin: string | null; hbd: string | null; open_flags: number; updated_at: string; is_demo: boolean;
}
export interface Filter {
  text?: string; date_from?: string; date_to?: string; services?: string[]; collection_mode?: string;
  status?: string; bewe?: string; anomalies_only?: boolean;
}
export interface Status {
  app_version: string; unlocked: boolean; profile: string | null; clinique_exists: boolean; demo_exists: boolean;
  practitioner: string | null; cipher_version: string | null; last_backup_at: string | null; data_dir: string;
  keychain: string; macos_version: string | null;
}
export interface Proportion { k: number; n: number; pct: number | null; ci_low: number | null; ci_high: number | null }
export interface Distribution { n: number; median: number | null; q1: number | null; q3: number | null; min: number | null; max: number | null; mean: number | null; sd: number | null }
export interface Stats {
  filters: string[]; n_visits: number; n_patients: number; n_legacy: number; n_prospective: number;
  bewe: { n_visits: number; n_analysable: number; origins: Record<string, number>; distribution: Distribution; categories: [string, number][]; gt0: Proportion; ge9: Proportion; ge14: Proportion };
  age_exact: Distribution; age_band_only: number; age_missing: number; services: Record<string, number>;
  vomiting_prospective: Record<string, number>; vomiting_legacy: Record<string, number>;
  protocol_prospective: Record<string, number>; prevention_legacy: Record<string, number>;
  hbd_done: Proportion;
  completeness: { field: string; label: string; n: number; available: number; missing: Record<string, number> }[];
}
