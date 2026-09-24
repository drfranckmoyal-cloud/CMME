#!/usr/bin/env python3
"""Recalcul du rapport descriptif BEWE à partir des seuls fichiers d'un export CMME figé.

Usage : python3 rapport_descriptif.py <dossier_export>

Aucune dépendance hors bibliothèque standard. Ne lit ni la base ni aucun fichier nominatif.
Vérifie d'abord les empreintes SHA-256 annoncées par manifest.json.
"""
import csv
import hashlib
import json
import math
import sys
from pathlib import Path


def quantile(sorted_vals, p):
    """Interpolation linéaire (méthode NumPy par défaut)."""
    if not sorted_vals:
        return None
    h = (len(sorted_vals) - 1) * p
    lo, hi = math.floor(h), math.ceil(h)
    return sorted_vals[lo] + (h - lo) * (sorted_vals[hi] - sorted_vals[lo])


def wilson(k, n, z=1.959963984540054):
    if n == 0:
        return None
    p = k / n
    denom = 1 + z * z / n
    centre = (p + z * z / (2 * n)) / denom
    half = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / denom
    return [100 * max(0.0, centre - half), 100 * min(1.0, centre + half)]


def category(t):
    return "0-2" if t <= 2 else "3-8" if t <= 8 else "9-13" if t <= 13 else "14-18"


def main(folder):
    d = Path(folder)
    manifest = json.loads((d / "manifest.json").read_text(encoding="utf-8"))
    for name, expected in manifest["sha256"].items():
        got = hashlib.sha256((d / name).read_bytes()).hexdigest()
        if got != expected:
            sys.exit(f"Empreinte différente pour {name} : fichier modifié depuis l'export")
    with open(d / "bewe.csv", newline="", encoding="utf-8") as f:
        rows = [r for r in csv.DictReader(f) if r["is_index"] == "1"]
    values = sorted(int(r["analysis_value"]) for r in rows if r["analysis_value"] != "")
    n = len(values)
    ge9 = sum(1 for v in values if v >= 9)
    out = {
        "index_visits": len(rows),
        "n_analysable": n,
        "median": quantile(values, 0.5),
        "q1": quantile(values, 0.25),
        "q3": quantile(values, 0.75),
        "categories": {c: sum(1 for v in values if category(v) == c) for c in ["0-2", "3-8", "9-13", "14-18"]},
        "ge9": {"k": ge9, "n": n, "ci95": wilson(ge9, n)},
        "gt0": {"k": sum(1 for v in values if v > 0), "n": n},
    }
    if n == 0:
        out["message"] = "aucune donnée analysable"
    print(json.dumps(out, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
