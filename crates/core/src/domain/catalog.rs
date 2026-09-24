//! Catalogue des champs du recueil : source unique pour l'interface, la validation native et l'export.
//! Les codes sont stables (anglais) ; les libellés sont français.

use serde::Serialize;
use std::sync::OnceLock;

pub const FORM_VERSION: &str = "CMME-FORM-2026.09";
pub const PROTOCOL_VERSION: &str = "CMME-FM-2026-09";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Choice,
    Multi,
    Number,
    Date,
    Text,
}

#[derive(Debug, Clone, Serialize)]
pub struct Opt {
    pub code: &'static str,
    pub label: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShowIf {
    pub field: &'static str,
    pub values: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub code: &'static str,
    pub section: &'static str,
    pub block: &'static str,
    pub label: &'static str,
    pub label_en: &'static str,
    pub kind: Kind,
    pub options: Vec<Opt>,
    pub exclusive: Vec<&'static str>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub integer: bool,
    pub unit: Option<&'static str>,
    pub allow_range: bool,
    pub export: bool,
    pub legacy: bool,
    pub essential: bool,
    pub wide: bool,
    pub hint: Option<&'static str>,
    pub show_if: Option<ShowIf>,
    pub missing: Vec<&'static str>,
    /// Rendu par un composant dédié (BEWE, expositions, prévention) plutôt que générique.
    pub custom: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Section {
    pub id: &'static str,
    pub number: u8,
    pub title: &'static str,
    pub heading: &'static str,
    pub subtitle: &'static str,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Block {
    pub id: &'static str,
    pub title: Option<&'static str>,
    pub collapsible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Catalog {
    pub form_version: &'static str,
    pub protocol_version: &'static str,
    pub sections: Vec<Section>,
    pub fields: Vec<Field>,
}

const REP: &[&str] = &["unknown", "declined", "not_applicable"];
const REC: &[&str] = &["unknown", "not_applicable"];
const EXAM: &[&str] = &["not_assessable", "not_applicable"];

fn opts(list: &[(&'static str, &'static str)]) -> Vec<Opt> {
    list.iter().map(|(c, l)| Opt { code: c, label: l }).collect()
}

const YES_NO: &[(&str, &str)] = &[("yes", "Oui"), ("no", "Non")];
const NPC: &[(&str, &str)] = &[("never", "Jamais"), ("past", "Ancien"), ("current", "Actuel")];

struct B {
    f: Field,
}

impl B {
    fn new(code: &'static str, section: &'static str, block: &'static str, label: &'static str, label_en: &'static str, kind: Kind) -> Self {
        B {
            f: Field {
                code,
                section,
                block,
                label,
                label_en,
                kind,
                options: vec![],
                exclusive: vec![],
                min: None,
                max: None,
                integer: false,
                unit: None,
                allow_range: false,
                export: kind != Kind::Text,
                legacy: false,
                essential: false,
                wide: false,
                hint: None,
                show_if: None,
                missing: REP.to_vec(),
                custom: false,
            },
        }
    }
    fn opts(mut self, o: &[(&'static str, &'static str)]) -> Self {
        self.f.options = opts(o);
        self
    }
    fn excl(mut self, e: &[&'static str]) -> Self {
        self.f.exclusive = e.to_vec();
        self
    }
    fn num(mut self, min: f64, max: f64, integer: bool, unit: Option<&'static str>) -> Self {
        self.f.min = Some(min);
        self.f.max = Some(max);
        self.f.integer = integer;
        self.f.unit = unit;
        self
    }
    fn range(mut self) -> Self {
        self.f.allow_range = true;
        self
    }
    fn essential(mut self) -> Self {
        self.f.essential = true;
        self
    }
    fn wide(mut self) -> Self {
        self.f.wide = true;
        self
    }
    fn hint(mut self, h: &'static str) -> Self {
        self.f.hint = Some(h);
        self
    }
    fn show_if(mut self, field: &'static str, values: &[&'static str]) -> Self {
        self.f.show_if = Some(ShowIf { field, values: values.to_vec() });
        self
    }
    fn missing(mut self, m: &[&'static str]) -> Self {
        self.f.missing = m.to_vec();
        self
    }
    fn legacy(mut self) -> Self {
        self.f.legacy = true;
        self.f.missing = vec![];
        self
    }
    fn no_export(mut self) -> Self {
        self.f.export = false;
        self
    }
    fn custom(mut self) -> Self {
        self.f.custom = true;
        self
    }
    fn done(self) -> Field {
        self.f
    }
}

fn build() -> Catalog {
    use Kind::*;
    let sections = vec![
        Section {
            id: "contexte",
            number: 1,
            title: "Contexte",
            heading: "Faire connaissance",
            subtitle: "Les repères de la consultation.",
            blocks: vec![
                Block { id: "contexte_main", title: None, collapsible: false },
                Block { id: "contexte_details", title: Some("Compléments TCA et contexte somatique"), collapsible: true },
            ],
        },
        Section {
            id: "habitudes",
            number: 2,
            title: "Habitudes",
            heading: "Les habitudes au quotidien",
            subtitle: "Hygiène bucco-dentaire et symptômes rapportés.",
            blocks: vec![
                Block { id: "habitudes_main", title: None, collapsible: false },
                Block { id: "habitudes_details", title: Some("Après vomissement, accès aux soins, produits"), collapsible: true },
            ],
        },
        Section {
            id: "expositions",
            number: 3,
            title: "Expositions",
            heading: "Les expositions acides",
            subtitle: "Plusieurs boissons et aliments peuvent être sélectionnés.",
            blocks: vec![
                Block { id: "expositions_main", title: None, collapsible: false },
                Block { id: "expositions_details", title: Some("Reflux, tabac, alcool et traitements"), collapsible: true },
            ],
        },
        Section {
            id: "examen",
            number: 4,
            title: "Examen",
            heading: "L'examen bucco-dentaire",
            subtitle: "Panoramique, examen endobuccal et score le plus élevé de chaque sextant.",
            blocks: vec![
                Block { id: "examen_main", title: None, collapsible: false },
                Block { id: "examen_bewe", title: None, collapsible: false },
                Block { id: "examen_constats", title: Some("Autres constats cliniques"), collapsible: false },
                Block { id: "examen_details", title: Some("Compléments d'examen (CAO, muqueuses, exobuccal)"), collapsible: true },
            ],
        },
        Section {
            id: "prevention",
            number: 5,
            title: "Prévention",
            heading: "La prévention, adaptée.",
            subtitle: "Consignez ce qui a été proposé pendant l'entretien.",
            blocks: vec![Block { id: "prevention_main", title: None, collapsible: false }],
        },
        Section {
            id: "observations",
            number: 6,
            title: "Observations",
            heading: "Les dernières observations",
            subtitle: "Un espace libre pour les précisions utiles.",
            blocks: vec![
                Block { id: "observations_main", title: None, collapsible: false },
                Block { id: "observations_orientation", title: Some("Besoins et orientation"), collapsible: false },
            ],
        },
        Section {
            id: "historique",
            number: 7,
            title: "Historique",
            heading: "Données de l'ancien recueil",
            subtitle: "Valeurs reprises de l'ancien tableau, conservées telles quelles.",
            blocks: vec![Block { id: "historique_main", title: None, collapsible: false }],
        },
    ];

    let cur_past = &["current", "past_only"];
    let fields = vec![
        // ---------- Contexte
        B::new("visit_date", "contexte", "contexte_main", "Date de consultation", "Visit date", Date).essential().missing(&["unknown"]).hint("JJ/MM/AAAA").done(),
        B::new("service_code", "contexte", "contexte_main", "Service", "Service", Choice)
            .opts(&[("centre_expert", "Centre expert"), ("sas", "SAS"), ("hospit_complete", "Hospitalisation complète"), ("hdj", "HDJ"), ("hdj_intensif", "HDJ intensif"), ("autre", "Autre")])
            .essential().missing(&["unknown"]).done(),
        B::new("age_years", "contexte", "contexte_main", "Âge", "Age (years)", Number).num(0.0, 110.0, true, Some("ans")).essential().missing(&["unknown", "declined"]).done(),
        B::new("sex_recorded", "contexte", "contexte_main", "Sexe / genre recueilli", "Recorded sex/gender", Choice)
            .opts(&[("female", "Femme"), ("male", "Homme"), ("other", "Autre / non binaire")]).done(),
        B::new("occupational_status", "contexte", "contexte_main", "Situation", "Occupational status", Choice)
            .opts(&[("studies", "Études"), ("employed", "Emploi"), ("unemployed", "Sans emploi"), ("retired", "Retraite"), ("other", "Autre")]).done(),
        B::new("occupation_text", "contexte", "contexte_main", "Profession / statut (précision)", "Occupation (free text)", Text).hint("Non exporté").done(),
        B::new("ed_diagnosis", "contexte", "contexte_main", "Diagnostic TCA documenté", "Documented eating disorder diagnosis", Choice)
            .opts(&[("AN", "Anorexie mentale"), ("BN", "Boulimie"), ("BED", "Hyperphagie boulimique"), ("ARFID", "ARFID"), ("OSFED", "OSFED"), ("UFED", "UFED"), ("rumination", "Mérycisme"), ("other", "Autre"), ("unspecified", "Non précisé")])
            .essential().missing(&["unknown"]).done(),
        B::new("ed_comment", "contexte", "contexte_main", "Commentaire sur le diagnostic TCA", "Eating disorder diagnosis comment", Text).wide().done(),
        B::new("ed_duration_months", "contexte", "contexte_main", "Ancienneté du TCA", "Eating disorder duration", Number).num(0.0, 900.0, false, Some("mois")).range().done(),
        B::new("ed_current_course", "contexte", "contexte_main", "Évolution actuelle", "Current course", Choice)
            .opts(&[("active", "Actif"), ("remission_reported", "Rémission rapportée"), ("other", "Autre")]).done(),
        B::new("ed_subtype", "contexte", "contexte_details", "Sous-type documenté", "Documented subtype", Text).no_export().done(),
        B::new("ed_onset_age", "contexte", "contexte_details", "Âge au début du TCA", "Age at onset", Number).num(0.0, 100.0, true, Some("ans")).range().done(),
        B::new("compensatory_behaviors", "contexte", "contexte_details", "Comportements compensatoires", "Compensatory behaviours", Multi)
            .opts(&[("vomiting", "Vomissements"), ("laxatives", "Laxatifs"), ("diuretics", "Diurétiques"), ("exercise", "Exercice excessif"), ("restriction", "Restriction"), ("other", "Autre")]).done(),
        B::new("binge_episodes_28d", "contexte", "contexte_details", "Crises (28 derniers jours)", "Binge episodes (28 days)", Number).num(0.0, 1000.0, true, Some("crises")).range().done(),
        B::new("chew_spit", "contexte", "contexte_details", "Mâcher-recracher", "Chew and spit", Choice).opts(NPC).done(),
        B::new("weight_kg", "contexte", "contexte_details", "Poids (dossier)", "Weight", Number).num(15.0, 300.0, false, Some("kg")).done(),
        B::new("height_cm", "contexte", "contexte_details", "Taille (dossier)", "Height", Number).num(80.0, 230.0, false, Some("cm")).done(),
        B::new("relevant_conditions", "contexte", "contexte_details", "Antécédents pertinents", "Relevant conditions", Text).wide().hint("RGO documenté, hyperémèse, autres expositions").done(),
        // ---------- Habitudes
        B::new("brushing_daily", "habitudes", "habitudes_main", "Brossages par jour", "Tooth brushing per day", Number).num(0.0, 10.0, false, Some("/ jour")).range().essential().done(),
        B::new("brush_type", "habitudes", "habitudes_main", "Type de brosse", "Brush type", Choice).opts(&[("manual", "Manuelle"), ("electric", "Électrique"), ("both", "Les deux"), ("other", "Autre")]).done(),
        B::new("bristle_hardness", "habitudes", "habitudes_main", "Dureté des poils", "Bristle hardness", Choice).opts(&[("soft", "Souple"), ("medium", "Médium"), ("hard", "Dure")]).done(),
        B::new("toothpaste_name", "habitudes", "habitudes_main", "Dentifrice", "Toothpaste name", Text).hint("Nom, fluor si connu").done(),
        B::new("last_dental_visit_months", "habitudes", "habitudes_main", "Dernière visite chez le dentiste", "Last dental visit (months ago)", Number).num(0.0, 600.0, false, Some("mois")).range().hint("Il y a combien de mois").done(),
        B::new("usual_dentist", "habitudes", "habitudes_main", "Dentiste traitant", "Regular dentist", Choice).opts(YES_NO).done(),
        B::new("dry_mouth_reported", "habitudes", "habitudes_main", "Bouche sèche déclarée", "Reported dry mouth", Choice).opts(YES_NO).done(),
        B::new("hypersensitivity", "habitudes", "habitudes_main", "Sensibilité dentaire", "Dentine hypersensitivity (reported)", Choice).opts(YES_NO).done(),
        B::new("hypersensitivity_intensity", "habitudes", "habitudes_main", "Intensité de la sensibilité", "Hypersensitivity intensity", Number).num(0.0, 10.0, true, Some("/ 10")).show_if("hypersensitivity", &["yes"]).done(),
        B::new("awake_bruxism_reported", "habitudes", "habitudes_main", "Serrement / grincement diurne déclaré", "Awake bruxism (reported)", Choice).opts(YES_NO).done(),
        B::new("sleep_bruxism_reported", "habitudes", "habitudes_main", "Bruxisme du sommeil déclaré", "Sleep bruxism (reported)", Choice).opts(YES_NO).done(),
        B::new("post_vomit_rinse", "habitudes", "habitudes_details", "Rinçage après vomissement", "Rinse after vomiting", Choice).opts(YES_NO).done(),
        B::new("post_vomit_brushing", "habitudes", "habitudes_details", "Brossage après vomissement", "Brushing after vomiting", Choice).opts(YES_NO).done(),
        B::new("post_vomit_brushing_delay_min", "habitudes", "habitudes_details", "Délai avant brossage", "Delay before brushing", Number).num(0.0, 600.0, true, Some("min")).range().show_if("post_vomit_brushing", &["yes"]).done(),
        B::new("forceful_brushing_reported", "habitudes", "habitudes_details", "Brossage énergique déclaré", "Forceful brushing (reported)", Choice).opts(YES_NO).done(),
        B::new("toothpaste_fluoride_ppm", "habitudes", "habitudes_details", "Fluor du dentifrice (étiquette)", "Toothpaste fluoride", Number).num(0.0, 6000.0, true, Some("ppm")).done(),
        B::new("toothpaste_features", "habitudes", "habitudes_details", "Particularités du dentifrice", "Toothpaste features", Multi)
            .opts(&[("charcoal", "Charbon"), ("whitening", "Blanchissant"), ("desensitizing", "Désensibilisant"), ("anti_erosion", "Anti-érosion"), ("other", "Autre")]).done(),
        B::new("interdental_cleaning", "habitudes", "habitudes_details", "Nettoyage interdentaire", "Interdental cleaning", Choice).opts(YES_NO).done(),
        B::new("mouthrinse_use", "habitudes", "habitudes_details", "Bain de bouche habituel", "Mouthrinse use", Choice).opts(YES_NO).done(),
        B::new("access_barrier", "habitudes", "habitudes_details", "Obstacles aux soins déclarés", "Reported barriers to care", Multi)
            .opts(&[("cost", "Coût"), ("anxiety", "Anxiété"), ("shame", "Honte"), ("availability", "Disponibilité"), ("other", "Autre"), ("none", "Aucun obstacle déclaré")]).excl(&["none"]).done(),
        B::new("chewing_difficulty", "habitudes", "habitudes_details", "Gêne à la mastication", "Chewing difficulty", Choice).opts(YES_NO).done(),
        B::new("aesthetic_concern", "habitudes", "habitudes_details", "Gêne esthétique", "Aesthetic concern", Choice).opts(YES_NO).done(),
        // ---------- Expositions
        B::new("vomiting_lifetime", "expositions", "expositions_main", "Vomissements", "Vomiting history", Choice)
            .opts(&[("never_reported", "Jamais rapportés"), ("past_only", "Anciens uniquement"), ("current", "Actuels")]).essential()
            .hint("« Jamais » seulement après question explicite").done(),
        B::new("vomiting_days_28d", "expositions", "expositions_main", "Jours avec vomissements (28 j)", "Days with vomiting (28 days)", Number).num(0.0, 28.0, true, Some("jours")).range().show_if("vomiting_lifetime", &["current"]).done(),
        B::new("vomiting_episodes_28d", "expositions", "expositions_main", "Nombre de vomissements (28 j)", "Vomiting episodes (28 days)", Number).num(0.0, 3000.0, true, Some("épisodes")).range().show_if("vomiting_lifetime", &["current"]).done(),
        B::new("vomiting_pattern", "expositions", "expositions_main", "Rythme", "Pattern", Choice)
            .opts(&[("daily", "Quotidien"), ("intermittent", "Intermittent"), ("periods", "Par périodes"), ("other", "Autre")]).show_if("vomiting_lifetime", cur_past).done(),
        B::new("vomiting_duration_months", "expositions", "expositions_main", "Durée cumulée", "Cumulative duration", Number).num(0.0, 900.0, false, Some("mois")).range().show_if("vomiting_lifetime", cur_past).done(),
        B::new("vomiting_stop_months", "expositions", "expositions_main", "Arrêt depuis", "Stopped since", Number).num(0.0, 900.0, false, Some("mois")).range().show_if("vomiting_lifetime", &["past_only"]).done(),
        B::new("vomiting_context", "expositions", "expositions_main", "Nature", "Context", Choice)
            .opts(&[("self_induced", "Auto-induits"), ("spontaneous", "Spontanés"), ("other", "Autre")]).show_if("vomiting_lifetime", cur_past).done(),
        B::new("acidic_drinks_status", "expositions", "expositions_main", "Boissons acides", "Acidic drinks", Choice)
            .opts(&[("reported", "Au moins une rapportée"), ("none_reported", "Aucune rapportée")]).custom().done(),
        B::new("acidic_drinks_free_text", "expositions", "expositions_main", "Boissons : précisions libres", "Drinks free text", Text).custom().done(),
        B::new("acidic_foods_status", "expositions", "expositions_main", "Aliments acides", "Acidic foods", Choice)
            .opts(&[("reported", "Au moins un rapporté"), ("none_reported", "Aucun rapporté")]).custom().done(),
        B::new("acidic_foods_free_text", "expositions", "expositions_main", "Aliments : précisions libres", "Foods free text", Text).custom().done(),
        B::new("exposure_note", "expositions", "expositions_main", "Quantités et fréquences (précision globale)", "Exposure note", Text).wide().hint("Ex. soda : 1 canette/j ; agrumes : 3 fois/sem").done(),
        B::new("reflux_symptoms", "expositions", "expositions_details", "Reflux / brûlures déclarés", "Reflux symptoms (reported)", Choice).opts(YES_NO).done(),
        B::new("reflux_documented", "expositions", "expositions_details", "RGO documenté", "Documented GERD", Choice).opts(YES_NO).done(),
        B::new("regurgitation_rumination", "expositions", "expositions_details", "Régurgitations / mérycisme", "Regurgitation / rumination", Choice)
            .opts(&[("reported", "Rapportés"), ("documented", "Documentés"), ("suspected", "Suspectés"), ("absent", "Absents")]).done(),
        B::new("tobacco_status", "expositions", "expositions_details", "Tabac", "Tobacco", Choice).opts(NPC).done(),
        B::new("tobacco_cigarettes_day", "expositions", "expositions_details", "Cigarettes par jour", "Cigarettes per day", Number).num(0.0, 100.0, true, Some("/ jour")).range().show_if("tobacco_status", &["current"]).done(),
        B::new("vaping", "expositions", "expositions_details", "Vapotage", "Vaping", Choice).opts(NPC).done(),
        B::new("alcohol_status", "expositions", "expositions_details", "Alcool", "Alcohol", Choice).opts(&[("none_reported", "Aucun déclaré"), ("past", "Ancien"), ("current", "Actuel")]).done(),
        B::new("alcohol_units_week", "expositions", "expositions_details", "Unités d'alcool par semaine", "Alcohol units per week", Number).num(0.0, 300.0, false, Some("unités / sem")).range().show_if("alcohol_status", &["current"]).done(),
        B::new("other_substances_status", "expositions", "expositions_details", "Autres substances", "Other substances", Choice).opts(NPC).done(),
        B::new("other_substances_text", "expositions", "expositions_details", "Autres substances : précisions", "Other substances (text)", Text).done(),
        B::new("medications_text", "expositions", "expositions_details", "Traitements en cours", "Current medications", Text).wide().hint("Notamment traitements potentiellement sialoprives").done(),
        // ---------- Examen
        B::new("panoramic_review_status", "examen", "examen_main", "Panoramique", "Panoramic radiograph", Choice)
            .opts(&[("reviewed", "Consultée"), ("not_available", "Non disponible"), ("not_reviewed", "Non consultée")]).essential().missing(&[]).done(),
        B::new("panoramic_date", "examen", "examen_main", "Date du cliché", "Panoramic date", Date).show_if("panoramic_review_status", &["reviewed"]).missing(&["unknown"]).done(),
        B::new("panoramic_source", "examen", "examen_main", "Origine du cliché", "Panoramic source", Choice)
            .opts(&[("existing_record", "Dossier existant"), ("performed_in_pathway", "Réalisée dans le parcours"), ("other", "Autre")]).show_if("panoramic_review_status", &["reviewed"]).missing(&["unknown"]).done(),
        B::new("panoramic_interpretability", "examen", "examen_main", "Lisibilité", "Interpretability", Choice)
            .opts(&[("usable", "Exploitable"), ("partial", "Partiellement exploitable"), ("not_usable", "Non exploitable")]).show_if("panoramic_review_status", &["reviewed"]).missing(EXAM).done(),
        B::new("panoramic_findings", "examen", "examen_main", "Constats radiographiques", "Radiographic findings", Multi)
            .opts(&[("none_noted", "Aucun constat particulier"), ("caries_suspected", "Carie suspectée"), ("periapical_suspected", "Lésion périapicale suspectée"), ("bone_loss", "Perte osseuse"), ("other", "Autre")])
            .excl(&["none_noted"]).show_if("panoramic_review_status", &["reviewed"]).missing(EXAM).done(),
        B::new("panoramic_note", "examen", "examen_main", "Note radiographique", "Radiographic note", Text).wide().show_if("panoramic_review_status", &["reviewed"]).hint("Dent / localisation si utile").done(),
        B::new("intraoral_exam_status", "examen", "examen_main", "Examen endobuccal", "Intraoral examination", Choice)
            .opts(&[("done", "Réalisé"), ("partial", "Partiel"), ("not_done", "Non réalisé")]).essential().missing(&[]).done(),
        B::new("caries_findings", "examen", "examen_constats", "Caries", "Caries", Choice).opts(&[("none_noted", "Aucune notée"), ("suspected", "Suspectées"), ("observed", "Observées")]).missing(EXAM).done(),
        B::new("mechanical_wear_signs", "examen", "examen_constats", "Autres usures (signes mécaniques)", "Mechanical wear signs", Choice).opts(YES_NO).missing(EXAM).done(),
        B::new("plaque_gingival_findings", "examen", "examen_constats", "Plaque / inflammation gingivale", "Plaque / gingival findings", Choice).opts(&[("present", "Présentes"), ("absent", "Absentes")]).missing(EXAM).done(),
        B::new("exam_note", "examen", "examen_constats", "Constats endobuccaux / radiographiques", "Examination note", Text).wide().hint("Localisation, description…").done(),
        B::new("mucosal_findings", "examen", "examen_details", "Muqueuses", "Mucosal findings", Choice).opts(&[("none_noted", "Rien de particulier noté"), ("present", "Lésion notée")]).missing(EXAM).done(),
        B::new("extraoral_findings", "examen", "examen_details", "Examen exobuccal", "Extraoral findings", Multi)
            .opts(&[("none_noted", "Rien de particulier noté"), ("sialadenosis", "Sialadénose"), ("asymmetry", "Asymétrie"), ("other", "Autre")]).excl(&["none_noted"]).missing(EXAM).done(),
        B::new("dmft_d", "examen", "examen_details", "CAO — C (cariées)", "DMFT — D", Number).num(0.0, 32.0, true, Some("dents")).missing(EXAM).done(),
        B::new("dmft_m", "examen", "examen_details", "CAO — A (absentes pour carie)", "DMFT — M", Number).num(0.0, 32.0, true, Some("dents")).missing(EXAM).done(),
        B::new("dmft_f", "examen", "examen_details", "CAO — O (obturées)", "DMFT — F", Number).num(0.0, 32.0, true, Some("dents")).missing(EXAM).done(),
        B::new("recession_note", "examen", "examen_details", "Récessions (sites, mm si mesurés)", "Recessions", Text).wide().done(),
        // ---------- Prévention
        B::new("hbd_teaching", "prevention", "prevention_main", "Enseignement HBD", "Oral hygiene instruction", Choice).opts(&[("done", "Réalisé"), ("not_done", "Non réalisé")]).custom().missing(&[]).done(),
        B::new("prevention_protocol", "prevention", "prevention_main", "Protocole proposé", "Prevention protocol", Choice)
            .opts(&[("none", "Aucun protocole proposé"), ("moderate", "Modéré"), ("advanced", "Avancé"), ("custom", "Personnalisé")]).custom().essential().missing(&[]).done(),
        B::new("bruxism_tray_existing", "prevention", "prevention_main", "Gouttière de bruxisme existante", "Existing bruxism splint", Choice).opts(YES_NO).custom().done(),
        B::new("prevention_note", "prevention", "prevention_main", "Adaptation / précisions", "Prevention note", Text).custom().done(),
        B::new("other_action", "prevention", "prevention_main", "Autre action (information, coordination…)", "Other action", Text).custom().done(),
        // ---------- Observations
        B::new("consultation_observations", "observations", "observations_main", "Observations", "Consultation observations", Text).wide().hint("Contexte, observations complémentaires, points à retenir…").done(),
        B::new("care_need", "observations", "observations_orientation", "Besoins de soins identifiés", "Care needs", Multi)
            .opts(&[("urgent_pain", "Urgence / douleur"), ("caries", "Carie"), ("periodontal", "Parodonte"), ("wear_specialist", "Usure / avis spécialisé"), ("other", "Autre"), ("none_identified", "Aucun besoin identifié")])
            .excl(&["none_identified"]).essential().missing(&["not_assessable"]).done(),
        B::new("referral_destination", "observations", "observations_orientation", "Orientation", "Referral", Choice)
            .opts(&[("none", "Aucune orientation"), ("general_dentist", "Chirurgien-dentiste de ville"), ("hospital_dental", "Service dentaire hospitalier"), ("specialist", "Spécialiste"), ("other", "Autre")]).missing(REC).done(),
        B::new("referral_status", "observations", "observations_orientation", "Statut de l'orientation", "Referral status", Choice)
            .opts(&[("proposed", "Proposée"), ("accepted", "Acceptée"), ("appointment_reported", "Rendez-vous rapporté"), ("confirmed", "Consultation confirmée")])
            .show_if("referral_destination", &["general_dentist", "hospital_dental", "specialist", "other"]).missing(&["unknown"]).done(),
        B::new("referral_note", "observations", "observations_orientation", "Orientation : précisions", "Referral note", Text).wide().done(),
        // ---------- Historique (reprise)
        B::new("age_band_legacy", "historique", "historique_main", "Classe d'âge (texte source)", "Legacy age band (raw)", Text).legacy().done(),
        B::new("age_band_min", "historique", "historique_main", "Classe d'âge : borne basse", "Legacy age band min", Number).num(0.0, 110.0, true, Some("ans")).legacy().done(),
        B::new("age_band_max", "historique", "historique_main", "Classe d'âge : borne haute", "Legacy age band max", Number).num(0.0, 110.0, true, Some("ans")).legacy().done(),
        B::new("vomiting_legacy_code", "historique", "historique_main", "Vomissements (ancienne colonne)", "Legacy vomiting yes/no", Choice).opts(YES_NO).legacy()
            .hint("« Non » ne signifie pas « jamais » : temporalité inconnue").done(),
        B::new("diet_risk_legacy", "historique", "historique_main", "Alimentation (ancien codage)", "Legacy diet coding", Choice)
            .opts(&[("erosive", "Avec risque érosif"), ("no_erosive", "Sans risque érosif associé"), ("caries_risk", "Risque carieux élevé"), ("other", "Autre")]).legacy().done(),
        B::new("hygiene_legacy", "historique", "historique_main", "Hygiène (ancien jugement)", "Legacy hygiene judgement", Choice)
            .opts(&[("sufficient", "Suffisante"), ("insufficient", "Insuffisante"), ("iatrogenic", "Iatrogène")]).legacy().done(),
        B::new("care_legacy", "historique", "historique_main", "Soins (ancien libellé)", "Legacy care label", Choice)
            .opts(&[("specialized_followup", "Suivi spécialisé"), ("nonspecialized_followup", "Suivi non spécialisé"), ("specialized_care", "Soins spécialisés"), ("nonspecialized_care", "Soins non spécialisés")]).legacy().done(),
        B::new("prevention_legacy", "historique", "historique_main", "Prévention (ancien libellé)", "Legacy prevention label", Choice)
            .opts(&[("hbd", "Enseignement HBD"), ("moderate", "Modéré"), ("advanced", "Avancé"), ("other", "Autre")]).legacy()
            .hint("Composantes inconnues : rien n'est reconstruit").done(),
        B::new("prevention_protocol_legacy_raw", "historique", "historique_main", "Prévention (texte source)", "Legacy prevention (raw)", Text).legacy().done(),
        B::new("other_wear_legacy", "historique", "historique_main", "Autres usures (ancienne colonne)", "Legacy other wear", Choice).opts(YES_NO).legacy().done(),
        B::new("dmft_total_historical", "historique", "historique_main", "CAO total historique", "Historical DMFT total", Number).num(0.0, 32.0, true, None).legacy().done(),
        B::new("fiche_pages_text", "historique", "historique_main", "Fiche de consultation Pages (texte intégral)", "Pages consultation form (full text)", Text).legacy().wide().done(),
        B::new("observations_legacy", "historique", "historique_main", "Observations (ancien tableau)", "Legacy observations", Text).wide().legacy().done(),
    ];
    Catalog { form_version: FORM_VERSION, protocol_version: PROTOCOL_VERSION, sections, fields }
}

pub fn catalog() -> &'static Catalog {
    static C: OnceLock<Catalog> = OnceLock::new();
    C.get_or_init(build)
}

pub fn field(code: &str) -> Option<&'static Field> {
    catalog().fields.iter().find(|f| f.code == code)
}

/// Libellés des catégories d'exposition alimentaire (contrat 02).
pub const DRINKS: [(&str, &str); 7] = [
    ("soda_sugar", "Sodas sucrés"),
    ("soda_diet", "Sodas sans sucre"),
    ("fruit_juice", "Jus de fruits"),
    ("energy_drink", "Boissons énergisantes"),
    ("sports_drink", "Boissons pour sportifs"),
    ("acidic_water", "Eau citronnée / aromatisée acide"),
    ("other", "Autre"),
];
pub const FOODS: [(&str, &str); 6] = [
    ("citrus", "Agrumes"),
    ("acid_fruits", "Autres fruits acides"),
    ("vinegar", "Vinaigre / vinaigrettes"),
    ("pickles", "Pickles / aliments marinés"),
    ("sour_candies", "Bonbons acidulés"),
    ("other", "Autre"),
];

pub fn exposure_categories(group: &str) -> Option<&'static [(&'static str, &'static str)]> {
    match group {
        "drink" => Some(&DRINKS),
        "food" => Some(&FOODS),
        _ => None,
    }
}

/// Actions de prévention (contrat 02). HBD est un champ indépendant, pas une composante de protocole.
pub const PREVENTION_ACTIONS: [(&str, &str); 5] = [
    ("anti_erosion_toothpaste", "Dentifrice anti-érosion"),
    ("anti_erosion_rinse", "Bain de bouche anti-érosion après vomissement"),
    ("tooth_mousse", "Tooth Mousse quotidien"),
    ("vomiting_semirigid_tray", "Gouttières semi-rigides pendant les vomissements"),
    ("other", "Autre mesure"),
];

pub fn protocol_components(protocol: &str) -> &'static [&'static str] {
    match protocol {
        "moderate" => &["anti_erosion_toothpaste", "anti_erosion_rinse"],
        "advanced" => &["anti_erosion_toothpaste", "anti_erosion_rinse", "tooth_mousse"],
        _ => &[],
    }
}
