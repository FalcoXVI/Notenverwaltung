# Erklärung: Architektur und Rust-Konzepte

Dieses Dokument erklärt dir, wie die Notenverwaltung aufgebaut ist und welche
Rust-Konzepte darin vorkommen. Jedes Konzept wird mit C verglichen und zeigt auf eine
konkrete Stelle im Code (`Datei:Zeile`). Die Zeilennummern können sich nach Änderungen
leicht verschieben, die genannten Funktionsnamen findest du über die Suche.

## Teil 1: Architektur

### Die vier Schichten

```
ui  ──►  db, export  ──►  calc  ──►  domain
```

Abhängigkeiten zeigen nur nach rechts. `calc` weiß nichts von Fenstern oder Dateien,
`domain` weiß von gar nichts außer sich selbst.

| Ordner | Aufgabe | Wichtigste Dateien |
|---|---|---|
| `src/domain/` | Reine Datentypen: Note, Punkte, Eintrag, Kategorie, Fach, Semester | `grade.rs`, `entry.rs`, `grading_key.rs` |
| `src/calc/` | Die gesamte Rechenlogik als reine Funktionen, samt Tests | `category.rs`, `subject.rs`, `what_if.rs` |
| `src/db/` | Alles mit SQLite: Schema, Migrationen, Laden, Speichern, Sicherung | `database.rs`, `loading.rs`, `schema_v1.sql` |
| `src/export/` | CSV-Export | `csv_export.rs` |
| `src/ui/` | Fenster, Seiten und Dialoge mit egui | `app.rs`, `store.rs`, `subject_view.rs` |

`src/main.rs` ist nur der Einstiegspunkt (wie `main()` in C) und ruft `ui::run()` auf
(`src/ui/mod.rs:34`). `src/lib.rs` listet die fünf Module auf.

### Warum diese Trennung?

Die Rechenlogik lässt sich ohne Fenster und ohne Datenbank testen. `cargo test` prüft
72 Fälle in unter einer Sekunde, darunter alle Grenzfälle aus der Aufgabenstellung
(genau 4,0 gegen 4,01; 48 gegen 49 von 96 Punkten; beste N; leere Kategorien).

### Der Weg einer Note durch die Rechnung

1. `entry_grade` (`src/calc/grading.rs:14`) macht aus einem einzelnen Eintrag eine Note.
2. `select_counted` (`src/calc/best_n.rs:12`) entscheidet, welche Einträge zählen.
3. `evaluate_values` (`src/calc/category.rs:32`) berechnet die Kategorie-Note.
4. `weighted_average` (`src/calc/subject.rs:40`) gewichtet die Kategorien zum Fachschnitt.
5. `round_up` (`src/calc/subject.rs:66`) rundet auf die ganze Note auf.

Das Was-wäre-wenn (`src/calc/what_if.rs:39`) rechnet nichts Eigenes. Es probiert jedes
mögliche Ergebnis der nächsten Leistung vom schlechtesten zum besten durch und schickt
es durch genau diese Kette. Dadurch stimmen „beste N“ und alle anderen Regeln
automatisch.

### Wie die Oberfläche arbeitet

egui ist eine „Immediate Mode“-GUI. Es gibt keine Button-Objekte, die irgendwo leben.
Stattdessen wird das ganze Fenster viele Male pro Sekunde neu beschrieben. Der Ablauf
pro Bild steht in `App::ui` (`src/ui/app.rs:168`):

1. Die Seiten lesen die Daten aus dem `Store` und zeichnen sich.
2. Was du anklickst, wird als `Action` (`src/ui/action.rs:10`) in eine Liste gelegt.
3. Nach dem Zeichnen arbeitet `App::handle` (`src/ui/app.rs:100`) die Liste ab.

Änderungen an den Daten sind als `Command` beschrieben (`src/ui/command.rs:13`).
Es gibt genau eine Stelle, die in die Datenbank schreibt: `Store::write`
(`src/ui/store.rs:73`). Danach lädt `Store::reload` alles neu. Bildschirm und Datei
können so nie auseinanderlaufen.

### Datensicherheit

- Jede Änderung, die mehrere Zeilen betrifft, läuft in einer Transaktion
  (`src/db/subjects.rs:15`). Entweder wird alles gespeichert oder nichts.
- Die Datenbank prüft selbst, dass Noten zwischen 1 und 5 liegen und ein Eintrag
  entweder Note oder Punkte ist (`CHECK` in `src/db/schema_v1.sql`).
- Schemaänderungen laufen über nummerierte Migrationen (`src/db/migrations.rs:8`).
  Für Version 2 hängst du eine neue SQL-Datei an die Liste an, die bestehende bleibt
  unverändert.
- Beim Start wird zuerst gesichert, dann migriert (`src/db/database.rs:31`).

## Teil 2: Rust-Konzepte im Vergleich zu C

### Structs

Wie in C ein Verbund aus Feldern. Der Unterschied: Funktionen, die zum Struct gehören,
stehen in einem `impl`-Block und werden mit Punkt aufgerufen.

| C | Rust |
|---|---|
| `struct subject { ... };` | `pub struct Subject { ... }` |
| `int subject_is_open_negative(const struct subject *s)` | `fn is_open_negative(&self) -> bool` |
| `subject_is_open_negative(&s)` | `s.is_open_negative()` |

Stelle: `src/domain/subject.rs:25`. `&self` ist der Zeiger auf das eigene Struct, den
du in C von Hand als ersten Parameter übergibst.

### Enums mit Daten

Ein C-`enum` ist nur eine Zahl. Ein Rust-`enum` kann pro Variante eigene Daten tragen.
In C würdest du dafür ein `struct` mit einem `tag` und einer `union` bauen und müsstest
selbst aufpassen, das richtige Feld zu lesen.

```rust
pub enum EntryValue {
    Grade(Grade),
    Points { achieved: Points, max: Points, actual_grade: Option<Grade> },
}
```

Stelle: `src/domain/entry.rs:10`. Ein Eintrag ist entweder eine Note oder Punkte.
„Note mit Maximalpunkten“ lässt sich gar nicht hinschreiben, der Compiler verhindert es.

Das einfache `enum Grade` (`src/domain/grade.rs:9`) hat nur die Werte 1 bis 5. Eine
„Note 7“ kann nirgends im Programm existieren. Die einzige Stelle, an der eine Zahl zur
Note wird, ist `Grade::from_number` (`src/domain/grade.rs:28`).

### `match`

Wie `switch` in C, mit zwei Unterschieden: Es gibt kein Durchfallen, und der Compiler
verlangt, dass jeder Fall behandelt wird. Kommt später eine dritte Variante zu
`EntryValue` dazu, übersetzt das Programm erst wieder, wenn jedes `match` sie kennt.

Stelle: `src/calc/grading.rs:16`. Dort werden die Daten gleich mit ausgepackt
(`EntryValue::Grade(grade) => ...`).

### `Option` statt NULL

In C zeigt ein Zeiger auf etwas oder ist `NULL`, und nichts zwingt dich zur Prüfung.
In Rust gibt es kein `NULL`. Ein Wert, der fehlen kann, hat den Typ `Option<T>` mit
den Varianten `Some(wert)` und `None`. An den Wert kommst du nur, indem du beide Fälle
behandelst.

- `report_grade: Option<Grade>` in `src/domain/subject.rs`: Die Zeugnisnote ist
  vielleicht noch nicht eingetragen.
- `final_grade` (`src/domain/subject.rs:19`): `self.exam_grade.or(self.report_grade)`
  heißt „nimm die Prüfungsnote, sonst die Zeugnisnote“.
- `let Some(graded) = result.graded else { continue };` (`src/calc/subject.rs:48`):
  Kategorien ohne Note werden übersprungen.

### `Result` und das `?`

In C gibt eine Funktion bei Fehlern meist `-1` zurück, und der Aufrufer muss daran
denken, das zu prüfen. In Rust gibt sie `Result<T, E>` zurück: entweder `Ok(wert)` oder
`Err(fehler)`. Ignorierst du das Ergebnis, warnt der Compiler.

Das Fragezeichen ist die Kurzform für „bei Fehler sofort mit diesem Fehler zurück“:

| C | Rust |
|---|---|
| `if (sqlite3_exec(...) != SQLITE_OK) return -1;` | `tx.execute_batch(sql)?;` |

Stelle: `src/db/migrations.rs:24`. Der eigene Fehlertyp `DbError`
(`src/db/error.rs:5`) ist wieder ein Enum mit Daten. Das `?` wirkt auch auf `Option`,
siehe `src/domain/decimal.rs:23`.

### Ownership (Besitz)

Die wichtigste Idee von Rust. In C rufst du `malloc` auf und musst irgendwann genau
einmal `free` aufrufen. Vergisst du es, hast du ein Speicherleck. Machst du es zweimal
oder benutzt den Zeiger danach weiter, stürzt das Programm ab.

In Rust hat jeder Wert genau einen Besitzer. Endet dessen Gültigkeitsbereich, wird der
Wert automatisch freigegeben. Es gibt kein `free` und keinen Garbage Collector.

- Im Test `data_survives_closing_and_reopening_the_file` (`src/db/tests.rs`) endet ein
  `{ ... }`-Block, die Variable `db` verschwindet, und damit wird die Datei geschlossen.
- Übergibst du einen Wert ohne `&`, wechselt der Besitz („move“). `Store::execute`
  (`src/ui/store.rs:56`) nimmt `command: Command` und verbraucht es. Der Aufrufer kann
  es danach nicht mehr verwenden, der Compiler verbietet es.

Diesen Fehler hat der Compiler während der Entwicklung tatsächlich gemeldet: Ein
`Vec<String>` wurde in eine Liste verschoben und danach noch einmal benutzt. Die Lösung
war ein ausdrückliches `.clone()` (`src/export/csv_export.rs`, Funktion `subject_lines`).
In C wäre das ein stiller Fehler zur Laufzeit gewesen.

### Borrowing (Ausleihen)

Meist will eine Funktion einen Wert nur ansehen, nicht besitzen. Dafür gibt es
Referenzen, im Grunde Zeiger mit Regeln:

| Rust | C-Entsprechung | Regel |
|---|---|---|
| `&T` | `const T *` | beliebig viele gleichzeitig, nur lesen |
| `&mut T` | `T *` | genau eine, und währenddessen keine andere |

Der Compiler prüft außerdem, dass keine Referenz länger lebt als der Wert, auf den sie
zeigt. Ein Zeiger auf eine bereits freigegebene Variable ist damit ausgeschlossen.

- `evaluate_subject(data: &SubjectData)` (`src/calc/subject.rs:25`) liest nur.
- `migrate(conn: &mut Connection)` (`src/db/migrations.rs:24`) verändert die Verbindung.
- Genau diese Regel erklärt den Aufbau der Oberfläche: Während die Seiten den `Store`
  lesend ausgeliehen haben, darf niemand hineinschreiben. Deshalb sammeln sie nur
  `Action`s, und geschrieben wird erst danach.

### Slices und `Vec`

`Vec<T>` ist ein wachsendes Array, das seine Länge kennt (in C: Zeiger, Länge und
Kapazität von Hand plus `realloc`). `&[T]` ist ein Ausschnitt daraus: Zeiger und Länge
in einem. Jeder Zugriff mit `[i]` wird geprüft. Ein Zugriff daneben beendet das
Programm sauber mit einer Meldung, statt fremden Speicher zu lesen.

Stelle: `select_counted(..., values: &[EntryValue], ...)` in `src/calc/best_n.rs:12`.

### Traits

Ein Trait beschreibt eine Fähigkeit, ähnlich einer Schnittstelle. In C käme dem ein
Struct mit Funktionszeigern am nächsten.

- `impl fmt::Display for Grade` (`src/domain/grade.rs:61`) legt fest, wie eine Note als
  Text aussieht. Danach funktioniert `format!("{grade}")`.
- `impl From<rusqlite::Error> for DbError` (`src/db/error.rs:40`) erlaubt dem `?`, einen
  SQLite-Fehler automatisch in den eigenen Fehlertyp umzuwandeln.
- `impl eframe::App for App` (`src/ui/app.rs:168`): eframe ruft die Methode `ui` auf,
  ohne den Typ `App` zu kennen. Es genügt, dass er den Trait erfüllt.
- `#[derive(Debug, Clone, PartialEq, ...)]` über einem Struct lässt den Compiler
  Standard-Traits selbst schreiben: Ausgabe zum Debuggen, Kopieren, Vergleich mit `==`.
  In C müsstest du für jedes Struct eine eigene Vergleichsfunktion schreiben.

### Newtypes: Typen, die Verwechslungen verhindern

In C wären alle IDs `long` und Punkte `long`, und du könntest sie beliebig vertauschen.
Hier bekommt jede Bedeutung einen eigenen Typ, der innen nur eine Zahl ist und zur
Laufzeit nichts kostet:

- `SubjectId`, `CategoryId` und weitere (`src/domain/ids.rs:5`): Wer eine Fach-ID
  übergibt, wo eine Kategorie-ID erwartet wird, bekommt einen Compilerfehler.
- `Points` (`src/domain/points.rs`) speichert Hundertstelpunkte als Ganzzahl.

### Exakte Brüche statt `double`

`0.1 + 0.2` ergibt mit `double` nicht genau `0.3`. Bei Noten wäre das fatal: Ein
Schnitt von genau 4,0 könnte als 4,0000000001 aufgerundet ein Fünfer werden.

`Fraction` (`src/domain/fraction.rs:7`) ist ein Bruch aus zwei Ganzzahlen aus der
Bibliothek `num-rational`. Die Schlüsselgrenzen werden ebenfalls als Bruch verglichen
(`src/domain/grading_key.rs:52`). Im ganzen Ordner `calc` kommt kein `f64` vor.

### Module und Sichtbarkeit

Jede Datei ist ein Modul. Ohne `pub` ist alles privat, wie `static` in einer C-Datei.
`mod.rs` übernimmt die Rolle der Header-Datei: Sie listet die Dateien des Ordners auf
und gibt mit `pub use` frei, was nach außen sichtbar ist (`src/calc/mod.rs`). `use`
entspricht ungefähr `#include`, holt aber nur Namen herein und kopiert keinen Text.

`pub(super)` heißt „sichtbar nur im übergeordneten Modul“. Das Feld `conn` der
Datenbank (`src/db/database.rs:12`) ist so markiert: Nur der Ordner `db` kommt an die
SQLite-Verbindung, die Oberfläche kann kein eigenes SQL absetzen.

### Closures

Kleine namenlose Funktionen, die Variablen aus ihrer Umgebung benutzen dürfen. In C
bräuchtest du einen Funktionszeiger plus einen `void *` für den Kontext.

Stelle: `let outcome = |candidate: EntryValue| ...` in `src/calc/what_if.rs:59` greift
auf die umliegenden Variablen `data` und `key` zu.

### Iteratoren

Statt `for (i = 0; i < n; i++)` verkettet man in Rust oft Schritte:

```rust
let sum: i64 = grades.iter().map(|g| g.grade.number()).sum();
```

Stelle: `src/calc/category.rs:64`. `iter` geht die Elemente durch, `map` wandelt jedes
um, `sum` addiert. Der Compiler macht daraus dieselbe Schleife, die du in C schreiben
würdest, ohne die Möglichkeit eines Indexfehlers.

### Tests

Tests stehen in derselben Datei wie der Code, in einem Block `#[cfg(test)] mod tests`
(`src/calc/subject.rs:71`). `cargo test` übersetzt und startet sie, im fertigen Programm
sind sie nicht enthalten. In C bräuchtest du dafür ein eigenes Testprogramm und ein
Framework.

### Makros

`define_id!` (`src/domain/ids.rs:5`) erzeugt vier gleich aufgebaute ID-Typen. Anders als
`#define` in C arbeitet ein Rust-Makro auf der Syntax, nicht auf rohem Text. Typische
Präprozessor-Fallen wie fehlende Klammern gibt es nicht. `format!`, `vec!` und
`assert_eq!` sind ebenfalls Makros, erkennbar am Ausrufezeichen.

## Teil 3: Wo fange ich an zu lesen?

1. `src/domain/grade.rs`: klein, zeigt Enum, `impl`, `Option` und einen Trait.
2. `src/domain/entry.rs`: das Enum mit Daten.
3. `src/calc/category.rs` und danach `src/calc/subject.rs`: die Kernrechnung. Lies die
   Tests am Dateiende mit, sie sind die beste Dokumentation der Regeln.
4. `src/db/migrations.rs`: kurz, zeigt `Result` und `?`.
5. `src/ui/app.rs`: wie alles zusammenläuft.
