//! Reprise de l'ancien recueil (XLSX, CSV) en trois couches : source brute immuable,
//! propositions de normalisation, données validées. Aucune fusion ni extraction automatique.

use crate::domain::bewe;
use crate::domain::catalog;
use crate::domain::legacy::{self, identity_key, Normalized};
use crate::domain::values::{self, FieldInput};
use crate::error::{CoreError, Result};
use crate::store::encounters::{insert_encounter, load_full, write_identity, write_value, IdentityInput};
use crate::store::{audit_on, new_id, now, today, Store};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

pub const PARSER_VERSION: &str = "cmme-import-1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheet {
    pub name: String,
    pub rows: Vec<Vec<String>>,
    pub merged_cells: usize,
}

#[derive(Debug, Clone)]
pub struct ReadFile {
    pub bytes: Vec<u8>,
    pub sha256: String,
    pub format: String,
    pub filename: String,
    pub sheets: Vec<Sheet>,
}

fn cell_to_string(d: &calamine::Data) -> String {
    use calamine::Data;
    match d {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                format!("{f}")
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => if *b { "VRAI".into() } else { "FAUX".into() },
        Data::DateTime(dt) => match dt.as_datetime() {
            Some(x) if x.time() == chrono::NaiveTime::MIN => x.date().format("%Y-%m-%d").to_string(),
            Some(x) => x.format("%Y-%m-%d %H:%M").to_string(),
            None => dt.to_string(),
        },
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#ERREUR {e:?}"),
    }
}

fn sniff_delimiter(first_line: &str) -> u8 {
    let count = |c: char| first_line.matches(c).count();
    [(';', count(';')), ('\t', count('\t')), (',', count(','))].into_iter().max_by_key(|x| x.1).map(|x| x.0 as u8).unwrap_or(b';')
}

pub fn read_file(path: &Path) -> Result<ReadFile> {
    let bytes = std::fs::read(path)?;
    let filename = path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
    let sha256 = hex::encode(Sha256::digest(&bytes));
    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    let (format, sheets) = match ext.as_str() {
        "csv" | "txt" | "tsv" => {
            let text = std::str::from_utf8(&bytes).map_err(|_| CoreError::Refused("le fichier CSV n'est pas en UTF-8 : réenregistrez-le en « CSV UTF-8 »".into()))?;
            let text = text.strip_prefix('\u{feff}').unwrap_or(text);
            let delim = sniff_delimiter(text.lines().next().unwrap_or(""));
            let mut rdr = csv::ReaderBuilder::new().delimiter(delim).has_headers(false).flexible(true).from_reader(text.as_bytes());
            let mut rows = vec![];
            for rec in rdr.records() {
                let rec = rec.map_err(|e| CoreError::Refused(format!("CSV illisible : {e}")))?;
                rows.push(rec.iter().map(|c| c.to_string()).collect());
            }
            // Nom de feuille fixe : deux versions d'un même CSV restent comparables ligne à ligne.
            ("csv".to_string(), vec![Sheet { name: "CSV".into(), rows, merged_cells: 0 }])
        }
        "xlsx" | "xlsm" | "xls" | "ods" => {
            use calamine::Reader;
            let mut wb = calamine::open_workbook_auto_from_rs(std::io::Cursor::new(bytes.clone())).map_err(|e| CoreError::Refused(format!("classeur illisible : {e}")))?;
            let names = wb.sheet_names().to_vec();
            let mut sheets = vec![];
            for name in names {
                let range = match wb.worksheet_range(&name) {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let (r0, c0) = range.start().unwrap_or((0, 0));
                let mut rows: Vec<Vec<String>> = vec![vec![]; r0 as usize];
                for row in range.rows() {
                    let mut v: Vec<String> = vec![String::new(); c0 as usize];
                    v.extend(row.iter().map(cell_to_string));
                    rows.push(v);
                }
                let merged = match &mut wb {
                    calamine::Sheets::Xlsx(x) => {
                        let _ = x.load_merged_regions();
                        x.merged_regions_by_sheet(&name).len()
                    }
                    _ => 0,
                };
                sheets.push(Sheet { name, rows, merged_cells: merged });
            }
            (ext.clone(), sheets)
        }
        "numbers" | "pages" => return Err(CoreError::Refused("format Apple non pris en charge : exportez d'abord le classeur en XLSX depuis Numbers (Fichier › Exporter vers › Excel)".into())),
        _ => return Err(CoreError::Refused("format non pris en charge (XLSX ou CSV attendus)".into())),
    };
    Ok(ReadFile { bytes, sha256, format, filename, sheets })
}

// ---------------------------------------------------------------- Mapping

pub const TARGETS: [(&str, &str); 19] = [
    ("ignore", "Ignorer (reste dans la source brute)"),
    ("patient_code", "Code du dossier"),
    ("full_name", "Nom et prénom"),
    ("last_name", "Nom"),
    ("first_name", "Prénom"),
    ("hospital_id", "Identifiant hospitalier"),
    ("visit_date", "Date de consultation"),
    ("service", "Service"),
    ("age", "Âge ou classe d'âge"),
    ("sex", "Sexe"),
    ("vomiting", "Vomissements (oui/non)"),
    ("diet", "Alimentation (ancien codage)"),
    ("hygiene", "Hygiène"),
    ("bewe", "BEWE"),
    ("dmft", "CAO"),
    ("care", "Soins"),
    ("prevention", "Prévention"),
    ("other_wear", "Autres usures"),
    ("observations", "Observations"),
];

pub fn suggest_target(header: &str) -> &'static str {
    let mut h = legacy::simplify(header);
    for suffix in [" brut", " source", " fictif", " source fictif"] {
        if let Some(x) = h.strip_suffix(suffix) {
            h = x.to_string();
        }
    }
    let h = h.as_str();
    let starts = |p: &str| h == p || h.starts_with(&format!("{p} "));
    if h.contains("fdi") {
        "ignore"
    } else if starts("code") || h == "id" || starts("identifiant") && !h.contains("hosp") || h.contains("dossier") {
        "patient_code"
    } else if h.contains("ipp") || h.contains("hospitalier") {
        "hospital_id"
    } else if (h.split(' ').any(|w| w == "nom") && h.split(' ').any(|w| w == "prenom" || w == "prenoms")) || h == "patient" || h == "identite" {
        "full_name"
    } else if starts("prenom") {
        "first_name"
    } else if starts("nom") {
        "last_name"
    } else if h.contains("date") {
        "visit_date"
    } else if starts("service") || starts("secteur") {
        "service"
    } else if starts("age") || h.contains("classe d age") {
        "age"
    } else if starts("sexe") || starts("genre") {
        "sex"
    } else if h.starts_with("vomis") {
        "vomiting"
    } else if h.starts_with("aliment") || h.starts_with("boisson") || h.contains("risque") {
        "diet"
    } else if h.starts_with("hygiene") {
        "hygiene"
    } else if h.contains("bewe") {
        "bewe"
    } else if starts("cao") || h == "dmft" {
        "dmft"
    } else if h.starts_with("soin") || h.starts_with("besoin") {
        "care"
    } else if h.starts_with("prevention") || h.starts_with("protocole") {
        "prevention"
    } else if h.contains("usure") {
        "other_wear"
    } else if h.starts_with("observation") || h.starts_with("remarque") || h.starts_with("commentaire") {
        "observations"
    } else {
        "ignore"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetPreview {
    pub name: String,
    pub n_rows: usize,
    pub header_row: usize,
    pub headers: Vec<String>,
    pub sample: Vec<Vec<String>>,
    pub suggested_kind: String,
    pub suggested_mapping: Vec<String>,
    pub merged_cells: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlreadyImported {
    pub imported_at: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub filename: String,
    pub sha256: String,
    pub format: String,
    pub size_bytes: usize,
    pub already_imported: Option<AlreadyImported>,
    pub other_documents: i64,
    pub sheets: Vec<SheetPreview>,
    pub targets: Vec<(String, String)>,
}

fn non_empty(row: &[String]) -> usize {
    row.iter().filter(|c| !c.trim().is_empty()).count()
}

fn guess_header(rows: &[Vec<String>]) -> usize {
    for (i, r) in rows.iter().enumerate().take(15) {
        let filled: Vec<&String> = r.iter().filter(|c| !c.trim().is_empty()).collect();
        if filled.len() >= 2 && filled.iter().filter(|c| c.trim().parse::<f64>().is_err()).count() * 2 >= filled.len() {
            return i;
        }
    }
    0
}

pub fn preview(store: &Store, path: &Path) -> Result<ImportPreview> {
    let f = read_file(path)?;
    let already = store
        .conn
        .query_row("SELECT imported_at, filename FROM source_document WHERE sha256 = ?1", [&f.sha256], |r| Ok(AlreadyImported { imported_at: r.get(0)?, filename: r.get(1)? }))
        .optional()?;
    let other: i64 = store.conn.query_row("SELECT count(*) FROM source_document", [], |r| r.get(0))?;
    let sheets = f
        .sheets
        .iter()
        .map(|s| {
            let h = guess_header(&s.rows);
            let headers: Vec<String> = s.rows.get(h).cloned().unwrap_or_default();
            let lname = legacy::simplify(&s.name);
            let data_rows = s.rows.iter().skip(h + 1).filter(|r| non_empty(r) > 0).count();
            let summary = lname.split(' ').any(|w| matches!(w, "synthese" | "total" | "totaux" | "bilan" | "resume"))
                || headers.iter().any(|x| matches!(legacy::simplify(x).as_str(), "total" | "moyenne" | "effectif"));
            let kind = if data_rows == 0 {
                "ignore"
            } else if summary {
                "summary"
            } else {
                "individual"
            };
            SheetPreview {
                name: s.name.clone(),
                n_rows: data_rows,
                header_row: h,
                suggested_mapping: headers.iter().map(|x| suggest_target(x).to_string()).collect(),
                headers,
                sample: s.rows.iter().skip(h + 1).filter(|r| non_empty(r) > 0).take(8).cloned().collect(),
                suggested_kind: kind.into(),
                merged_cells: s.merged_cells,
            }
        })
        .collect();
    Ok(ImportPreview {
        filename: f.filename,
        sha256: f.sha256,
        format: f.format,
        size_bytes: f.bytes.len(),
        already_imported: already,
        other_documents: other,
        sheets,
        targets: TARGETS.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetConfig {
    pub name: String,
    /// `individual` (une ligne = une consultation), `summary` (tableau de synthèse : jamais de patient), `ignore`.
    pub kind: String,
    pub header_row: usize,
    pub mapping: Vec<String>,
    #[serde(default)]
    pub service_from_sheet: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StageSummary {
    pub document_id: String,
    pub rows_read: usize,
    pub pending: usize,
    pub ready: usize,
    pub needs_review: usize,
    pub unchanged: usize,
    pub excluded: HashMap<String, usize>,
    pub summary_sheets: Vec<String>,
}

fn normalize_row(map: &HashMap<&str, String>, sheet: &str, service_from_sheet: bool) -> Normalized {
    let mut n = Normalized::default();
    let mut push = |x: Normalized| {
        n.proposals.extend(x.proposals);
        n.flags.extend(x.flags);
    };
    let get = |t: &str| map.get(t).cloned();
    if let Some(v) = get("patient_code").filter(|v| !v.trim().is_empty()) {
        push(Normalized { proposals: vec![legacy::Proposal { target: "patient_code".into(), value: json!(v.trim()), rule: "raw_code" }], flags: vec![] });
    }
    match get("visit_date") {
        Some(v) => push(legacy::visit_date(&v, today())),
        None => push(Normalized { proposals: vec![], flags: vec!["date_missing"] }),
    }
    let service_raw = get("service").filter(|s| !s.trim().is_empty()).or_else(|| service_from_sheet.then(|| sheet.to_string()));
    if let Some(s) = service_raw {
        let rule = if get("service").map(|x| !x.trim().is_empty()).unwrap_or(false) { "legacy_service_label" } else { "service_from_sheet_name" };
        let mut x = Normalized::default();
        x.proposals.push(legacy::Proposal { target: "service_label_source".into(), value: json!(s), rule: "raw_kept" });
        if let Some(code) = legacy::service(&s) {
            x.proposals.push(legacy::Proposal { target: "service_code".into(), value: json!(code), rule });
        } else {
            x.proposals.push(legacy::Proposal { target: "service_code".into(), value: json!("autre"), rule });
        }
        push(x);
    }
    if let Some(v) = get("age") {
        let mut a = legacy::age(&v);
        if a.flags.iter().any(|f| *f == "age_band_mismatch" || *f == "age_unparseable") && !a.proposals.iter().any(|p| p.target == "age_years") {
            a.proposals.push(legacy::Proposal { target: "age_years".into(), value: Value::Null, rule: "no_exact_age" });
        }
        push(a);
    }
    if let Some(v) = get("sex") {
        let mut x = legacy::sex(&v);
        if !x.flags.is_empty() {
            x.proposals.push(legacy::Proposal { target: "sex_recorded".into(), value: Value::Null, rule: "unrecognized" });
        }
        push(x);
    }
    let with_null = |mut x: Normalized, target: &str| {
        if !x.flags.is_empty() && !x.proposals.iter().any(|p| p.target == target) {
            x.proposals.push(legacy::Proposal { target: target.into(), value: Value::Null, rule: "unrecognized" });
        }
        x
    };
    if let Some(v) = get("vomiting") {
        let x = legacy::vomiting(&v);
        push(if x.flags.contains(&"vomiting_ambiguous") { with_null(x, "vomiting_legacy_code") } else { x });
    }
    if let Some(v) = get("diet") {
        push(legacy::diet(&v));
    }
    if let Some(v) = get("hygiene") {
        push(with_null(legacy::hygiene(&v), "hygiene_legacy"));
    }
    if let Some(v) = get("care") {
        push(with_null(legacy::care(&v), "care_legacy"));
    }
    if let Some(v) = get("prevention") {
        push(legacy::prevention(&v));
    }
    if let Some(v) = get("other_wear") {
        push(with_null(legacy::other_wear(&v), "other_wear_legacy"));
    }
    if let Some(v) = get("dmft") {
        push(with_null(legacy::dmft(&v), "dmft_total_historical"));
    }
    if let Some(v) = get("observations").filter(|v| !v.trim().is_empty()) {
        push(Normalized { proposals: vec![legacy::Proposal { target: "observations_legacy".into(), value: json!(v), rule: "raw_kept" }], flags: vec![] });
    }
    if let Some(v) = get("bewe") {
        let p = bewe::parse_legacy(&v);
        let mut x = Normalized::default();
        if !v.trim().is_empty() {
            x.proposals.push(legacy::Proposal { target: "bewe_legacy_raw".into(), value: json!(v), rule: "raw_kept" });
        }
        x.proposals.push(legacy::Proposal { target: "bewe_total_historical".into(), value: p.proposed_total.map(|t| json!(t)).unwrap_or(Value::Null), rule: "legacy_bewe_grammar" });
        if let Some((a, b)) = p.band {
            x.proposals.push(legacy::Proposal { target: "bewe_band".into(), value: json!([a, b]), rule: "legacy_bewe_grammar" });
        }
        if let Some(f) = p.flag {
            x.flags.push(f);
        }
        push(x);
    }
    n
}

/// Cible de résolution d'une anomalie : une proposition à trancher, ou une décision de rattachement.
fn flag_target(flag: &str) -> Option<&'static str> {
    Some(match flag {
        "not_recorded" | "band_only" | "band_mismatch" | "ambiguous_exact_value" | "out_of_range" | "unparseable" => "bewe_total_historical",
        "date_suspect" | "date_unparseable" => "visit_date",
        "age_band_mismatch" | "age_unparseable" => "age_years",
        "vomiting_ambiguous" => "vomiting_legacy_code",
        "sex_ambiguous" => "sex_recorded",
        "hygiene_out_of_category" => "hygiene_legacy",
        "care_out_of_category" => "care_legacy",
        "other_wear_ambiguous" => "other_wear_legacy",
        "dmft_to_review" => "dmft_total_historical",
        _ => return None,
    })
}

const LINK_FLAGS: [&str; 4] = ["duplicate_candidate", "duplicate_exact", "no_identity", "changed_since_previous_version"];

impl Store {
    /// Étape 1 : copie chiffrée de la source, lignes brutes, propositions et anomalies. Rien n'est validé.
    pub fn import_stage(&mut self, path: &Path, config: Vec<SheetConfig>) -> Result<StageSummary> {
        let f = read_file(path)?;
        if let Some((at, name)) = self.conn.query_row("SELECT imported_at, filename FROM source_document WHERE sha256 = ?1", [&f.sha256], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))).optional()? {
            return Err(CoreError::Refused(format!("ce fichier exact a déjà été importé le {} (« {name} ») : aucune duplication", &at[..10])));
        }
        let author = self.author.clone();
        let tx = self.conn.transaction()?;
        let doc_id = new_id();
        tx.execute(
            "INSERT INTO source_document(id, sha256, filename, format, size_bytes, imported_at, parser_version, content, config) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![doc_id, f.sha256, f.filename, f.format, f.bytes.len() as i64, now(), PARSER_VERSION, f.bytes, serde_json::to_string(&config)?],
        )?;
        let mut summary = StageSummary { document_id: doc_id.clone(), ..Default::default() };
        let mut staged: Vec<(String, Option<String>, String, Vec<(String, String)>)> = vec![];
        for cfg in &config {
            let Some(sheet) = f.sheets.iter().find(|s| s.name == cfg.name) else { continue };
            if cfg.kind == "summary" {
                summary.summary_sheets.push(cfg.name.clone());
                continue;
            }
            if cfg.kind != "individual" {
                continue;
            }
            let headers: Vec<String> = sheet.rows.get(cfg.header_row).cloned().unwrap_or_default();
            if cfg.mapping.len() > headers.len().max(1) + 200 {
                return Err(CoreError::validation("mapping", "nombre de colonnes incohérent"));
            }
            for valid in &cfg.mapping {
                if !TARGETS.iter().any(|(t, _)| t == valid) {
                    return Err(CoreError::validation("mapping", "cible inconnue"));
                }
            }
            for (idx, row) in sheet.rows.iter().enumerate().skip(cfg.header_row + 1) {
                if non_empty(row) == 0 {
                    continue;
                }
                summary.rows_read += 1;
                let row_number = idx + 1; // numérotation tableur, en-tête compris
                let cells: Vec<(String, String)> = row.iter().enumerate().map(|(i, v)| (headers.get(i).cloned().unwrap_or_else(|| format!("Colonne {}", i + 1)), v.clone())).collect();
                let rid = new_id();
                let mut excluded: Option<&str> = None;
                if row.iter().map(|c| c.trim()).collect::<Vec<_>>() == headers.iter().map(|c| c.trim()).collect::<Vec<_>>() {
                    excluded = Some("repeated_header");
                } else if non_empty(row) == 1 {
                    excluded = Some("separator_row");
                }
                let mut map: HashMap<&str, String> = HashMap::new();
                for (i, t) in cfg.mapping.iter().enumerate() {
                    if t != "ignore" {
                        if let Some(v) = row.get(i) {
                            map.insert(t.as_str(), v.clone());
                        }
                    }
                }
                let name = map.get("full_name").cloned().filter(|x| !x.trim().is_empty()).or_else(|| {
                    let n = [map.get("last_name").cloned(), map.get("first_name").cloned()].into_iter().flatten().filter(|x| !x.trim().is_empty()).collect::<Vec<_>>().join(" ");
                    (!n.is_empty()).then_some(n)
                });
                let ikey = name.as_ref().map(|n| identity_key(n)).or_else(|| map.get("patient_code").filter(|c| !c.trim().is_empty()).map(|c| format!("code:{}", c.trim())));
                let norm = normalize_row(&map, &cfg.name, cfg.service_from_sheet);
                let mut flags: Vec<String> = norm.flags.iter().map(|s| s.to_string()).collect();
                if ikey.is_none() && excluded.is_none() {
                    flags.push("no_identity".into());
                }
                if sheet.merged_cells > 0 {
                    // Les cellules fusionnées ne sont jamais propagées ; information seulement.
                }
                let status = if excluded.is_some() { "excluded" } else { "pending" };
                tx.execute(
                    "INSERT INTO source_record(id, document_id, sheet, row_number, cells, identity_key, status, exclusion_reason, flags) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                    params![rid, doc_id, cfg.name, row_number as i64, serde_json::to_string(&cells)?, ikey, status, excluded, serde_json::to_string(&flags)?],
                )?;
                if let Some(e) = excluded {
                    *summary.excluded.entry(e.to_string()).or_default() += 1;
                    continue;
                }
                // Identité : jamais une proposition de champ exportable ; conservée pour le rattachement.
                for (t, v) in [("last_name", map.get("last_name")), ("first_name", map.get("first_name")), ("full_name", map.get("full_name")), ("hospital_id", map.get("hospital_id"))] {
                    if let Some(v) = v.filter(|v| !v.trim().is_empty()) {
                        tx.execute("INSERT INTO import_proposal(record_id, target, proposed, rule) VALUES (?1,?2,?3,'raw_identity')", params![rid, t, json!(v.trim()).to_string()])?;
                    }
                }
                for p in &norm.proposals {
                    tx.execute(
                        "INSERT OR REPLACE INTO import_proposal(record_id, target, proposed, rule) VALUES (?1,?2,?3,?4)",
                        params![rid, p.target, (!p.value.is_null()).then(|| p.value.to_string()), p.rule],
                    )?;
                }
                staged.push((rid, ikey, cfg.name.clone(), cells));
            }
        }
        // Doublons dans le fichier : identité normalisée identique.
        let clinical = |cells: &Vec<(String, String)>| cells.iter().filter(|(h, _)| !matches!(suggest_target(h), "full_name" | "last_name" | "first_name" | "patient_code" | "hospital_id")).cloned().collect::<Vec<_>>();
        let mut by_key: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, s) in staged.iter().enumerate() {
            if let Some(k) = &s.1 {
                by_key.entry(k.clone()).or_default().push(i);
            }
        }
        let add_flag = |tx: &rusqlite::Transaction, rid: &str, flag: &str, extra: Option<Value>| -> Result<()> {
            let (fl, diff): (String, Option<String>) = tx.query_row("SELECT flags, diff FROM source_record WHERE id = ?1", [rid], |r| Ok((r.get(0)?, r.get(1)?)))?;
            let mut v: Vec<String> = serde_json::from_str(&fl)?;
            if !v.iter().any(|x| x == flag) {
                v.push(flag.to_string());
            }
            let mut d: Value = diff.and_then(|d| serde_json::from_str(&d).ok()).unwrap_or(json!({}));
            if let Some(Value::Object(e)) = extra {
                for (k, val) in e {
                    match (d.get_mut(&k), val) {
                        (Some(Value::Array(a)), Value::Array(b)) => a.extend(b),
                        (_, val) => {
                            d[&k] = val;
                        }
                    }
                }
            }
            tx.execute("UPDATE source_record SET flags = ?1, diff = ?2 WHERE id = ?3", params![serde_json::to_string(&v)?, d.to_string(), rid])?;
            Ok(())
        };
        for idxs in by_key.values().filter(|v| v.len() > 1) {
            for (j, &i) in idxs.iter().enumerate() {
                let others: Vec<Value> = idxs.iter().filter(|&&o| o != i).map(|&o| json!({"record_id": staged[o].0, "sheet": staged[o].2})).collect();
                let exact = idxs[..j].iter().any(|&o| clinical(&staged[o].3) == clinical(&staged[i].3));
                add_flag(&tx, &staged[i].0, if exact { "duplicate_exact" } else { "duplicate_candidate" }, Some(json!({"same_file": others})))?;
            }
        }
        // Doublons avec les dossiers existants et versions précédentes du même historique.
        for (rid, ikey, sheet, cells) in &staged {
            let Some(k) = ikey else { continue };
            let mut st = tx.prepare("SELECT p.id, p.code FROM patient p JOIN patient_identity i ON i.patient_id = p.id WHERE i.identity_key = ?1 AND p.merged_into IS NULL")?;
            let cands: Vec<Value> = st.query_map([k], |r| Ok(json!({"patient_id": r.get::<_, String>(0)?, "code": r.get::<_, String>(1)?})))?.collect::<rusqlite::Result<_>>()?;
            drop(st);
            let mut prev_st = tx.prepare(
                "SELECT r.id, r.cells, r.encounter_id, e.patient_id FROM source_record r LEFT JOIN encounter e ON e.id = r.encounter_id
                 WHERE r.identity_key = ?1 AND r.sheet = ?2 AND r.document_id <> ?3 AND r.status IN ('validated','unchanged') ORDER BY r.rowid DESC LIMIT 1",
            )?;
            let prev: Option<(String, String, Option<String>, Option<String>)> = prev_st.query_row(params![k, sheet, doc_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))).optional()?;
            drop(prev_st);
            if let Some((prid, pcells, penc, ppat)) = prev {
                let old: Vec<(String, String)> = serde_json::from_str(&pcells)?;
                if &old == cells {
                    tx.execute("UPDATE source_record SET status = 'unchanged', previous_record_id = ?1, encounter_id = ?2, link_patient_id = ?3 WHERE id = ?4", params![prid, penc, ppat, rid])?;
                    summary.unchanged += 1;
                    continue;
                }
                let changes: Vec<Value> = cells
                    .iter()
                    .enumerate()
                    .filter(|(i, (_, v))| old.get(*i).map(|o| &o.1) != Some(v))
                    .map(|(i, (h, v))| json!({"column": h, "old": old.get(i).map(|o| o.1.clone()), "new": v}))
                    .collect();
                tx.execute("UPDATE source_record SET previous_record_id = ?1 WHERE id = ?2", params![prid, rid])?;
                add_flag(&tx, rid, "changed_since_previous_version", Some(json!({"changes": changes, "previous_encounter_id": penc})))?;
                continue;
            }
            if !cands.is_empty() {
                add_flag(&tx, rid, "duplicate_candidate", Some(json!({"existing": cands})))?;
            }
        }
        // Bilan
        let mut st = tx.prepare("SELECT flags FROM source_record WHERE document_id = ?1 AND status = 'pending'")?;
        for fl in st.query_map([&doc_id], |r| r.get::<_, String>(0))? {
            let v: Vec<String> = serde_json::from_str(&fl?)?;
            summary.pending += 1;
            if v.iter().any(|f| legacy::is_blocking(f)) {
                summary.needs_review += 1;
            } else {
                summary.ready += 1;
            }
        }
        drop(st);
        audit_on(&tx, &author, "import_stage", "source_document", Some(&doc_id), None, None, Some(&serde_json::to_string(&summary)?), None)?;
        tx.commit()?;
        Ok(summary)
    }
}

// ---------------------------------------------------------------- Revue

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalRow {
    pub target: String,
    pub label: String,
    pub proposed: Option<Value>,
    pub rule: String,
    pub decision: String,
    pub decided_value: Option<Value>,
    pub justification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordRow {
    pub id: String,
    pub document_id: String,
    pub sheet: String,
    pub row_number: i64,
    pub cells: Vec<(String, String)>,
    pub status: String,
    pub exclusion_reason: Option<String>,
    pub flags: Vec<(String, String, bool)>,
    pub unresolved: Vec<String>,
    pub proposals: Vec<ProposalRow>,
    pub diff: Option<Value>,
    pub link_decision: Option<String>,
    pub link_patient_id: Option<String>,
    pub encounter_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRow {
    pub id: String,
    pub filename: String,
    pub sha256: String,
    pub format: String,
    pub imported_at: String,
    pub size_bytes: i64,
    pub counts: HashMap<String, i64>,
    pub needs_review: i64,
}

fn target_label(t: &str) -> String {
    match t {
        "patient_code" => "Code du dossier".into(),
        "last_name" => "Nom".into(),
        "first_name" => "Prénom".into(),
        "full_name" => "Nom et prénom".into(),
        "hospital_id" => "Identifiant hospitalier".into(),
        "service_label_source" => "Service (libellé source)".into(),
        "bewe_total_historical" => "BEWE : total historique exact".into(),
        "bewe_band" => "BEWE : tranche historique".into(),
        "bewe_legacy_raw" => "BEWE : texte source".into(),
        other => catalog::field(other).map(|f| f.label.to_string()).unwrap_or_else(|| other.to_string()),
    }
}

fn unresolved(flags: &[String], proposals: &[ProposalRow], link: &Option<String>) -> Vec<String> {
    flags
        .iter()
        .filter(|f| legacy::is_blocking(f))
        .filter(|f| {
            if LINK_FLAGS.contains(&f.as_str()) {
                return link.is_none();
            }
            match flag_target(f) {
                Some(t) => proposals.iter().find(|p| p.target == t).map(|p| p.decision == "pending").unwrap_or(true),
                None => true,
            }
        })
        .cloned()
        .collect()
}

impl Store {
    pub fn import_documents(&self) -> Result<Vec<DocumentRow>> {
        let mut st = self.conn.prepare("SELECT id, filename, sha256, format, imported_at, size_bytes FROM source_document ORDER BY imported_at DESC")?;
        let docs: Vec<(String, String, String, String, String, i64)> = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)))?.collect::<rusqlite::Result<_>>()?;
        let mut out = vec![];
        for (id, filename, sha256, format, imported_at, size) in docs {
            let mut counts = HashMap::new();
            let mut st = self.conn.prepare("SELECT status, count(*) FROM source_record WHERE document_id = ?1 GROUP BY status")?;
            for r in st.query_map([&id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))? {
                let (s, n) = r?;
                counts.insert(s, n);
            }
            let needs = self.import_records(&id, None)?.iter().filter(|r| r.status == "pending" && !r.unresolved.is_empty()).count() as i64;
            out.push(DocumentRow { id, filename, sha256, format, imported_at, size_bytes: size, counts, needs_review: needs });
        }
        Ok(out)
    }

    pub fn import_records(&self, document_id: &str, status: Option<&str>) -> Result<Vec<RecordRow>> {
        let mut props: HashMap<String, Vec<ProposalRow>> = HashMap::new();
        {
            let mut st = self.conn.prepare("SELECT p.record_id, p.target, p.proposed, p.rule, p.decision, p.decided_value, p.justification FROM import_proposal p JOIN source_record r ON r.id = p.record_id WHERE r.document_id = ?1")?;
            for r in st.query_map([document_id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    ProposalRow {
                        label: target_label(&r.get::<_, String>(1)?),
                        target: r.get(1)?,
                        proposed: r.get::<_, Option<String>>(2)?.and_then(|s| serde_json::from_str(&s).ok()),
                        rule: r.get(3)?,
                        decision: r.get(4)?,
                        decided_value: r.get::<_, Option<String>>(5)?.and_then(|s| serde_json::from_str(&s).ok()),
                        justification: r.get(6)?,
                    },
                ))
            })? {
                let (rid, p) = r?;
                props.entry(rid).or_default().push(p);
            }
        }
        let mut st = self.conn.prepare("SELECT * FROM source_record WHERE document_id = ?1 ORDER BY sheet, row_number")?;
        let rows = st
            .query_map([document_id], |r| {
                Ok((
                    r.get::<_, String>("id")?,
                    r.get::<_, String>("sheet")?,
                    r.get::<_, i64>("row_number")?,
                    r.get::<_, String>("cells")?,
                    r.get::<_, String>("status")?,
                    r.get::<_, Option<String>>("exclusion_reason")?,
                    r.get::<_, String>("flags")?,
                    r.get::<_, Option<String>>("diff")?,
                    r.get::<_, Option<String>>("link_decision")?,
                    r.get::<_, Option<String>>("link_patient_id")?,
                    r.get::<_, Option<String>>("encounter_id")?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut out = vec![];
        for (id, sheet, row_number, cells, st_, excl, flags, diff, link, link_pid, enc) in rows {
            if let Some(s) = status {
                if s != st_ {
                    continue;
                }
            }
            let flags: Vec<String> = serde_json::from_str(&flags)?;
            let mut p = props.remove(&id).unwrap_or_default();
            p.sort_by_key(|x| x.target.clone());
            let unres = if st_ == "pending" { unresolved(&flags, &p, &link) } else { vec![] };
            out.push(RecordRow {
                flags: flags.iter().map(|f| (f.clone(), legacy::flag_label(f).to_string(), legacy::is_blocking(f))).collect(),
                unresolved: unres,
                id,
                document_id: document_id.to_string(),
                sheet,
                row_number,
                cells: serde_json::from_str(&cells)?,
                status: st_,
                exclusion_reason: excl,
                proposals: p,
                diff: diff.and_then(|d| serde_json::from_str(&d).ok()),
                link_decision: link,
                link_patient_id: link_pid,
                encounter_id: enc,
            });
        }
        Ok(out)
    }

    fn record_pending(&self, record_id: &str) -> Result<()> {
        let s: String = self.conn.query_row("SELECT status FROM source_record WHERE id = ?1", [record_id], |r| r.get(0)).optional()?.ok_or_else(|| CoreError::NotFound("ligne source".into()))?;
        if s != "pending" {
            return Err(CoreError::Refused("ligne déjà traitée".into()));
        }
        Ok(())
    }

    /// Décision sur une proposition : accepter, corriger (justification obligatoire) ou marquer indisponible.
    pub fn import_decide(&mut self, record_id: &str, target: &str, decision: &str, value: Option<Value>, justification: Option<String>) -> Result<()> {
        self.record_pending(record_id)?;
        let just = justification.filter(|j| !j.trim().is_empty());
        let proposed: Option<Option<String>> = self.conn.query_row("SELECT proposed FROM import_proposal WHERE record_id = ?1 AND target = ?2", params![record_id, target], |r| r.get(0)).optional()?;
        let Some(proposed) = proposed else { return Err(CoreError::NotFound("proposition".into())) };
        let decided = match decision {
            "accepted" => {
                if proposed.is_none() {
                    return Err(CoreError::Refused("aucune valeur proposée : corriger ou marquer indisponible".into()));
                }
                None
            }
            "corrected" => {
                if just.as_ref().map(|j| j.trim().len() < 3).unwrap_or(true) {
                    return Err(CoreError::validation("justification", "une justification est obligatoire pour corriger"));
                }
                let v = value.filter(|v| !v.is_null()).ok_or_else(|| CoreError::validation(target, "valeur attendue"))?;
                validate_target(target, &v)?;
                Some(v.to_string())
            }
            "unavailable" | "pending" => None,
            _ => return Err(CoreError::validation("décision", "décision inconnue")),
        };
        self.conn.execute(
            "UPDATE import_proposal SET decision = ?1, decided_value = ?2, justification = ?3, decided_at = ?4 WHERE record_id = ?5 AND target = ?6",
            params![decision, decided, just, (decision != "pending").then(now), record_id, target],
        )?;
        self.audit("import_decide", "source_record", Some(record_id), Some(target), proposed.as_deref(), decided.as_deref().or(Some(decision)), just.as_deref())?;
        Ok(())
    }

    /// Rattachement : même patient qu'un dossier existant, personne différente (homonyme) ou nouveau dossier.
    pub fn import_link(&mut self, record_id: &str, decision: &str, patient_id: Option<String>) -> Result<()> {
        self.record_pending(record_id)?;
        match decision {
            "same_patient" => {
                let pid = patient_id.clone().ok_or_else(|| CoreError::validation("rattachement", "dossier cible manquant"))?;
                let ok: i64 = self.conn.query_row("SELECT count(*) FROM patient WHERE id = ?1 AND merged_into IS NULL", [&pid], |r| r.get(0))?;
                if ok == 0 {
                    return Err(CoreError::NotFound("dossier cible".into()));
                }
            }
            "different_person" | "new" => {}
            "" => {
                self.conn.execute("UPDATE source_record SET link_decision = NULL, link_patient_id = NULL WHERE id = ?1", [record_id])?;
                return Ok(());
            }
            _ => return Err(CoreError::validation("rattachement", "décision inconnue")),
        }
        self.conn.execute("UPDATE source_record SET link_decision = ?1, link_patient_id = ?2 WHERE id = ?3", params![decision, if decision == "same_patient" { patient_id } else { None }, record_id])?;
        self.audit("import_link", "source_record", Some(record_id), None, None, Some(decision), None)?;
        Ok(())
    }

    pub fn import_exclude(&mut self, record_id: &str, reason: &str) -> Result<()> {
        self.record_pending(record_id)?;
        if reason.trim().len() < 3 {
            return Err(CoreError::validation("motif", "motif d'exclusion obligatoire"));
        }
        self.conn.execute("UPDATE source_record SET status = 'excluded', exclusion_reason = ?1 WHERE id = ?2", params![reason.trim(), record_id])?;
        self.audit("import_exclude", "source_record", Some(record_id), None, None, None, Some(reason.trim()))?;
        Ok(())
    }

    pub fn import_reinclude(&mut self, record_id: &str) -> Result<()> {
        let n = self.conn.execute("UPDATE source_record SET status = 'pending', exclusion_reason = NULL WHERE id = ?1 AND status = 'excluded'", [record_id])?;
        if n == 0 {
            return Err(CoreError::Refused("seule une ligne exclue peut être réintégrée".into()));
        }
        self.audit("import_reinclude", "source_record", Some(record_id), None, None, None, None)?;
        Ok(())
    }

    /// Validation : crée les consultations historiques pour les lignes prêtes. Les lignes non résolues
    /// restent en attente (jamais perdues). Sans liste, seules les lignes sans anomalie sont validées en lot.
    pub fn import_commit(&mut self, document_id: &str, record_ids: Option<Vec<String>>) -> Result<CommitReport> {
        let records = self.import_records(document_id, Some("pending"))?;
        let author = self.author.clone();
        let tx = self.conn.transaction()?;
        let mut report = CommitReport::default();
        for rec in records {
            let selected = match &record_ids {
                Some(ids) => ids.contains(&rec.id),
                None => true,
            };
            if !selected {
                continue;
            }
            let blocking = rec.flags.iter().any(|f| f.2);
            if !rec.unresolved.is_empty() || (record_ids.is_none() && blocking) {
                report.left_pending += 1;
                continue;
            }
            let prop = |t: &str| rec.proposals.iter().find(|p| p.target == t);
            let final_value = |t: &str| -> Option<Value> {
                let p = prop(t)?;
                match p.decision.as_str() {
                    "corrected" => p.decided_value.clone(),
                    "unavailable" => None,
                    _ => p.proposed.clone(),
                }
            };
            // Dossier
            let pid = match (rec.link_decision.as_deref(), &rec.link_patient_id) {
                (Some("same_patient"), Some(pid)) => pid.clone(),
                _ => {
                    let pid = new_id();
                    let wanted = final_value("patient_code").and_then(|v| v.as_str().map(String::from));
                    let code = match wanted {
                        Some(c) if tx.query_row("SELECT count(*) FROM patient WHERE code = ?1", [&c], |r| r.get::<_, i64>(0))? == 0 => c,
                        _ => {
                            let n: i64 = tx.query_row("SELECT count(*) FROM patient WHERE code LIKE 'H-%'", [], |r| r.get(0))?;
                            let mut i = n + 1;
                            loop {
                                let c = format!("H-{i:04}");
                                if tx.query_row("SELECT count(*) FROM patient WHERE code = ?1", [&c], |r| r.get::<_, i64>(0))? == 0 {
                                    break c;
                                }
                                i += 1;
                            }
                        }
                    };
                    tx.execute("INSERT INTO patient(id, code, created_at) VALUES (?1,?2,?3)", params![pid, code, now()])?;
                    let s = |t: &str| final_value(t).and_then(|v| v.as_str().map(String::from));
                    let (ln, fnm) = match (s("last_name"), s("first_name"), s("full_name")) {
                        (None, None, Some(full)) => (Some(full), None),
                        (a, b, _) => (a, b),
                    };
                    if ln.is_some() || fnm.is_some() || s("hospital_id").is_some() {
                        write_identity(&tx, &pid, &IdentityInput { last_name: ln, first_name: fnm, hospital_id: s("hospital_id") })?;
                    }
                    report.patients_created += 1;
                    pid
                }
            };
            let eid = insert_encounter(&tx, &pid, "legacy_retrospective", &author)?;
            tx.execute("UPDATE encounter SET source_record_id = ?1 WHERE id = ?2", params![rec.id, eid])?;
            for p in &rec.proposals {
                let source_type = if p.decision == "corrected" { "clinician_adjudication" } else { "legacy_import" };
                let v = final_value(&p.target);
                match p.target.as_str() {
                    "patient_code" | "last_name" | "first_name" | "full_name" | "hospital_id" => {}
                    "service_label_source" => {
                        tx.execute("UPDATE encounter SET service_label_source = ?1 WHERE id = ?2", params![v.and_then(|x| x.as_str().map(String::from)), eid])?;
                    }
                    "bewe_legacy_raw" => {
                        tx.execute("UPDATE encounter SET bewe_legacy_raw = ?1 WHERE id = ?2", params![p.proposed.as_ref().and_then(|x| x.as_str().map(String::from)), eid])?;
                    }
                    "bewe_total_historical" => {
                        let t = v.and_then(|x| x.as_i64()).filter(|t| (0..=18).contains(t));
                        tx.execute("UPDATE encounter SET bewe_total_historical = ?1 WHERE id = ?2", params![t, eid])?;
                    }
                    "bewe_band" => {
                        if let Some(Value::Array(a)) = v {
                            tx.execute("UPDATE encounter SET bewe_historical_band_min = ?1, bewe_historical_band_max = ?2 WHERE id = ?3", params![a.first().and_then(|x| x.as_i64()), a.get(1).and_then(|x| x.as_i64()), eid])?;
                        }
                    }
                    field => {
                        let input = match v {
                            Some(val) => FieldInput { field: field.into(), value: Some(val), source_type: Some(source_type.into()), ..Default::default() },
                            None => FieldInput {
                                field: field.into(),
                                missing_reason: Some(if p.decision == "unavailable" { "ambiguous_source" } else { "not_recorded" }.into()),
                                source_type: Some(source_type.into()),
                                ..Default::default()
                            },
                        };
                        if let Some(sv) = values::validate(&input)? {
                            write_value(&tx, &eid, &author, &sv, Some(&rec.id))?;
                        }
                    }
                }
            }
            // Dates et services dénormalisés
            let date: Option<String> = tx.query_row("SELECT value_text FROM field_value WHERE encounter_id = ?1 AND field = 'visit_date'", [&eid], |r| r.get(0)).optional()?.flatten();
            let service: Option<String> = tx.query_row("SELECT value_text FROM field_value WHERE encounter_id = ?1 AND field = 'service_code'", [&eid], |r| r.get(0)).optional()?.flatten();
            tx.execute("UPDATE encounter SET visit_date = ?1, visit_date_precision = CASE WHEN ?1 IS NULL THEN NULL ELSE 'day' END, service_code = ?2 WHERE id = ?3", params![date, service, eid])?;
            // Anomalies conservées sur la consultation (exceptions acceptées visibles pour l'analyse de sensibilité).
            for (flag, _, is_blocking) in &rec.flags {
                let status = match flag_target(flag).and_then(|t| prop(t)) {
                    Some(p) if p.decision == "accepted" => "accepted",
                    Some(_) => "resolved",
                    None if !is_blocking => "accepted",
                    None => "resolved",
                };
                let detail = match flag_target(flag).and_then(|t| prop(t)) {
                    Some(p) => p.justification.clone(),
                    None => None,
                };
                tx.execute("INSERT INTO encounter_flag(encounter_id, flag, detail, status, created_at) VALUES (?1,?2,?3,?4,?5)", params![eid, flag, detail, status, now()])?;
            }
            let t = now();
            tx.execute("UPDATE encounter SET status = 'validated', revision = 1, version = version + 1, validated_at = ?1 WHERE id = ?2", params![t, eid])?;
            let snap = load_full(&tx, &eid, false)?;
            tx.execute("INSERT INTO encounter_revision(encounter_id, revision, snapshot, reason, created_at, author) VALUES (?1,1,?2,'Reprise de l''historique',?3,?4)", params![eid, serde_json::to_string(&snap)?, t, author])?;
            tx.execute("UPDATE source_record SET status = 'validated', encounter_id = ?1, link_patient_id = ?2 WHERE id = ?3", params![eid, pid, rec.id])?;
            // Propositions tranchées automatiquement par règle déterministe (lignes sans anomalie).
            tx.execute("UPDATE import_proposal SET decision = 'accepted', decided_at = ?1 WHERE record_id = ?2 AND decision = 'pending' AND proposed IS NOT NULL", params![t, rec.id])?;
            audit_on(&tx, &author, "import_commit", "encounter", Some(&eid), None, None, Some(&rec.id), None)?;
            report.encounters_created += 1;
        }
        tx.commit()?;
        Ok(report)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommitReport {
    pub encounters_created: usize,
    pub patients_created: usize,
    pub left_pending: usize,
}

fn validate_target(target: &str, v: &Value) -> Result<()> {
    match target {
        "bewe_total_historical" => {
            let ok = v.as_i64().map(|t| (0..=18).contains(&t)).unwrap_or(false);
            if !ok {
                return Err(CoreError::validation(target, "entier de 0 à 18"));
            }
        }
        "bewe_band" | "bewe_legacy_raw" | "service_label_source" => {}
        "patient_code" | "last_name" | "first_name" | "full_name" | "hospital_id" => {
            if v.as_str().map(|s| s.trim().is_empty()).unwrap_or(true) {
                return Err(CoreError::validation(target, "texte attendu"));
            }
        }
        field => {
            values::validate(&FieldInput { field: field.into(), value: Some(v.clone()), ..Default::default() })?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- Fusion manuelle, annulable

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRow {
    pub id: String,
    pub source_patient_id: String,
    pub target_patient_id: String,
    pub reason: String,
    pub at: String,
    pub undone_at: Option<String>,
    pub moved: Vec<String>,
}

impl Store {
    pub fn merge_patients(&mut self, source: &str, target: &str, reason: &str) -> Result<String> {
        if source == target {
            return Err(CoreError::Refused("un dossier ne se fusionne pas avec lui-même".into()));
        }
        if reason.trim().len() < 3 {
            return Err(CoreError::validation("motif", "motif de fusion obligatoire"));
        }
        let author = self.author.clone();
        let tx = self.conn.transaction()?;
        for p in [source, target] {
            let ok: i64 = tx.query_row("SELECT count(*) FROM patient WHERE id = ?1 AND merged_into IS NULL", [p], |r| r.get(0))?;
            if ok == 0 {
                return Err(CoreError::NotFound("dossier".into()));
            }
        }
        let mut st = tx.prepare("SELECT id FROM encounter WHERE patient_id = ?1")?;
        let moved: Vec<String> = st.query_map([source], |r| r.get(0))?.collect::<rusqlite::Result<_>>()?;
        drop(st);
        tx.execute("UPDATE encounter SET patient_id = ?1 WHERE patient_id = ?2", params![target, source])?;
        tx.execute("UPDATE patient SET merged_into = ?1 WHERE id = ?2", params![target, source])?;
        let id = new_id();
        tx.execute("INSERT INTO merge_event(id, source_patient_id, target_patient_id, moved_encounters, reason, at) VALUES (?1,?2,?3,?4,?5,?6)", params![id, source, target, serde_json::to_string(&moved)?, reason.trim(), now()])?;
        audit_on(&tx, &author, "merge", "patient", Some(source), None, None, Some(target), Some(reason.trim()))?;
        tx.commit()?;
        Ok(id)
    }

    pub fn undo_merge(&mut self, merge_id: &str) -> Result<()> {
        let author = self.author.clone();
        let tx = self.conn.transaction()?;
        let (src, tgt, moved, undone): (String, String, String, Option<String>) = tx
            .query_row("SELECT source_patient_id, target_patient_id, moved_encounters, undone_at FROM merge_event WHERE id = ?1", [merge_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .optional()?
            .ok_or_else(|| CoreError::NotFound("fusion".into()))?;
        if undone.is_some() {
            return Err(CoreError::Refused("fusion déjà annulée".into()));
        }
        let moved: Vec<String> = serde_json::from_str(&moved)?;
        for e in &moved {
            tx.execute("UPDATE encounter SET patient_id = ?1 WHERE id = ?2 AND patient_id = ?3", params![src, e, tgt])?;
        }
        tx.execute("UPDATE patient SET merged_into = NULL WHERE id = ?1", [&src])?;
        tx.execute("UPDATE merge_event SET undone_at = ?1 WHERE id = ?2", params![now(), merge_id])?;
        audit_on(&tx, &author, "undo_merge", "patient", Some(&src), None, None, Some(&tgt), None)?;
        tx.commit()?;
        Ok(())
    }

    pub fn merges(&self) -> Result<Vec<MergeRow>> {
        let mut st = self.conn.prepare("SELECT id, source_patient_id, target_patient_id, reason, at, undone_at, moved_encounters FROM merge_event ORDER BY at DESC")?;
        let rows = st
            .query_map([], |r| {
                let m: String = r.get(6)?;
                Ok(MergeRow { id: r.get(0)?, source_patient_id: r.get(1)?, target_patient_id: r.get(2)?, reason: r.get(3)?, at: r.get(4)?, undone_at: r.get(5)?, moved: serde_json::from_str(&m).unwrap_or_default() })
            })?
            .collect::<rusqlite::Result<_>>()?;
        Ok(rows)
    }

    /// Dossiers partageant une identité normalisée : candidats à revoir, jamais fusionnés d'office.
    pub fn merge_candidates(&self) -> Result<Vec<Vec<(String, String)>>> {
        let mut st = self.conn.prepare(
            "SELECT i.identity_key, p.id, p.code FROM patient p JOIN patient_identity i ON i.patient_id = p.id
             WHERE p.merged_into IS NULL AND i.identity_key IN (SELECT identity_key FROM patient_identity i2 JOIN patient p2 ON p2.id = i2.patient_id WHERE p2.merged_into IS NULL AND identity_key IS NOT NULL GROUP BY identity_key HAVING count(*) > 1)
             ORDER BY i.identity_key, p.code",
        )?;
        let mut groups: Vec<(String, Vec<(String, String)>)> = vec![];
        for r in st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)))? {
            let (k, id, code) = r?;
            match groups.last_mut() {
                Some((kk, v)) if *kk == k => v.push((id, code)),
                _ => groups.push((k, vec![(id, code)])),
            }
        }
        Ok(groups.into_iter().map(|g| g.1).collect())
    }
}
