-- Schéma CMME v1. Toutes les données cliniques vivent dans cette base chiffrée (SQLCipher).

CREATE TABLE app_setting (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE patient (
  id TEXT PRIMARY KEY,
  code TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL,
  merged_into TEXT REFERENCES patient(id),
  is_demo INTEGER NOT NULL DEFAULT 0 CHECK (is_demo IN (0,1))
);

-- Identité clinique : espace restreint, jamais exportée.
CREATE TABLE patient_identity (
  patient_id TEXT PRIMARY KEY REFERENCES patient(id),
  last_name TEXT,
  first_name TEXT,
  full_name_raw TEXT,
  hospital_id TEXT,
  identity_key TEXT
);
CREATE INDEX idx_identity_key ON patient_identity(identity_key);

CREATE TABLE encounter (
  id TEXT PRIMARY KEY,
  patient_id TEXT NOT NULL REFERENCES patient(id),
  collection_mode TEXT NOT NULL CHECK (collection_mode IN ('legacy_retrospective','structured_prospective')),
  form_version TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('draft','validated','amending')),
  revision INTEGER NOT NULL DEFAULT 0,
  version INTEGER NOT NULL DEFAULT 1,
  visit_date TEXT,
  visit_date_precision TEXT CHECK (visit_date_precision IN ('day','month','year')),
  service_code TEXT,
  service_label_source TEXT,
  examiner TEXT,
  bewe_total_derived INTEGER CHECK (bewe_total_derived BETWEEN 0 AND 18),
  bewe_total_historical INTEGER CHECK (bewe_total_historical BETWEEN 0 AND 18),
  bewe_historical_band_min INTEGER,
  bewe_historical_band_max INTEGER,
  bewe_legacy_raw TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  validated_at TEXT,
  source_record_id TEXT
);
CREATE INDEX idx_encounter_patient ON encounter(patient_id);

CREATE TABLE field_value (
  encounter_id TEXT NOT NULL REFERENCES encounter(id),
  field TEXT NOT NULL,
  value_text TEXT,
  value_num REAL,
  value_num_max REAL,
  precision TEXT CHECK (precision IN ('exact','estimated','range')),
  date_precision TEXT CHECK (date_precision IN ('day','month','year')),
  missing_reason TEXT CHECK (missing_reason IN ('not_recorded','not_asked','unknown','declined','not_assessable','not_applicable','ambiguous_source')),
  source_type TEXT CHECK (source_type IN ('patient_report','medical_record','clinical_exam','instrument_measurement','legacy_import','clinician_adjudication')),
  source_ref TEXT,
  certainty TEXT CHECK (certainty IN ('documented','reported','suspected','unresolved')),
  recorded_at TEXT NOT NULL,
  author TEXT,
  PRIMARY KEY (encounter_id, field),
  -- Valeur et raison de manque sont mutuellement exclusives, et l'une des deux est présente.
  CHECK ((missing_reason IS NULL) = (value_text IS NOT NULL OR value_num IS NOT NULL)),
  CHECK (value_num_max IS NULL OR value_num_max >= value_num)
);

CREATE TABLE bewe_sextant (
  encounter_id TEXT NOT NULL REFERENCES encounter(id),
  sextant TEXT NOT NULL CHECK (sextant IN ('upper_right','upper_anterior','upper_left','lower_left','lower_anterior','lower_right')),
  score INTEGER CHECK (score IN (0,1,2,3)),
  missing_reason TEXT CHECK (missing_reason IN ('not_assessable','unknown')),
  unassessable_reason TEXT CHECK (unassessable_reason IN ('edentulous','restoration','limited_access','other')),
  recorded_at TEXT NOT NULL,
  PRIMARY KEY (encounter_id, sextant),
  CHECK ((score IS NULL) <> (missing_reason IS NULL))
);

CREATE TABLE exposure (
  encounter_id TEXT NOT NULL REFERENCES encounter(id),
  grp TEXT NOT NULL CHECK (grp IN ('drink','food')),
  category TEXT NOT NULL,
  temporality TEXT CHECK (temporality IN ('current','past')),
  freq_min REAL CHECK (freq_min >= 0),
  freq_max REAL,
  freq_unit TEXT CHECK (freq_unit IN ('day','week','month')),
  quantity_text TEXT,
  recorded_at TEXT NOT NULL,
  PRIMARY KEY (encounter_id, grp, category),
  CHECK (freq_max IS NULL OR freq_max >= freq_min)
);

CREATE TABLE prevention_action (
  encounter_id TEXT NOT NULL REFERENCES encounter(id),
  action_type TEXT NOT NULL CHECK (action_type IN ('anti_erosion_toothpaste','anti_erosion_rinse','tooth_mousse','vomiting_semirigid_tray','other')),
  decision TEXT CHECK (decision IN ('advised','not_advised','not_applicable')),
  proposed_by TEXT CHECK (proposed_by IN ('moderate','advanced')),
  confirmed INTEGER NOT NULL DEFAULT 0 CHECK (confirmed IN (0,1)),
  st_already_used INTEGER NOT NULL DEFAULT 0 CHECK (st_already_used IN (0,1)),
  st_done_in_consultation INTEGER NOT NULL DEFAULT 0 CHECK (st_done_in_consultation IN (0,1)),
  st_given_today INTEGER NOT NULL DEFAULT 0 CHECK (st_given_today IN (0,1)),
  st_refused INTEGER NOT NULL DEFAULT 0 CHECK (st_refused IN (0,1)),
  product_text TEXT,
  route TEXT CHECK (route IN ('direct','tray','both','unspecified')),
  frequency_text TEXT,
  tray_minutes INTEGER CHECK (tray_minutes BETWEEN 1 AND 240),
  note TEXT,
  protocol_version TEXT,
  recorded_at TEXT NOT NULL,
  PRIMARY KEY (encounter_id, action_type),
  -- La durée en gouttière ne s'applique jamais à la voie directe.
  CHECK (tray_minutes IS NULL OR route IN ('tray','both'))
);

CREATE TABLE encounter_revision (
  encounter_id TEXT NOT NULL REFERENCES encounter(id),
  revision INTEGER NOT NULL,
  snapshot TEXT NOT NULL,
  reason TEXT,
  created_at TEXT NOT NULL,
  author TEXT,
  PRIMARY KEY (encounter_id, revision)
);

CREATE TABLE encounter_flag (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id TEXT NOT NULL REFERENCES encounter(id),
  flag TEXT NOT NULL,
  detail TEXT,
  status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open','accepted','resolved')),
  resolution TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE audit_event (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  at TEXT NOT NULL,
  kind TEXT NOT NULL,
  entity TEXT NOT NULL,
  entity_id TEXT,
  field TEXT,
  old_value TEXT,
  new_value TEXT,
  reason TEXT,
  author TEXT
);
CREATE INDEX idx_audit_entity ON audit_event(entity_id);

-- Reprise : source brute immuable, propositions, décisions.
CREATE TABLE source_document (
  id TEXT PRIMARY KEY,
  sha256 TEXT NOT NULL UNIQUE,
  filename TEXT NOT NULL,
  format TEXT NOT NULL,
  size_bytes INTEGER NOT NULL,
  imported_at TEXT NOT NULL,
  parser_version TEXT NOT NULL,
  content BLOB NOT NULL,
  config TEXT
);

CREATE TABLE source_record (
  id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL REFERENCES source_document(id),
  sheet TEXT NOT NULL,
  row_number INTEGER NOT NULL,
  cells TEXT NOT NULL,
  identity_key TEXT,
  status TEXT NOT NULL CHECK (status IN ('pending','excluded','validated','unchanged')),
  exclusion_reason TEXT,
  flags TEXT NOT NULL DEFAULT '[]',
  diff TEXT,
  link_patient_id TEXT,
  link_decision TEXT CHECK (link_decision IN ('same_patient','different_person','new')),
  encounter_id TEXT,
  previous_record_id TEXT,
  UNIQUE (document_id, sheet, row_number)
);

CREATE TABLE import_proposal (
  record_id TEXT NOT NULL REFERENCES source_record(id),
  target TEXT NOT NULL,
  proposed TEXT,
  rule TEXT NOT NULL,
  decision TEXT NOT NULL DEFAULT 'pending' CHECK (decision IN ('pending','accepted','corrected','unavailable')),
  decided_value TEXT,
  justification TEXT,
  decided_at TEXT,
  PRIMARY KEY (record_id, target)
);

CREATE TABLE merge_event (
  id TEXT PRIMARY KEY,
  source_patient_id TEXT NOT NULL,
  target_patient_id TEXT NOT NULL,
  moved_encounters TEXT NOT NULL,
  reason TEXT NOT NULL,
  at TEXT NOT NULL,
  undone_at TEXT
);

CREATE TABLE research_project (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  period_start TEXT,
  period_end TEXT,
  include_legacy INTEGER NOT NULL DEFAULT 1,
  include_prospective INTEGER NOT NULL DEFAULT 1,
  exclude_open_anomalies INTEGER NOT NULL DEFAULT 1,
  plan_version TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE research_eligibility (
  project_id TEXT NOT NULL REFERENCES research_project(id),
  patient_id TEXT NOT NULL REFERENCES patient(id),
  status TEXT NOT NULL CHECK (status IN ('excluded')),
  reason TEXT NOT NULL,
  at TEXT NOT NULL,
  PRIMARY KEY (project_id, patient_id)
);

CREATE TABLE study_id_map (
  project_id TEXT NOT NULL REFERENCES research_project(id),
  kind TEXT NOT NULL CHECK (kind IN ('patient','visit')),
  local_id TEXT NOT NULL,
  study_id TEXT NOT NULL,
  PRIMARY KEY (project_id, kind, local_id),
  UNIQUE (project_id, study_id)
);

CREATE TABLE export_snapshot (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES research_project(id),
  created_at TEXT NOT NULL,
  manifest TEXT NOT NULL,
  files TEXT NOT NULL
);
