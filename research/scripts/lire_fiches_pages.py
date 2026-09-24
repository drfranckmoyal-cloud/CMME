#!/usr/bin/env python3
"""Lit les fiches « Évaluation dentaire CMME » exportées de Pages en texte brut et produit un JSON structuré.

Usage : python3 lire_fiches_pages.py <dossier_txt> <sortie.json>

Règles : seules les valeurs explicitement écrites sont reprises ; rien n'est déduit. Le texte intégral de
la fiche est conservé. Ce script ne contient ni ne journalise aucune donnée de patient.
"""
import json
import re
import sys
from pathlib import Path

LABELS = [
    "Date", "NOM Prénom", "Age", "Sexe", "Service", "Dernière visite CD", "Occupation", "Cigarette", "Tabac",
    "Alcool", "Brossage", "Alimentation", "TCA (dossier)",
]


def value_after(text, label):
    m = re.search(r"^[ \t]*" + re.escape(label) + r"[ \t]*:[ \t]*(.*)$", text, re.M)
    return m.group(1).strip() if m else None


def block(text, start_label, stop_patterns):
    """Texte entre une rubrique et la suivante (lignes vides retirées)."""
    m = re.search(r"^[ \t]*" + re.escape(start_label) + r"[ \t]*:?(.*)$", text, re.M)
    if not m:
        return None
    rest = [m.group(1).strip()]
    for line in text[m.end():].splitlines():
        if any(re.match(p, line.strip()) for p in stop_patterns):
            break
        rest.append(line.strip())
    out = "\n".join(x for x in rest if x)
    return out or None


def as_int(s, lo, hi):
    if s is None:
        return None
    t = s.strip()
    return int(t) if re.fullmatch(r"\d{1,3}", t) and lo <= int(t) <= hi else None


def parse_date(s):
    if not s:
        return None
    m = re.fullmatch(r"(\d{1,2})/(\d{1,2})/(\d{2}|\d{4})", s.strip())
    if not m:
        return None
    d, mo, y = int(m.group(1)), int(m.group(2)), m.group(3)
    y = int(y) + 2000 if len(y) == 2 else int(y)
    if not (1 <= mo <= 12 and 1 <= d <= 31):
        return None
    return f"{y:04d}-{mo:02d}-{d:02d}"


def months(s):
    """« 2 ans », « 6 mois » → mois (estimation). Toute autre formulation reste du texte."""
    if not s:
        return None
    m = re.fullmatch(r"(\d{1,2})\s*(an|ans|mois)\.?", s.strip().lower())
    if not m:
        return None
    n = int(m.group(1))
    return n * 12 if m.group(2).startswith("an") else n


def service(s):
    """Libellé de service tel qu'écrit, fautes de frappe évidentes corrigées (SAAS, Hopsit)."""
    if not s or not s.strip():
        return None
    t = s.strip()
    low = t.lower()
    if low == "saas":
        return "SAS"
    if low.startswith("hopsit"):
        return "Hospit" + t[6:]
    return t


SECTION_STOPS = [r"^Historique", r"^Examen", r"^Score BEWE", r"^Indice CAO", r"^Recommandations", r"^Anamn"]


def parse(path, root):
    text = path.read_text(encoding="utf-8", errors="replace")
    rel = str(path.relative_to(root))
    f = {"file": rel[:-4] + ".pages", "raw": text}
    for lab in LABELS:
        f[lab] = value_after(text, lab)
    bewe_line = re.search(r"Score BEWE[ \t]*=[ \t]*(.*)$", text, re.M)
    f["bewe_raw"] = bewe_line.group(1).strip() if bewe_line else None
    cao_line = re.search(r"Indice CAO[ \t]*=[ \t]*(.*)$", text, re.M)
    f["cao_raw"] = cao_line.group(1).strip() if cao_line else None
    for k in ("C", "A", "O"):
        m = re.search(r"^[ \t]*" + k + r"[ \t]*=[ \t]*(.*)$", text, re.M)
        f["cao_" + k] = as_int(m.group(1), 0, 32) if m else None
    f["alimentation"] = block(text, "Alimentation", [r"^TCA"] + SECTION_STOPS)
    f["tca"] = block(text, "TCA (dossier)", SECTION_STOPS)
    f["recommandations"] = block(text, "Recommandations", [r"^$^"])
    return {
        "file": f["file"],
        "raw": text,
        "date": parse_date(f["Date"]),
        "date_raw": f["Date"],
        # Nom de la rubrique, sinon nom du fichier (les fiches portent le nom du patient).
        "name": (f["NOM Prénom"] or "").strip() or Path(rel).stem.strip() or None,
        "name_from_filename": not (f["NOM Prénom"] or "").strip(),
        "age": as_int(re.sub(r"\s*a(ns)?\.?$", "", (f["Age"] or "").strip()), 5, 110),
        "age_raw": f["Age"],
        "sex": {"f": "female", "h": "male", "m": "male"}.get((f["Sexe"] or "").strip().lower()),
        "service_raw": service(f["Service"]),
        "last_visit_months": months(f["Dernière visite CD"]),
        "last_visit_raw": f["Dernière visite CD"],
        "occupation": f["Occupation"] or None,
        "tobacco_raw": f["Cigarette"] or f["Tabac"],
        "alcohol_raw": f["Alcool"],
        "brushing_raw": f["Brossage"],
        "diet": f["alimentation"],
        "tca": f["tca"],
        "bewe_raw": f["bewe_raw"] or None,
        "cao_raw": f["cao_raw"] or None,
        "cao_c": f["cao_C"], "cao_a": f["cao_A"], "cao_o": f["cao_O"],
        "recommendations": f["recommandations"],
    }


def quasi_vide(x):
    """Modèle ouvert mais pas rempli : au plus une rubrique clinique renseignée."""
    return sum(1 for k in ("age", "sex", "bewe_raw", "tca", "occupation", "cao_c", "cao_o") if x.get(k)) <= 1


def main(src, dst):
    root = Path(src)
    fiches = [parse(p, root) for p in sorted(root.rglob("*.txt"))]
    for x in fiches:
        x["quasi_vide"] = quasi_vide(x)
    Path(dst).write_text(json.dumps(fiches, ensure_ascii=False, indent=1), encoding="utf-8")
    named = sum(1 for x in fiches if x["name"])
    print(f"{len(fiches)} fiches lues, {named} avec un nom, {sum(1 for x in fiches if x['date'])} datées, "
          f"{sum(1 for x in fiches if x['bewe_raw'])} avec BEWE renseigné")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
