// Mise en forme française : dates, nombres, libellés. Aucune valeur n'est inventée.

export const fmtNum = (n: number | null | undefined, digits = 1): string => {
  if (n === null || n === undefined || Number.isNaN(n)) return "—";
  return Number.isInteger(n) ? String(n) : n.toLocaleString("fr-FR", { maximumFractionDigits: digits });
};

export const fmtPct = (p: number | null | undefined): string =>
  p === null || p === undefined ? "—" : `${p.toLocaleString("fr-FR", { maximumFractionDigits: 1, minimumFractionDigits: 1 })} %`;

/** Date clinique ISO partielle → affichage français, précision conservée. */
export function fmtClinicalDate(iso: string | null | undefined): string {
  if (!iso) return "—";
  const [y, m, d] = iso.split("-");
  if (d) return `${d}/${m}/${y}`;
  if (m) return `${m}/${y}`;
  return y;
}

/** Saisie au format européen JJ/MM/AAAA uniquement → ISO. null si invalide. */
export function parseFrenchDate(s: string): string | null {
  const m = s.trim().match(/^(\d{1,2})\/(\d{1,2})\/(\d{4})$/);
  if (!m) return null;
  const [, d, mo, y] = m;
  const iso = `${y}-${mo.padStart(2, "0")}-${d.padStart(2, "0")}`;
  const dt = new Date(`${iso}T12:00:00`);
  return dt.getFullYear() === +y && dt.getMonth() + 1 === +mo && dt.getDate() === +d ? iso : null;
}

export const fmtTime = (iso: string | null | undefined): string =>
  iso ? new Date(iso).toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit", second: "2-digit" }) : "";

export const fmtDateTime = (iso: string | null | undefined): string =>
  iso ? new Date(iso).toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "short" }) : "—";

/** Nombre saisi en français (virgule) → nombre, ou null si vide ; NaN si invalide. */
export function parseFrNumber(s: string): number | null {
  const t = s.trim().replace(/\s/g, "").replace(",", ".");
  if (!t) return null;
  if (!/^-?\d+(\.\d+)?$/.test(t)) return NaN;
  return Number(t);
}

export const numToInput = (n: number | null | undefined): string =>
  n === null || n === undefined ? "" : String(n).replace(".", ",");

export const SERVICE_LABELS: Record<string, string> = {
  centre_expert: "Centre expert", sas: "SAS", hospit_complete: "Hospitalisation complète",
  hdj: "HDJ", hdj_intensif: "HDJ intensif", autre: "Autre",
};
export const VOMITING_LABELS: Record<string, string> = {
  never_reported: "Jamais rapportés", past_only: "Anciens", current: "Actuels",
  legacy_yes: "Oui (ancien codage)", legacy_no: "Non (ancien codage)",
  unknown: "Inconnu", declined: "Refus", not_applicable: "Non applicable", not_asked: "Non renseigné", not_recorded: "Non consigné",
};
export const PROTOCOL_LABELS: Record<string, string> = {
  none: "Aucun protocole", moderate: "Modéré", advanced: "Avancé", custom: "Personnalisé",
  hbd: "HBD (ancien)", other: "Autre (ancien)", not_asked: "Non renseigné", not_recorded: "Non consigné",
};
export const STATUS_LABELS: Record<string, string> = { draft: "Brouillon", validated: "Validée", amending: "Correction en cours" };
export const ORIGIN_LABELS: Record<string, string> = {
  derived: "six sextants", historical: "total historique", conflict: "conflit à arbitrer", incomplete: "sextants incomplets", missing: "non disponible",
};
