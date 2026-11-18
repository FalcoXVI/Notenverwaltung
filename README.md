# Notenverwaltung

Desktop-App für Schulnoten in einer semestrierten Schulform (Österreich, Noten 1–5).
Geschrieben in Rust mit egui/eframe und SQLite.

## Starten

Die fertige App liegt als `notenverwaltung.exe` in diesem Ordner. Doppelklick genügt,
es ist keine Installation nötig. Die Datei kannst du an einen beliebigen Ort kopieren.

## Selbst bauen

Voraussetzung: Rust (über `rustup`, MSVC-Toolchain) und die Visual Studio Build Tools.
Beides ist auf diesem Rechner eingerichtet.

In einer PowerShell in diesem Ordner:

```powershell
cargo test              # alle Unit-Tests ausführen
cargo run               # Debug-Version bauen und starten (mit Konsolenfenster)
cargo build --release   # optimierte Version bauen
```

Der Release-Build liegt danach unter `target\release\notenverwaltung.exe`.
Der erste Build dauert einige Minuten, weil alle Bibliotheken übersetzt werden.
Danach geht es schnell.

## Wo liegen meine Daten?

| Was | Ort |
|---|---|
| Datenbank | `%APPDATA%\notenverwaltung\noten.db` |
| Automatische Sicherungen | `%APPDATA%\notenverwaltung\backups\` |

Bei jedem Start schreibt die App eine Sicherung in den Backup-Ordner und behält die
letzten 10. Zusätzlich kannst du unter „Einstellungen & Daten“ jederzeit eine Sicherung
an einen Ort deiner Wahl speichern und wiederherstellen. Vor einer Wiederherstellung
wird der aktuelle Stand automatisch gesichert.

## Bedienung in Kürze

1. **Semester anlegen**, leer oder als Kopie eines bestehenden (Fächer, Kategorien,
   Gewichte und Notenschlüssel werden übernommen, Einträge nicht).
2. **Fach anlegen**: Kategorien mit Gewicht (Summe 100 %), Typ „Punkte“ oder
   „Note 1–5“, optional „nur die besten N zählen“. Die Positiv-Grenze und, sobald
   bekannt, den echten Notenschlüssel trägst du im selben Dialog ein.
3. **Fach öffnen** (Klick auf den Namen): Einträge erfassen, Zeugnisnote und
   Semesterprüfung nachtragen, rechts das Was-wäre-wenn.
4. **Offene negative Fächer**: alle Fächer mit Zeugnisnote 5 ohne positive
   Semesterprüfung, über alle Semester.

Ein `~` vor einer Note heißt „geschätzt“: Sie hängt vom angenommenen Notenschlüssel
ab, weil der echte beim Fach noch nicht eingetragen ist.

## Rechenregeln

- **Punkte-Kategorie:** Summe erreicht / Summe maximal, daraus per Notenschlüssel die
  Note. Hat mindestens ein zählender Eintrag eine tatsächliche Note, wird jeder Eintrag
  einzeln in eine Note umgerechnet und gemittelt.
- **Noten-Kategorie:** Durchschnitt der zählenden Noten.
- **Beste N:** besser ist die bessere Note (tatsächliche, sonst per Schlüssel), bei
  Gleichstand der höhere Prozentsatz.
- **Grenzen** gelten immer als „über“: genau 50 % ist bei Grenze 50 noch negativ.
- **Fachschnitt:** gewichteter Durchschnitt der Kategorie-Noten, leere Kategorien
  fallen heraus. Die aktuelle Note ist der aufgerundete Schnitt (4,0 → 4; 4,01 → 5).
- **Geschätzt** ist nur, was wirklich vom unbekannten Schlüssel abhängt. Ein Fünfer
  ist immer exakt, weil die Positiv-Grenze bekannt ist.
- Gerechnet wird ausschließlich mit Brüchen aus Ganzzahlen, nie mit Kommazahlen.

Wie der Code aufgebaut ist und welche Rust-Konzepte darin vorkommen, steht in
[ERKLAERUNG.md](ERKLAERUNG.md).
