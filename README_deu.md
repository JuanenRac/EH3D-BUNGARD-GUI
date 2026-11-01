<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-BUNGARD-GUI banner" width="100%">
</p>

# 🖥️ EH3D-BUNGARD-GUI

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | 🇩🇪 <b>Deutsch</b> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### Das moderne Fenster für die Bungard-CCD-Maschinen: ein Starter, eine 2D- und eine 3D-Ansicht des Auftrags, Bohren und Fräsen zusammen, in 37 Sprachen

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Tests-198-0f6cbd.svg" alt="Tests">
  <img src="https://img.shields.io/badge/Version-0.1.8-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Ehrlichkeitscheck - was heute tatsächlich läuft:** ein Rust-Programm mit einem Fenster und 198 Tests, die auf einem Rechner laufen. Der Starter bietet die moderne Oberfläche oder die klassischen Bohr- und Fräsprogramme für eine Maschine mit manuellem (MTC) oder automatischem (ATC) Werkzeugwechsel. Der Betrachter öffnet Bohrdateien, HP-GL-Plotdateien und ZIP-Archive mit Aufträgen (aus einem Ordnerbrowser, per Drag and Drop oder von der Befehlszeile), zeichnet die Draufsicht mit jedem Loch nach Bohrergröße und den Fräsbahnen darüber, zeigt die Maschine in 3D mit dem in der Konfiguration gewählten Tisch, bildet Nutzen und verfolgt den Auftrag während des Laufs. Der Auftrag läuft im **Simulator** der Engines von EH3D-DRILLPRO und EH3D-ROUTEPRO; dieselbe Maschinendatei, die sie lesen, bestimmt, wie eine echte Maschine erreicht wird (die RS-232-Schnittstelle des Rechners, ein USB-RS232-Adapter oder der EH3D-NODE-BUNGARD direkt oder über EH3D-SERVER), und **keiner dieser Wege wurde von diesem Fenster aus ausprobiert**. Die Fenster für Verbindung, Einstellungen und Werkzeuge ändern die Werte, die sie anbieten, die vollständigen Maschinen- und Werkzeugdateien lassen sich noch nicht bearbeiten, und die 3D-Ansicht zeigt die Höhe der Bahnen nicht.

---

> **Stand von Sicherheit und IT-Sicherheit:** dieses Programm ist kein Sicherheitssystem. Es behandelt weder Endschalter noch einen Not-Halt und hat nie eine echte Maschine bewegt. Mit einer echten Verbindung in der Maschinendatei würde der Auftrag die Maschine über die Engines von EH3D-DRILLPRO und EH3D-ROUTEPRO bewegen: lies zuerst deren Hinweise, nimm das Werkzeug heraus, halte eine Hand am Netzschalter der Maschine und sei bereit, den Auftrag zu stoppen. Das Projekt ist unabhängig: es wird nicht von Bungard Elektronik hergestellt, empfohlen oder unterstützt, und Produktnamen erscheinen nur, um zu sagen, mit welchen Dateiformaten und Maschinen es arbeiten soll.

## 📋 Auf einen Blick

| | |
|---|---|
| Version | `0.1.8` |
| Reifegrad | Gerüst |
| Tests | 198 |
| Technologie-Stack | Rust |
| Auf echter Hardware | Nein. Es lief nur im Simulator und auf einem Rechner; von diesem Fenster aus wurde keine Maschine betrieben |

**Springe zu:** [Überblick](#-überblick) · [Architektur](#-architektur) · [Schnellstart](#-schnellstart) · [Tests](#-tests) · [Struktur](#-struktur-des-repositorys) · [Verwandte](#-verwandte-projekte)

## 🚦 Projektstatus

Jede Zeile ist ein Teil dieses Repositorys, und die Ampel sagt, wie weit er heute ist. Sie beruht auf dem, was das Repository zeigen kann (Tests, Builds, Läufe), nicht auf dem, was geplant ist.

🟢 Fertig und überprüft · 🟡 Funktioniert, mit bekannten Grenzen · 🟠 Teilweise fertig oder noch nicht überprüft · 🔴 Nicht gemacht oder funktioniert nicht

| Bereich | Stand | Was das hier bedeutet |
|---|---|---|
| Fenster und Starter | 🟡 | Ein Rust-Fenster mit Starter (die moderne Oberfläche oder die klassischen Programme), fünf Designs und 37 Sprachen; es läuft auf einem Rechner und wurde mit Bildern des simulierten Auftrags angesehen. |
| Ansichten | 🟡 | Die Draufsicht (Löcher nach Bohrergröße, Fräsbahnen) und die 3D-Ansicht der Maschine mit Schienen, gelochtem Tisch, fahrendem Portal und Werkzeugablage, angepasst an die Tischgröße; beide lassen sich ziehen und zoomen. Die 3D-Ansicht zeigt noch keine Höhe der Bahnen. |
| Dateien und Nutzenbildung | 🟡 | Bohrdateien, HP-GL-Dateien und ZIP-Archive werden aus einem Browser, per Drag and Drop oder von der Befehlszeile geöffnet; die Platine wird in drei Modi vervielfacht. Mit den kleinen Beispieldateien dieses Repositorys ausprobiert. |
| Einstellungen | 🟠 | Nutzbare Breite und Tiefe des Tisches werden geändert und in den Maschinendateien gespeichert; der Rest der Maschinen- und Werkzeugdateien wird weiter mit den klassischen Programmen bearbeitet. |
| Maschinensteuerung | 🔴 | Der Auftrag läuft über die Engines von EH3D-DRILLPRO und EH3D-ROUTEPRO, also sind im Prinzip alle ihre Verbindungen da; er lief nur im Simulator. Das Verbindungsfenster wählt die Verbindung (den RS-232 des Rechners, einen USB-RS232-Adapter, den EH3D-Knoten oder den EH3D-Server), testet sie und speichert sie in der Maschinendatei, und Pause hält den Auftrag zwischen zwei Bewegungen an. |
| Tests | 🟡 | 198 Tests bestehen auf einem Rechner (`cargo test --features gui`): der Starter, die Einstellungen, das Einlesen der Dateien in die Szene, die Maschinenprofile, der Nutzen und die Befehlszeile. Keiner berührt eine Maschine. |
| Fertige Pakete | 🟠 | Ein Workflow baut das Programm für Windows (64 und 32 Bit), Linux und macOS mit den Beispieldateien; eine Version wurde noch nicht veröffentlicht. |

## 🎯 Überblick

* Der **Starter**: die moderne Oberfläche oder die klassischen Bohr- und Fräsprogramme für eine Maschine mit manuellem oder automatischem Werkzeugwechsel; sie werden als die Programme gestartet, die sie sind, wenn sie neben diesem liegen. Die Wahl lässt sich merken.
* Die **Draufsicht**: der Tisch mit jedem Loch nach Bohrergröße und den Fräsbahnen darüber, mit der Maus gezogen und mit dem Rad gezoomt; eine Schaltfläche (oder ein Doppelklick) stellt die Ansicht zurück.
* Die **3D-Ansicht der Maschine**: Schienen, gelochter Tisch, fahrendes Portal und Werkzeugablage, mit dem Tisch in der in der Konfiguration eingestellten Größe. Sie wird durch Ziehen gedreht, mit dem Rad gezoomt und mit Umschalt oder der rechten Taste verschoben. Die Maschinenprofile (CCD/2, CCD, CCD Premium und CCD XL) stammen aus den veröffentlichten Maßen; die Maschine wird am Tisch der Konfiguration erkannt.
* **Dateien öffnen**: eine Bohrdatei, eine HP-GL-Datei oder ein ZIP-Archiv mit einem Auftrag, aus einem Öffnen-Fenster mit Ordnerbrowser und Laufwerken, durch Ablegen der Dateien auf dem Fenster, mit `Strg+O` oder von der Befehlszeile. Ein Name im Archiv, der den Arbeitsordner verlassen würde, wird nicht aufgelistet, und keine Datei wird über eine Größengrenze hinaus gelesen.
* **Nutzenbildung**: Kopien der Platine in Spalten und Zeilen, platziert nach dem Abstand zwischen den Platinen, nach der Entfernung von einer zur nächsten oder so viele, wie auf den Tisch passen; die Kopien werden gezeichnet und der Auftrag fertigt sie.
* Der **Auftrag**: er läuft auf den Engines der klassischen Programme mit deren Maschinendatei (`demo` Simulator, `rs232`, `usb`, `node` oder `server`), zeigt den Fortschritt und stellt dem Bediener die Fragen der Maschine (Spindel, Werkzeugwechsel).
* Die **Platine und die Werkzeuge**: eine Bibliothek von Basismaterialien vieler Hersteller (Bungard, Fortex, CIF, generisches FR-4, CEM und FR-2, Isola, Panasonic, Shengyi, ITEQ, Ventec, Nelco, Arlon, Rogers, Taconic, Bergquist, DuPont, MG Chemicals, MSC, Yuxinda, ThinFlex), gewählt nach Material, Dicke, Kupfer, Kupferseiten und Größe und gemerkt; die Werkzeuge des Auftrags stehen als Bohren oder Fräsen mit Vorschub und Drehzahl in der Liste, das arbeitende leuchtet, und jeder Wert hat Standardwerte (nach dem Bohrerdurchmesser und dem Platinenmaterial) oder kann angeklickt und eingetippt werden.
* **Platten ohne Kupfer zum Fräsen und Bohren:** neben den kupferkaschierten Platinen enthält die Bibliothek blanke Platten in den üblichen Dicken (Aluminiumlegierung 0,2 bis 5 mm, Kupfer 0,1 bis 5 mm, PTFE 0,2 bis 5 mm, Birkensperrholz, MDF, Balsa und Linde 0,5 bis 5 mm, Acryl, Polycarbonat, ABS, POM, Nylon, PVC, HDPE, PETG, G10 und Phenolharzpapier). Sie tragen kein Kupfer (das Plattenfenster sagt es und blendet die Kupferseiten aus) und keine elektrischen Werte, die niemand für sie veröffentlicht, und jede Materialart hat eigene Startwerte für Drehzahl, Vorschub und Zustellung je Durchgang beim Bohren und Fräsen, stets innerhalb der Grenzen der Spindel.
* Die **Einstellungen**: Design, Sprache, nutzbare Tischgröße, Geschwindigkeit der Simulation (ein Auftrag dauert so lange wie auf der Maschine, von x0,5 bis x50), ob der Auftrag beim Werkzeugwechsel und beim Spindelstart nachfragt und ob dieses Fenster beim nächsten Mal direkt öffnet. Die Tasten leuchten unter dem Zeiger auf und sinken beim Drücken ein.
* **Haltestege für die Außenkontur des Panels:** im Fenster Panelisierung kann die Außenkontur mit Haltestegen gefräst werden, die die Platinen bis zum Herausbrechen halten: wie viele pro Seite, wie breit und wo (gleichmäßig, nahe den Enden oder gemeinsam in der Mitte); die Planansicht zeigt die geschnittenen Pfade. Auch Hersteller, Dicke und Größe der Platine werden aus Auswahllisten gewählt.
* **Tischkalibrierung:** ein Reiter des Einstellungsfensters misst die Höhe des Tisches in einem Punktraster (standardmäßig 7 mal 5), zeichnet die Karte (blau tief, rot hoch) mit Spanne und Neigung in mm pro Meter und speichert sie bei der Maschine. Solange der Tasterknoten nicht verbunden ist, wird der Taster simuliert, und die Korrektur der Aufträge wird noch nicht angewendet.
* **Der Auftrag geht nach einem Absturz weiter:** während ein Auftrag läuft, wird in Echtzeit gespeichert, was er zum Weitermachen braucht (die Datei, das Panel, die gebohrten Löcher); fallen Fenster, Rechner oder Verbindung aus, fragt der nächste Start, ob er dort weitermachen oder abbrechen soll. Das Fenster öffnet sich maximiert, und die Auftragsknöpfe sind hell (grün zum Starten, blau für Pause, rot zum Stoppen).
* **Gesprochene Hinweise:** neben den Tönen können das Auftragsende und die Werkzeuganforderung von einer Stimme gesagt werden, in der Sprache und dem Geschlecht aus derselben Tonliste (`sound_finish = voice:es:f`). Die Stimmen im Ordner `voices/` sind synthetisch (Piper) und frei weitergebbar, mit einem langsamen, förmlichen Satz; es gibt Stimmen für 27 Sprachen (die ersten sieben sowie Bulgarisch, Tschechisch, Dänisch, Niederländisch, Estnisch, Finnisch, Griechisch, Ungarisch, Lettisch, Norwegisch, Polnisch, Rumänisch, Slowakisch, Slowenisch, Schwedisch, Ukrainisch, Russisch, Albanisch, Baskisch und brasilianisches Portugiesisch; das Geschlecht der neuen wurde nicht gehört, prüfe es), die übrigen (Japanisch, Kroatisch, Irisch, Maltesisch, Katalanisch, Galicisch, Serbisch, Isländisch und Litauisch) haben keine frei weitergebbare Stimme und spielen den Ton, es gibt keine weibliche chinesische oder portugiesische Stimme und keine echte Aufnahme einer Person: lege eigene WAV-Dateien in dieselben Ordner, sie ersetzen diese. Siehe `voices/README.txt`.
* **Ein Werkzeugkatalog:** ein Fenster (aus dem Werkzeugfenster) listet mehr als 450 Werkzeuge mit dem 1/8-Zoll-Schaft (3,175 mm) des Spannfutters einer Bungard in zwölf Arten: die Bohrer von der 0,05-mm-Mikroserie bis 7 mm (kleine, mittlere und große mit Spanbrecher), Langlochbohrer, diamantverzahnte, Druckschnitt-, Zugschnitt-, Spanbrecher- und diamantbeschichtete Fräser, Schaftfräser, V-Fräser, Kugelfräser, Anbohrer und Senker. Jedes hat seine Serie, die Länge der Schneide und die Drehzahl, die Vorschübe und die Zustelltiefe aus der Tabelle des Herstellers für FR-4; ein Klick zeichnet das Werkzeug (mit seinen Maßen) und sagt, wofür es ist, und eine Spindel, die langsamer dreht als die Tabelle (standardmäßig 24 000, im Fenster änderbar), bekommt dasselbe Werkzeug mit im gleichen Verhältnis gesenkten Vorschüben. Ein V-Fräser nennt die Breite seiner Nut bei einer Tiefe. Die Beispielplatine hat die Isolation von vier Leiterbahnen, mit einem V-Fräser gefräst (Werkzeug 3 der Frästabelle).
* **Die Aufträge folgen dem Tisch, und der Knoten am Bohrkopf:** ein Kontrollkästchen im Reiter der Kalibrierung lässt Bohren und Fräsen ihr Z mit der Tischkarte korrigieren (nur eine vollständige Karte, nur auf Wunsch); ein Fenster (aus dem Verbindungsfenster) legt fest, wie EH3D-NODE-BUNGARD-AUX erreicht wird (nicht benutzt, direkt oder über den Server), seine Adresse, das Token und ob seine Kamera gezeigt wird, und zeigt das Kamerabild, Drehzahl und Temperatur der Spindel, den Schutzausgang und eine Messung des Tisches unter dem Kopf. Über den Server fragt es den Knoten über den Server ab, mit Adresse, Token und Maschinen-ID des Verbindungsfensters (der Server hält das Token des Knotens); der Taster der Kalibrierung ist noch simuliert.
* **Leichter zu lesen und zu verschieben:** Die Werkzeugliste der Auftragskarte scrollt, statt die Ebenen aus dem Fenster zu drücken, jede geöffnete Auswahlliste hat eine funktionierende Leiste (Ziehen, Klicken, Mausrad), eine Schaltfläche der 2D-Ansicht verschiebt nur das Design über Platine und Tisch, jedes Fräswerkzeug hat in der 2D- und 3D-Ansicht seine eigene Farbe (eine Datei mit drei Fräsern zeigt drei Farben), und die Stege werden auch bei nur einer Kopie in den Außenrand einer Platine geschnitten, und nur dort, nie in die Taschen und Leiterbahnen im Inneren.
* **Dezente Animationen:** Bildschirme und Fenster blenden beim Öffnen ein, und während ein Auftrag läuft, atmet der Ring des Kopfes und sendet eine Welle aus, die Striche der noch zu fräsenden Bahnen wandern an ihnen entlang, ein Licht quert den Fortschrittsbalken und das arbeitende Werkzeug atmet in der Liste. In Ruhe bewegt sich nichts.

## 🧩 Rolle in EH3D

EH3D-BUNGARD-GUI ist die moderne Oberfläche der Bungard-Maschinen in der EH3D-Familie. Sie hängt per Pfad von den Nachbarordnern EH3D-COMMON (die Szene, die Designs und die Texte in 37 Sprachen von `eh3d-view` sowie der ZIP-Leser von `eh3d-archive`), EH3D-DRILLPRO und EH3D-ROUTEPRO (ihre Engines und ihre klassischen Programme) ab: sie müssen neben diesem liegen.

## 🏛️ Architektur

<p align="center">
  <img src="images/architecture.svg" alt="Architektur von EH3D-BUNGARD-GUI: wer es nutzt, was es enthält und wovon es abhängt" width="100%">
</p>

Wie die Teile dieses Repositorys zusammenpassen: wer es nutzt, was es enthält und wovon es abhängt. Das Bild wird aus `docs/architecture.json` gezeichnet und gegen den Code geprüft.

## 🖼️ Screenshots


<p align="center">
  <img src="images/screenshots/viewer.png" alt="viewer" width="100%">
</p>

<p align="center"><em>Der Betrachter: der Auftrag auf dem Tisch in 2D, die Maschine in 3D und die Bedienelemente des Auftrags.</em></p>

<p align="center">
  <img src="images/screenshots/edit-select.png" alt="edit" width="100%">
</p>

<p align="center"><em>Bearbeiten in der 2D-Ansicht: Das Auswahlwerkzeug hat die Löcher der linken Platinenhälfte gewählt, bereit zum Kopieren, Einfügen, Ausschneiden oder Löschen; jedes Werkzeug der Liste hat eine eigene Schaltfläche, um es aus dem Auftrag zu entfernen.</em></p>

<p align="center">
  <img src="images/screenshots/open-preview.png" alt="open file with preview" width="100%">
</p>

<p align="center"><em>Das Öffnen-Fenster mit der Vorschau der gewählten Bohrdatei rechts neben der Liste; ein Schalter darunter blendet sie aus, und die Liste bekommt ihren Platz zurück.</em></p>

<p align="center">
  <img src="images/screenshots/job-running.png" alt="job running" width="100%">
</p>

<p align="center"><em>Ein laufender Auftrag: die fertigen Löcher und Bahnen, die Position des Werkzeugs, Pause und Stopp.</em></p>

<p align="center">
  <img src="images/screenshots/atc-bar.png" alt="Werkzeugleiste der ATC-Modelle" width="100%">
</p>

<p align="center"><em>Die Werkzeugleiste eines ATC-Modells: die Plätze der Werkzeuge neben dem Tisch in der Draufsicht und in 3D, mit dem Werkzeug, das genommen wird, und dem Platz, an dem es abgelegt wird.</em></p>

<p align="center">
  <img src="images/screenshots/atc-bar-window.png" alt="Fenster der Werkzeugleiste" width="100%">
</p>

<p align="center"><em>Das Fenster der Werkzeugleiste: wo das erste und das letzte Werkzeug liegen, wie viele Plätze es gibt und wie groß die Werkzeuge sind; die Plätze dazwischen liegen in gleichen Abständen.</em></p>

<p align="center">
  <img src="images/screenshots/panes.png" alt="verstellbare Bereiche" width="100%">
</p>

<p align="center"><em>Die drei Teile des Hauptfensters mit anderen Breiten: Wer die dünnen Balken dazwischen zieht, gibt der Liste, der Draufsicht oder der Maschinenansicht mehr Platz (ein Doppelklick stellt die Vorgabe wieder her).</em></p>

<p align="center">
  <img src="images/screenshots/board-too-big.png" alt="Platine größer als der Tisch" width="100%">
</p>

<p align="center"><em>Die Warnung über der Draufsicht, wenn die Platine nicht auf den Tisch der Maschine passt, auch nicht um 90 Grad gedreht: die Größen von beiden und um wie viel sie überstehen.</em></p>

<p align="center">
  <img src="images/screenshots/boards-all-makers.png" alt="Platinenliste mit allen Herstellern" width="100%">
</p>

<p align="center"><em>Die Platinenliste mit den Platinen aller Hersteller auf einmal: der erste Eintrag des Herstellermenüs ist Alle Hersteller, und die Liste ist alphabetisch geordnet.</em></p>

<p align="center">
  <img src="images/screenshots/panel.png" alt="panel" width="100%">
</p>

<p align="center"><em>Das Fenster Panelisierung: die Kopien der Platine und die Haltestege der Außenkontur, gewählt mit Auswahllisten und Zählern.</em></p>

<p align="center">
  <img src="images/screenshots/panelized.png" alt="Platine im Nutzen 2 x 2" width="100%">
</p>

<p align="center"><em>Die Platine als Nutzen 2 x 2 auf dem Tisch einer CCD/2: die vier Kopien liegen mit dem Abstand dazwischen auf der virtuellen Platine, in der Draufsicht und in 3D.</em></p>

<p align="center">
  <img src="images/screenshots/tools.png" alt="tools" width="100%">
</p>

<p align="center"><em>Die Werkzeuge: Vorschübe, Drehzahlen und Tiefen, die Löcher jedes Bohrers auf der Platine und die insgesamt gebohrten.</em></p>

<p align="center">
  <img src="images/screenshots/settings.png" alt="allgemeine Einstellungen" width="100%">
</p>

<p align="center"><em>Die allgemeinen Einstellungen: Thema, Sprache, Einheiten, Simulationsgeschwindigkeit und der Schalter für Vollbild (F11 macht dasselbe).</em></p>

<p align="center">
  <img src="images/screenshots/machine-settings.png" alt="settings" width="100%">
</p>

<p align="center"><em>Die Einstellungen: jedes Bungard-Modell, der Tisch und die Einheiten der Dateien.</em></p>

<p align="center">
  <img src="images/screenshots/themes.png" alt="themes" width="100%">
</p>

<p align="center"><em>Elf Themen im Einstellungsfenster, jedes mit seinem Namen in der Sprache des Fensters.</em></p>

<p align="center">
  <img src="images/screenshots/neon-theme.png" alt="Thema Neon Arcade" width="100%">
</p>

<p align="center"><em>Das Thema Neon Arcade.</em></p>

<p align="center">
  <img src="images/screenshots/sounds.png" alt="sounds" width="100%">
</p>

<p align="center"><em>Die Liste der Töne für das Ende des Auftrags: sechzig Töne von hohen bis zu tiefen, der Standardton, keiner oder eine eigene Datei.</em></p>

<p align="center">
  <img src="images/screenshots/calibration.png" alt="calibration" width="100%">
</p>

<p align="center"><em>Die Tischkalibrierung: die Höhenkarte, in einem Punktraster gemessen, mit Spanne und Neigung.</em></p>

<p align="center">
  <img src="images/screenshots/catalogue.png" alt="catalogue of bits" width="100%">
</p>

<p align="center"><em>Der Katalog der Bohrer, nach Art gefiltert, mit den Hinweisen zur Spindel und zum Material der Platine.</em></p>

<p align="center">
  <img src="images/screenshots/fleet.png" alt="machines and stations" width="100%">
</p>

<p align="center"><em>Die Maschinen und Stationen des Servers: Zustand, Auftrag und Fortschritt jeder Maschine und die Maschine, an die die Aufträge gehen.</em></p>

<p align="center">
  <img src="images/screenshots/head-node.png" alt="Knoten am Bohrkopf" width="100%">
</p>

<p align="center"><em>Der Knoten am Bohrkopf: seine Kamera, Drehzahl und Temperatur der Spindel, der Schutzausgang und der LED-Ring um die Kamera (aus und drei Stufen).</em></p>

<p align="center">
  <img src="images/screenshots/quality-check.png" alt="Qualitätsprüfung" width="100%">
</p>

<p align="center"><em>Die Qualitätsprüfung nach dem Auftrag: Die Kamera betrachtet die Platine, jede Bohrung des Auftrags ist markiert (grün richtig, rot falsch, bernstein außerhalb des Bildes), die gefrästen Kanäle werden gemessen und das Urteil ist so groß wie ein Schild.</em></p>

<p align="center">
  <img src="images/screenshots/quality-tolerances.png" alt="Toleranzen" width="100%">
</p>

<p align="center"><em>Die Toleranzen des Analysators der Werkstatt: Größe und Lage der Bohrungen, Rundheit, Grat, Breite und Gleichmäßigkeit der Kanäle und was ein Bild gut genug zum Beurteilen macht.</em></p>

## ✨ Funktionen

* Designs: Forge (Standard), Neon Arcade, Glass Pro, das klassische Windows-Grau und eines, das den Farben von Windows folgt.
* 37 Sprachen im ganzen Fenster: Englisch, Spanisch, Französisch, Italienisch, Deutsch, Chinesisch, Japanisch und europäisches Portugiesisch.
* Die Textfelder haben das Menü der rechten Maustaste (Ausschneiden, Kopieren, Einfügen, Alles auswählen, Rückgängig) und nehmen die üblichen Tastenkürzel an; die Breite von Auftragsliste, Draufsicht und Maschinenansicht wird durch Ziehen der dünnen Leisten dazwischen eingestellt (ein Doppelklick stellt den Standard wieder her); das Fenster kann den Bildschirm ausfüllen (F11 oder Einstellungen); vor einer Platine, die nicht auf den Tisch der aktiven Maschine passt, auch nicht um 90 Grad gedreht, wird gewarnt; und die Liste der Platinen ist alphabetisch sortiert und zeigt auch die Platinen aller Hersteller auf einmal.
* Maschinenprofile mit den veröffentlichten Maßen und einem einstellbaren Tisch: die Maschine in 3D wächst mit der nutzbaren Fläche der Konfiguration.
* Ein Fenster der Maschinen und Stationen: mit der Serververbindung zeigt es alle Maschinen des Servers nach Stationen gruppiert, was jede tut, ihren Auftrag mit Fortschritt, die bereiten Aufträge, und wählt die Maschine, an die die Aufträge des Fensters gehen; `--fleet DATEI.json` zeigt eine gespeicherte Antwort ohne Server.
* Die Wahl eines Maschinenmodells in den Einstellungen passt beide Ansichten an seinen Tisch an: der 2D-Plan, die 3D-Maschine (Rahmen, Schienen, Portal und Werkzeugmagazin dieses Modells) und der Zoom beginnen mit den neuen Maßen von vorn.
* Aufträge in ZIP-Archiven: die erste Bohrdatei und die erste Fräsdatei werden gezeigt, die anderen sind aufgelistet und mit einem Klick ladbar.
* Nutzenbildung in drei Modi (Abstand, Raster, so viele wie passen) mit den gezeichneten Kopien auf dem Tisch.
* Fortschritt des Auftrags live und die Fragen der Maschine an den Bediener.
* Ein Info-Fenster mit Version, Lizenz und der verwendeten Verbindung.
* Einstellungen in Reitern mit allen Bungard-Modellen MTC und ATC (Tisch, Werkzeugwechsler, Grenzen) und den Einheiten für Schritte und Koordinaten (mm oder mil), alles mit echter Wirkung auf den Auftrag.
* Ein Not-Aus-Knopf, der angezeigt, ausgeblendet und verschoben werden kann; Start erlischt, solange der Auftrag läuft, und kommt mit Pause oder Stopp zurück; die Simulation läuft im Hintergrund weiter.
* Eine 2D-Ansicht, die jedem Werkzeug folgt, Bohr- und Fräsdatei gemeinsam dreht und die Platine verschiebt; eine Bibliothek mit mehr als hundert Basismaterialien; Töne am Ende des Auftrags und beim Werkzeugwechsel.
* Dateien anderer Programme: der Umriss einer Gerber-Platine (`.GKO`, `.GM1`, `.GML` oder eine `.GBR`, die sich als Profil ausweist) und eine DXF-Zeichnung öffnen sich als Fräsaufträge; die Kreise und Bögen der Plotterdateien (`CI`, `AA`, `AR`) werden nach dem Sehnenwinkel der Konfiguration in Sehnen zerlegt.
* Ein Auftrag, der nicht auf den Tisch passt, wird vor dem Start abgelehnt; das Öffnen-Fenster listet die zuletzt geöffneten Dateien; das Werkzeugfenster zählt die Löcher jedes Bohrers auf der Platine und in allen Aufträgen, mit einer Taste, die die Zähler zurücksetzt.
* Das Werkzeugfenster öffnet sich über einer Datei, die Sie öffnen (im Fenster Öffnen, aus den letzten Dateien oder durch Ablegen), wie die klassischen Programme die Werkzeuge einer Datei nach dem Laden zeigen; ein Kontrollkästchen in den Einstellungen schaltet es ab.
* Auswahllisten, wo eine Wahl viele Werte hat: Thema, Sprache, Maschinenmodell und der Ton jedes Ereignisses werden aus einer scrollbaren Liste gewählt statt aus Reihen von Knöpfen. Das Ende des Auftrags und die Werkzeuganforderung haben je eine eigene Liste mit 60 Tönen, von hohen bis zu tiefen (Glocken, steigende und fallende Töne, Fanfaren, Piepser, Alarme, Klopfen, Sirenen), den Standardton, keinen oder eine eigene Datei; die Wahl wird abgespielt.
* **Licht und Qualitätsprüfung mit der Kamera des Kopfes:** Das Fenster des Knotens schaltet den LED-Ring ein und aus, und das Fenster Qualitätsprüfung betrachtet die Platine nach dem Auftrag: Es beurteilt die Bohrungen des Auftrags (vorhanden, rund, richtige Größe, am Platz), die gefrästen Kanäle (so breit wie das Werkzeug, gleichmäßig, ohne Engstellen) und das Bild selbst, markiert jede Bohrung im Bild, zeigt ein Urteil so groß wie ein Schild (IN ORDNUNG, NICHT IN ORDNUNG oder nicht beurteilbar) und lässt den Ring dasselbe in Farbe zeigen. Profile (Standard, Feinraster, Prototyp), Toleranzen, der Maßstab und seine Kalibrierung an einer bekannten Bohrung werden in einer Datei gespeichert und dem Server übergeben. Das Ergebnis kann auch gesprochen werden: zwei weitere Töne in den Einstellungen, für eine Prüfung in Ordnung und eine nicht in Ordnung, jeweils aus, ein Piepton, ein Ton des Katalogs, eine eigene Datei oder eine Stimme, die sagt, dass die Sichtprüfung der Platine in Ordnung oder nicht in Ordnung ist, wie die anderen Töne.
* **Eine 3D-Maschine, die sich wie die echte verhält:** In Ruhe wartet der Kopf in Home, und das Verschieben der Platine in der Draufsicht bewegt ihn nie. Beim Start eines Auftrags fährt die Maschine hoch, fährt in Home, fährt zum ersten Punkt der Arbeit und erst dann beginnt die Arbeit, der Kopf folgt ihr, beim Bohren und beim Fräsen; am Ende fährt er wieder nach Home. Die Leisten der Listen haben ihre Pfeile nach oben und unten.
* **Zwei Spindeln und ein Katalog, der sie kennt:** Die Maschine trägt die Originalspindel (63000 U/min) oder die mit Frequenzumrichter (24000 U/min), in den Einstellungen gewählt, jede mit eigenen Grenzen (Drehzahl, XY- und Z-Vorschub, dickstes Werkzeug), die in der Einstellungsdatei liegen. Im Werkzeugfenster hat jedes Werkzeug eine Schaltfläche Katalog: Bohrer oder Fräser wählen, und Drehzahlen, Vorschübe und Zustellungen gehen ins Werkzeug. Kann die Spindel sie nicht liefern, sagt das Fenster es: Es schlägt die brauchbaren Werte vor (gleicher Vorschub pro Umdrehung, langsamerer XY oder Z) oder markiert das Werkzeug als nicht kompatibel und lässt es nicht übernehmen. Die Listen der Platinen zeigen 16 Materialien gleichzeitig.
* **Bohren, Fräsen oder beides, Werkzeug für Werkzeug:** Das Bohren und das Fräsen des Auftrags werden im linken Bereich und im Werkzeugfenster ein- oder ausgeschaltet (beide an: erst Bohren, dann Fräsen, nacheinander, ohne dass die Maschine dazwischen nach Home fährt), und der Punkt jedes Werkzeugs schaltet es ab, damit der Auftrag es auslässt.
* **Mehrere Dateien gleichzeitig und professionelle Beispiele:** Das Fenster Öffnen wählt zwei oder mehr Dateien mit Strg+Klick oder Umschalt+Klick (Bohren und Fräsen zusammen; mehrere gleicher Art bilden einen Auftrag, die Werkzeuge nach Durchmesser nummeriert). Der Ordner `examples` des Pakets enthält vier Entwürfe echter Komplexität, jeweils mit Bohr- und Frässdatei: eine 32-Bit-Mikrocontrollerplatine (Stiftleisten, USB-Aussparung, Via-Ring des LQFP), ein vierkanaliges analoges Frontend (DIP-Verstärker, BNC und Schraubklemmen, Guardringe, ein Isolationsschlitz), ein 2,4-GHz-Verstärker mit Filter (Kanten-SMA, Via-Zäune, Schirmgehäuse-Vernähung, Isolationsfräsen entlang der 50-Ohm-Leitung) und ein 5-V-3-A-Netzteil (TO-220, Elkos, thermische Vias).
* **37 Sprachen:** das ganze Fenster spricht Englisch, Spanisch, Italienisch, Deutsch, Französisch, Portugiesisch, Chinesisch, Japanisch, Bulgarisch, Kroatisch, Tschechisch, Dänisch, Niederländisch, Estnisch, Finnisch, Griechisch, Ungarisch, Irisch, Lettisch, Litauisch, Maltesisch, Polnisch, Rumänisch, Slowakisch, Slowenisch, Schwedisch, Norwegisch, Isländisch, Ukrainisch, Russisch, Serbisch, Albanisch, Katalanisch, Galicisch, Baskisch, brasilianisches Portugiesisch und kanadisches Französisch. Die Wörter der 29 Sprachen nach den ersten acht stammen vom Modell und warten auf die Durchsicht durch Muttersprachler; ein fehlender Text wird auf Englisch gezeigt. Auch die Web-Panels der Knoten der Maschine sprechen alle 37; der Bildschirm der Maschine (HMI) behält vorerst die ersten acht.
* **Windows ohne schwarzes Fenster:** Die Programme mit Fenster (dieses, DRILLPRO, ROUTEPRO und MOTIONCNC) öffnen unter Windows keine Konsole mehr; aus einem Terminal gestartet, schreiben sie Hilfe und Fehler weiterhin dorthin. **Fenster, die sich bewegen:** Jedes Fenster über dem Hauptfenster wird an seiner Oberkante gezogen und hat ein X in der oberen rechten Ecke.
* **Vorschau im Öffnen-Fenster:** Rechts neben der Liste wird vor dem Öffnen der Plan der gewählten Bohr- oder Fräsdatei gezeichnet (Löcher mit Durchmesser, Bahnen, Größe der Platine). Ein Schalter darunter schaltet sie ein oder aus, und die Liste nutzt den Platz, den der Plan freilässt; die Wahl wird gemerkt.
* **Auftrag in der 2D-Ansicht bearbeiten:** Ein Werkzeug der Liste links kann aus dem Auftrag entfernt werden (seine Löcher oder Bahnen verschwinden aus der Datei und aus allen Ansichten); in der 2D-Ansicht wählt das Auswahlwerkzeug Löcher und Bahnen per Klick oder Rahmen (von links nach rechts: was ganz darin liegt; von rechts nach links: was der Rahmen berührt; Strg oder Umschalt fügt hinzu), und sie lassen sich kopieren, einige Millimeter weiter einfügen, ausschneiden und löschen – mit den Schaltflächen oder mit Strg+C, Strg+X, Strg+V, Strg+A und Entf. Die von einer Bearbeitung geschriebenen Dateien sind neu; das Original wird nie verändert.
* **Relief und Symbole überall:** Schaltflächen, Kontrollkästchen, Textfelder, Listenleisten (mit ihren kleinen Pfeiltasten) und die Felder der Listen haben einen Verlauf, einen Lichtrand und einen Schatten und antworten dem Zeiger mit einem Leuchten und beim Drücken mit Einsinken, in jedem Thema; das X in der Ecke eines Fensters ist rot, dreht sich und leuchtet unter dem Zeiger; die kleinen Schaltflächen der 2D-Ansicht und des Öffnen-Fensters sind Symbole mit einer Notiz, die ihren Namen sagt.
* **Mehr Bearbeitung in der 2D-Ansicht, Start bei einem Werkzeug und serielle Schnittstellen:** Das Gewählte lässt sich ziehen (oder mit den Pfeiltasten verschieben: 1 mm, 0,1 mm mit Umschalt, 10 mm mit Strg), ausschneiden, und jede Bearbeitung ist rückgängig und wiederholbar zu machen; bei einem Nutzen ist ein Loch jeder Kopie dasselbe Loch der Platine. Jedes Werkzeug der Liste hat eine Schaltfläche, um den Auftrag bei ihm zu beginnen (die vorherigen entfallen), und zeigt, wie viele Löcher oder Bahnen von der Gesamtzahl erledigt sind. Nach dem Nutzen gehen beide Ansichten zurück in die Mitte der Arbeit. Das Verbindungsfenster listet die seriellen Schnittstellen des Systems für RS-232 und USB-RS232 auf und wählt die erste selbst.
* **Launcher, Felder und Ebenen mit Relief:** Die Karten des Launchers und die Felder haben Verlauf, Schatten und Leuchten unter dem Zeiger, die Werkzeugliste des Auftrags und die Kontrollkästchen der Ebenen antworten ebenfalls dem Zeiger, und das "<" zurück zum Launcher ist jetzt eine Schaltfläche mit Hinweis.
* **Der Werkzeugwechsler der ATC-Modelle:** Vor dem Start eines Auftrags zeigt ein Fenster die Plätze der Leiste (Platz 0 ist das Testwerkzeug, in der Spindel, und bleibt frei) und was in welchen kommt, in der Reihenfolge, in der der Auftrag die Werkzeuge nutzt; ein Auftrag mit mehr Werkzeugen, als die Leiste fasst, läuft in Bestückungen: die, die passen, ein Halt zum Neubestücken und der Rest. Bohr- und Fräswerkzeuge teilen sich eine Bestückung, solange sie passen.
* **Die Werkzeugleiste der ATC-Modelle:** Die Leiste am Ende der Maschine, in der die Werkzeuge warten, wird in der Draufsicht und in der 3D-Ansicht gezeichnet: ihre Plätze in einer Reihe mit Nummer, die Werkzeuge darin (Farbe ihrer Größe und Nummer), das Werkzeug im Kopf, das als Nächstes zu holende und der freie Platz, an dem das benutzte abgelegt wird. Eine Schaltfläche im Einstellungsfenster öffnet ein Fenster, in dem die genaue Position (X, Y, Z) des ersten und des letzten Werkzeugs und die Zahl der Plätze eingestellt werden; die Plätze dazwischen liegen in gleichen Abständen und werden mit den übrigen Plätzen des Wechslers in der Maschinendatei gespeichert, zusammen mit den Werkzeuglängen (38 mm, 21 mm vom Ring bis zur Spitze), der Höhe, in der das Spannfutter über den Werkzeugen fährt, und der Seite, von der es einfährt. **Nichts davon wurde an einer echten Maschine erprobt.**
* **Bungard-Katalog in der Plattenbibliothek:** Die vier Bungard-Materialien tragen die Daten des Katalogs 2024/25 (Dicken 0,5 bis 2,5 mm, Kupfer 18, 35 und 70 um und 105 um bei 1,5 mm, die Standardzuschnitte und die Tafeln 510 x 570 und 510 x 1150 mm; Tafeln aus FR2, FR3 und CEM-1). Auch die Pfeile des Spindelkatalogs, der Qualitätsprüfung und der Materialien sowie die Zahlenfelder haben Symbole und Relief.

## 🔌 Schnittstellen

* **Fenster:** der Starter und der Betrachter, mit Maus und Tastatur (`Enter` bestätigt die Wahl des Starters, `Esc` geht zurück, `Strg+O` öffnet Dateien).
* **Befehlszeile:** `bungard-gui [DATEI...] [--drill DATEI.NCD] [--route DATEI.PLT] [--zip AUFTRAG.ZIP] [--lang CODE] [--theme NAME] [--data ORDNER] [--connection DATEI] [--screen launcher|viewer]`; Dateien ohne Option werden so geöffnet, wie ihre Endung es sagt. `--modal`, `--panel`, `--yaw`, `--shot` und `--start` legen ein Fenster, den Nutzen, die Drehung der 3D-Ansicht, ein Bild des Fensters und den Start des Auftrags fest, für die Bilder.
* **Maschine:** über die Engines von EH3D-DRILLPRO und EH3D-ROUTEPRO und die gemeinsame Maschinendatei: die RS-232-Schnittstelle des Rechners, ein USB-RS232-Adapter, der EH3D-NODE-BUNGARD oder EH3D-SERVER. Ohne Maschinendatei läuft der Auftrag im Simulator.

## ⚙️ Konfiguration

Die Verbindung ist die **Maschinendatei** von EH3D-DRILLPRO und EH3D-ROUTEPRO (`machine.cfg`, mit `connection = demo|rs232|usb|node|server` und jedem an einer echten Maschine abgestimmten Wert): `--connection DATEI` zeigt auf sie, ohne sie läuft der Auftrag im Simulator. `--data ORDNER` ist der Ordner mit den Maschinen- und Werkzeugdateien der klassischen Programme (`DRILLPRO.DCF`, `DRILLPRO.DTD`, `ROUTEPRO.RCF`, `ROUTEPRO.RTD`; der Ordner `testdata/` enthält Beispiele). Das Einstellungsfenster ändert nutzbare Breite und Tiefe des Tisches und speichert sie in diesen Dateien. Was das Fenster sich merkt (die Wahl des Starters, die Sprache und das Design), liegt in einer kleinen Datei im Einstellungsordner des Benutzers, bei den anderen EH3D-Programmen. `--theme forge|neon|glass|windows-classic|system` und `--lang en|es|it|de|fr|pt|zh|ja` wählen Aussehen und Sprache.

## 🚀 Schnellstart

```bash
cargo build --release --features gui
cargo test --features gui
cargo clippy --all-targets --features gui -- -D warnings
cargo fmt --all --check
python -I tools/ci_validate.py
```

Starte es mit `run.bat` (Windows) oder `./run.sh` (Linux und macOS); beide bauen das Fenster bei Bedarf zuerst und öffnen ohne Argumente den Starter mit der Testplatine aus `testdata/`. Sie brauchen Rust und die Nachbarordner `EH3D-COMMON`, `EH3D-DRILLPRO` und `EH3D-ROUTEPRO`. `cargo run --release --features gui -- --zip AUFTRAG.ZIP` öffnet einen in ein ZIP-Archiv gepackten Auftrag. Unter Linux braucht das Fenster X11 oder Wayland.

## 🧪 Tests

Die Suite hat 198 Tests (`cargo test --features gui`). Sie prüfen den Starter und was er sich merkt, die Einstellungen, wie eine Bohr- oder HP-GL-Datei zur Szene wird, die Maschinenprofile, den Nutzen und die Befehlszeile. Die Engines, die es nutzt (Protokoll, Bewegung, Sicherheitsschranke), werden in EH3D-DRILLPRO und EH3D-ROUTEPRO getestet. Die CI führt außerdem `cargo fmt --check` und `cargo clippy` mit Warnungen als Fehler aus, und `python -I tools/ci_validate.py` prüft Manifest, Versionen, Changelog und die acht READMEs.

## 🏗️ Build und Veröffentlichung

Die Version steht in `Cargo.toml` und in `eh3d.project.json`, und jede Version hat ihren Eintrag in [CHANGELOG.md](CHANGELOG.md). `build.bat` / `build.sh` bauen das Fenster (`cargo build --release --features gui`) und `test.bat` / `test.sh` führen die Tests aus; keines ändert die Version.

**Fertige Pakete:** der Workflow `Release binaries` dieses Repositorys baut das Programm für Windows (64 und 32 Bit), Linux und macOS (Apple Silicon und Intel), jedes Paket mit Lizenz, diesem README und den Testdateien als Beispiele sowie seinem SHA-256. Eine Version wurde noch nicht veröffentlicht.

## 📂 Struktur des Repositorys

<p align="center">
  <img src="images/repository-structure.svg" alt="Struktur des Repositorys EH3D-BUNGARD-GUI" width="100%">
</p>

<details>
<summary>Textversion des Baums</summary>

```text
EH3D-BUNGARD-GUI/
├── src/
│   ├── gui.rs                                   das Fenster: Starter, Draufsicht, 3D-Ansicht, Auftragsfortschritt
│   ├── gui/extra.rs                             Dateien öffnen, Nutzenbildung, Tischeinstellungen, Info, Ansichten bewegen
│   ├── gui/fleet.rs                             Fenster der Maschinen und Stationen des Servers
│   ├── inspection.rs gui/inspect.rs             die Qualitätsprüfung der Arbeit mit der Kamera
│   ├── gui/machine3d.rs                         die Maschine der 3D-Ansicht: Start, Home, erster Punkt, Arbeit
│   ├── gui/opener.rs  gui/board.rs              Fenster Öffnen, Platine, Werkzeuge, Einstellungen
│   ├── jobs.rs                                  eine Bohr- oder HP-GL-Datei als Szene für die Ansichten
│   ├── boards.rs                                die Bibliothek der Basismaterialien vieler Hersteller
│   ├── machines.rs                              die Bungard-Maschinenprofile und der Arbeitsbereich aus der Tischgröße
│   ├── launcher.rs                              die Starter-Auswahl, die klassischen Programme und was gemerkt wird
│   ├── link.rs                                  die Verbindung der Maschinendatei: welche Schlüssel geschrieben werden
│   ├── cli.rs                                   die Befehlszeile
│   ├── lib.rs bin/                              die Wurzel der Bibliothek und die ausführbare Datei bungard-gui
│   └── spindle.rs                               die zwei Spindeln, ihre Grenzen und ob ein Werkzeug passt
├── tests/                                       die Tests, die die Dateien des Repositorys prüfen
├── testdata/                                    eine kleine Platine: Bohr- und Umrissdateien sowie Beispiel-Maschinen- und -Werkzeugdateien
├── docs/                                        die Quellen der beiden Bilder
├── tools/                                       CI-Prüfung, README-Prüfung und die Werkzeuge für die Bilder
├── images/                                      Banner, die beiden Bilder und die Screenshots
├── voices/                                      gesprochene Hinweise
├── .github/                                     CI- und Release-Workflows, Vorlagen für Issues und Pull Requests
├── build.sh build.bat                           bauen das Fenster
├── run.sh run.bat                               starten das Fenster, ohne Argumente mit den Testdateien
├── eh3d.project.json bump_manifest_version.py   Manifest und Versionswerkzeug
├── Cargo.toml Cargo.lock                        das Rust-Paket und seine festgelegten Abhängigkeiten
├── CHANGELOG.md                                 was sich in jeder Version geändert hat
├── SECURITY.md SUPPORT.md                       wie man eine Schwachstelle meldet und wo man fragt
└── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   wie man beiträgt, der Verhaltenskodex und die Lizenz
```

</details>

## 🗺️ Roadmap und bekannte Grenzen

**Bekannte Grenzen heute**

* Von diesem Fenster aus wurde noch kein Auftrag auf einer echten Maschine gefahren: es lief nur im Simulator der Engines von EH3D-DRILLPRO und EH3D-ROUTEPRO.
* Das Verbindungsfenster schreibt nur die Schlüssel der Verbindung in die Maschinendatei; das Einstellungsfenster schreibt die Schlüssel, die es anbietet (Tisch, Modell, Einheiten, Töne und Simulationsgeschwindigkeit), das Werkzeugfenster die Vorschübe, Drehzahlen und Tiefen; der Rest der Maschinen- und Werkzeugdateien wird hier nicht bearbeitet.
* Die Pause hält den Auftrag zwischen zwei Bewegungen an, nicht mitten in einer, und die 3D-Ansicht zeigt die Höhe der Bahnen nicht.
* Nur die Maschinen CCD und CCD/2 werden gesteuert; die Profile der CCD Premium und der CCD XL werden gezeichnet, ihre eigene Steuereinheit wird aber nicht unterstützt.
* Das Fenster braucht die Nachbarordner EH3D-COMMON, EH3D-DRILLPRO und EH3D-ROUTEPRO neben sich, weil es per Pfad von ihnen abhängt.

**Als Nächstes**

* Eine Pause mitten in einer Bewegung, nicht nur zwischen zwei Bewegungen.
* Die Höhe (Z) der Bahnen in der 3D-Ansicht.
* Die ganze Konfiguration laden und speichern, nicht nur die Tischgröße.
* Verpacken mit den klassischen Programmen, damit ein Download alles enthält.
* Ein Test an einer echten Maschine.

## 🌍 Sprachen

README und Texte gibt es auf Englisch, Spanisch, Französisch, Italienisch, Deutsch, Chinesisch, Japanisch und europäischem Portugiesisch.

## 🔗 Verwandte Projekte

* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO): das Bohrprogramm; dieses Fenster nutzt seine Engine und sein klassisches Programm
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO): das Fräsprogramm; dieses Fenster nutzt seine Engine und sein klassisches Programm
* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON): die Ansicht (Szene, Designs, Texte in 37 Sprachen) und der ZIP-Leser, die dieses Fenster nutzt
* [EH3D-NODE-BUNGARD](https://github.com/JuanenRac/EH3D-NODE-BUNGARD): ein ESP32-S3-Knoten im Gehäuse der Maschine, der die Schritte über seine eigene serielle Schnittstelle ausgibt
* [EH3D-SERVER](https://github.com/JuanenRac/EH3D-SERVER): das Zentrum der EH3D-Familie: er kann den Auftrag über seinen Knoten auf der Maschine ausführen

## 📚 Dokumentation

* [Verbindung und Abstimmung](https://github.com/JuanenRac/EH3D-DRILLPRO/blob/main/docs/CONNECTION.md): die Maschinendatei, die Verbindungen und jeder Wert (EH3D-DRILLPRO)
* [Protokollnotizen](https://github.com/JuanenRac/EH3D-DRILLPRO/blob/main/docs/PROTOCOL.md): was darüber bekannt ist, wie die Steuereinheit angesteuert wird (EH3D-DRILLPRO)
* [Changelog](CHANGELOG.md)

## 🤝 Mitwirken und Community

* Führe vor einem Pull Request `cargo fmt --all --check`, `cargo clippy --all-targets --features gui -- -D warnings`, `cargo test --features gui` und `python -I tools/ci_validate.py` aus: alles muss bestehen. Siehe [CONTRIBUTING.md](CONTRIBUTING.md) und den [Verhaltenskodex](CODE_OF_CONDUCT.md).
* Das README gibt es in acht Sprachen: eine Textänderung wird in allen acht Dateien gemacht, und die beiden Bilder werden aus `docs/architecture.json` und `docs/structure.json` mit `tools/make_architecture.py` und `tools/make_structure.py` gezeichnet.
* Beschreibe nicht als funktionierend an einer Maschine, was nur im Simulator ausprobiert wurde.
* Melde eine Schwachstelle privat: [SECURITY.md](SECURITY.md). Fragen, Ideen und Berichte: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTOR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LIZENZ

Software: GPL-3.0-or-later. Siehe [LICENSE](LICENSE). Die Hardware-Entwürfe der Suite verwenden die CERN OHL-S.
