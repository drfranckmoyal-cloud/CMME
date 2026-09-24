#!/usr/bin/env python3
"""Extrait le texte principal d'un fichier Pages (.pages) sans ouvrir Pages.

Le fichier est une archive zip ; Index/Document.iwa contient des blocs compressés (Snappy brut, format
« IWA » d'Apple) de messages protobuf. Le corps du document est la plus longue chaîne UTF-8 qui contient
le repère donné. Bibliothèque standard uniquement.

Usage : python3 pages_en_texte.py <liste_relative.txt> <racine_source> <racine_sortie>
"""
import sys
import zipfile
from pathlib import Path

MARQUEUR = "valuation dentaire"


def varint(buf, i):
    shift = result = 0
    while True:
        b = buf[i]
        i += 1
        result |= (b & 0x7F) << shift
        if not b & 0x80:
            return result, i
        shift += 7


def snappy_raw(data):
    """Décompression Snappy « raw » (sans cadre), implémentation de référence minimale."""
    n, i = varint(data, 0)
    out = bytearray()
    while i < len(data):
        tag = data[i]
        i += 1
        kind = tag & 3
        if kind == 0:  # littéral
            ln = tag >> 2
            if ln >= 60:
                nb = ln - 59
                ln = int.from_bytes(data[i:i + nb], "little")
                i += nb
            ln += 1
            out += data[i:i + ln]
            i += ln
            continue
        if kind == 1:
            ln = ((tag >> 2) & 7) + 4
            off = ((tag >> 5) << 8) | data[i]
            i += 1
        elif kind == 2:
            ln = (tag >> 2) + 1
            off = int.from_bytes(data[i:i + 2], "little")
            i += 2
        else:
            ln = (tag >> 2) + 1
            off = int.from_bytes(data[i:i + 4], "little")
            i += 4
        start = len(out) - off
        for k in range(ln):  # copie octet par octet (recouvrement possible)
            out.append(out[start + k])
    if len(out) != n:
        raise ValueError("taille décompressée inattendue")
    return bytes(out)


def iwa_decode(raw):
    out = bytearray()
    i = 0
    while i < len(raw):
        if raw[i] != 0:
            raise ValueError("bloc IWA inattendu")
        ln = int.from_bytes(raw[i + 1:i + 4], "little")
        out += snappy_raw(raw[i + 4:i + 4 + ln])
        i += 4 + ln
    return bytes(out)


def corps(buf):
    marker = MARQUEUR.encode()
    best = None
    pos = buf.find(marker)
    while pos != -1:
        # Remonter vers une en-tête de champ protobuf (type 2) dont la longueur couvre le repère.
        for start in range(max(0, pos - 12), pos):
            try:
                ln, j = varint(buf, start + 1)
            except IndexError:
                continue
            if buf[start] & 7 == 2 and j <= pos and j + ln >= pos + len(marker) and j + ln <= len(buf):
                try:
                    s = buf[j:j + ln].decode("utf-8")
                except UnicodeDecodeError:
                    continue
                if best is None or len(s) > len(best):
                    best = s
        pos = buf.find(marker, pos + 1)
    if best is None:
        raise ValueError("corps du document introuvable")
    # Séparateurs de paragraphe d'Apple → sauts de ligne.
    return best.replace(" ", "\n").replace(" ", "\n").replace("￼", "")


def texte(path):
    with zipfile.ZipFile(path) as z:
        return corps(iwa_decode(z.read("Index/Document.iwa")))


def main(liste, src, dst):
    ok = ko = 0
    for rel in Path(liste).read_text(encoding="utf-8").splitlines():
        out = Path(dst) / (rel[:-6] + ".txt")
        try:
            t = texte(Path(src) / rel)
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(t, encoding="utf-8")
            ok += 1
        except Exception as e:  # noqa: BLE001 — on compte et on continue
            ko += 1
            print(f"ÉCHEC {rel.rsplit('/', 1)[0]} : {type(e).__name__}")
    print(f"extraits={ok} échecs={ko}")


if __name__ == "__main__":
    main(*sys.argv[1:4])
