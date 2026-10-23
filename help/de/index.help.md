<!-- TOPIC: getting_started -->
<!-- TITLE: Erste Schritte -->

# Erste Schritte mit FileForge Workbench

FileForge Workbench ist eine befehlsgesteuerte, ISPF-inspirierte
Bearbeitungsumgebung. Dieser Rundgang behandelt die Grundlagen für eine erste
Sitzung.

<!-- DRAFT: AI-DRAFTED German help, PENDING NATIVE-SPEAKER REVIEW (CR-NR-103
Task 10). Only the TITLE and body prose are translated; the TOPIC key and the
topic/feature:/line: cross-reference keys stay unlocalised (Req 6.3).
Untranslated topics fall back to the English help/ directory (Req 6.2). Help
content is user-facing rendered markdown (no Fluent escape processing), so
accented letters are written literally -- the runtime-UI-strings exception to the
ASCII-first documentation rule. -->

## Eine Datei öffnen

- Verwenden Sie das Menü Datei, oder tippen Sie einen Dateipfad in die Zeile
  `Command ===>` und drücken Sie die Eingabetaste.
- Dateien öffnen sich in einem Bearbeitungskontext innerhalb eines Arbeitsbereich-Tabs.

## Grundlegende Navigation

- **F7 / F8** blättern eine Seite nach oben / unten (gesteuert durch den `SCROLL ===>`-Wert).
- **UP** und **DOWN** blättern gemäß dem aktiven SCROLL-Wert.
- **TOP** und **BOTTOM** springen zur ersten / letzten Zeile.
- **LOCATE n** bewegt sich zur Zeilennummer n.

## Bearbeiten

- Im Bearbeitungsmodus tippen Sie direkt in den Textbereich.
- **Zeilenbefehle** im Präfixbereich (zum Beispiel `D` zum Löschen einer Zeile,
  `CC` zum Kopieren eines Blocks) wirken auf einzelne Zeilen.
  Siehe [Line Commands](line:index).
- **UNDO** und **REDO** machen Änderungen rückgängig und wenden sie erneut an.
  Siehe [Undo](feature:undo).

## Speichern

- **SAVE** schreibt den aktuellen Puffer. **CANCEL** verwirft Änderungen. **END** (F3)
  schließt den aktuellen Kontext.

## Die Befehlszeile

- Jede Aktion ist ein Befehl. Menüs, Schaltflächen und Funktionstasten rufen alle
  dieselben Befehle auf, die Sie in die Zeile `Command ===>` tippen können.
- Drücken Sie jederzeit **F1** für kontextsensitive Hilfe zu dem, was Sie gerade tun.
- **F12** (RETRIEVE) ruft vorherige Befehle ab.

## Die Workbench anordnen

- Arbeitsbereiche sind Tabs; ein Arbeitsbereich kann geteilt oder in ein eigenes
  Fenster abgekoppelt werden.
- Siehe [Tabs](feature:tabs) und [Docking](feature:docking).

Siehe auch: [Help Index](index), [Function Keys](feature:function_keys).
