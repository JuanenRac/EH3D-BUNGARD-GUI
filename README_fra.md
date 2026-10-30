<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-BUNGARD-GUI banner" width="100%">
</p>

# 🖥️ EH3D-BUNGARD-GUI

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | 🇫🇷 <b>Français</b> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### La fenêtre moderne pour les machines Bungard CCD : un lanceur, une vue 2D et une vue 3D du travail, perçage et fraisage ensemble, en huit langues

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Tests-73-0f6cbd.svg" alt="Tests">
  <img src="https://img.shields.io/badge/Version-0.0.5-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Vérification d'honnêteté - ce qui tourne aujourd'hui :** un programme Rust avec une fenêtre et 73 tests qui s'exécutent sur un ordinateur. Le lanceur propose l'interface moderne ou les programmes classiques de perçage et de fraisage pour une machine avec changement d'outil manuel (MTC) ou automatique (ATC). Le visualiseur ouvre des fichiers de perçage, des fichiers de tracé HP-GL et des archives ZIP de travaux (depuis un navigateur de dossiers, par glisser-déposer ou depuis la ligne de commande), dessine la vue en plan avec chaque trou par diamètre de foret et les trajets de fraisage par-dessus, montre la machine en 3D avec la table de la taille choisie dans la configuration, panélise la carte et suit le travail pendant qu'il tourne. Le travail s'exécute dans le **simulateur** des moteurs de EH3D-DRILLPRO et EH3D-ROUTEPRO ; le même fichier de machine qu'ils lisent décide comment on atteint une vraie machine (le RS-232 de l'ordinateur, un adaptateur USB-RS232, ou le EH3D-NODE-BUNGARD directement ou via EH3D-SERVER), et **aucune de ces voies n'a été essayée depuis cette fenêtre**. Les fenêtres de connexion, de réglages et d'outils changent les valeurs qu'elles offrent, les fichiers entiers de machine et d'outils ne sont pas encore modifiables, et la vue 3D ne montre pas la hauteur des trajets.

---

> **État de la sûreté et de la sécurité :** ce programme n'est pas un système de sécurité. Il ne gère ni les fins de course ni un arrêt d'urgence, et il n'a jamais piloté une vraie machine. Avec une connexion réelle dans le fichier de la machine, le travail déplacerait la machine via les moteurs de EH3D-DRILLPRO et EH3D-ROUTEPRO : lisez d'abord leurs notes, retirez l'outil, gardez une main sur l'interrupteur de la machine et soyez prêt à arrêter le travail. Le projet est indépendant : il n'est ni fabriqué, ni approuvé, ni soutenu par Bungard Elektronik, et les noms de produits n'apparaissent que pour dire avec quels formats de fichiers et quelles machines il est prévu de fonctionner.

## 📋 En un coup d'œil

| | |
|---|---|
| Version | `0.0.5` |
| Maturité | Échafaudage |
| Tests | 73 |
| Pile technique | Rust |
| Sur du matériel réel | Non. Il n'a tourné que sur le simulateur et sur un ordinateur ; aucune machine n'a été pilotée depuis cette fenêtre |

**Aller à :** [Présentation](#-présentation) · [Architecture](#-architecture) · [Démarrage rapide](#-démarrage-rapide) · [Tests](#-tests) · [Structure](#-structure-du-dépôt) · [Liés](#-projets-liés)

## 🚦 État du projet

Chaque ligne est une partie de ce dépôt et le feu dit où elle en est aujourd'hui. Elle est écrite d'après ce que le dépôt peut montrer (tests, compilations, exécutions), pas d'après ce qui est prévu.

🟢 Fait et vérifié · 🟡 Fonctionne, avec des limites connues · 🟠 À moitié fait ou pas encore vérifié · 🔴 Pas fait ou ne fonctionne pas

| Domaine | État | Ce que cela veut dire ici |
|---|---|---|
| Fenêtre et lanceur | 🟡 | Une fenêtre Rust avec un lanceur (l'interface moderne ou les programmes classiques), cinq thèmes et huit langues ; elle tourne sur un ordinateur et a été regardée avec des images du travail simulé. |
| Vues | 🟡 | La vue en plan (trous par diamètre, trajets de fraisage) et la vue 3D de la machine avec rails, table perforée, portique mobile et rack d'outils, ajustée à la taille de la table ; toutes deux se déplacent et se zooment. La vue 3D ne montre pas encore la hauteur des trajets. |
| Fichiers et panélisation | 🟡 | Les fichiers de perçage, HP-GL et les archives ZIP s'ouvrent depuis un navigateur, par glisser-déposer ou depuis la ligne de commande ; la carte est répétée en trois modes. Essayé avec les petits fichiers d'exemple de ce dépôt. |
| Réglages | 🟠 | La largeur et la profondeur utiles de la table se changent et s'enregistrent dans les fichiers de la machine ; le reste des fichiers de machine et d'outils s'édite encore avec les programmes classiques. |
| Pilotage de la machine | 🔴 | Le travail passe par les moteurs de EH3D-DRILLPRO et EH3D-ROUTEPRO, donc en principe toutes leurs connexions sont là ; il n'a tourné que sur le simulateur. La fenêtre de Connexion choisit la liaison (le RS-232 de l'ordinateur, un adaptateur USB-RS232, le nœud EH3D ou le serveur EH3D), la teste et l'enregistre dans le fichier de la machine, et Pause retient le travail entre deux mouvements. |
| Tests | 🟡 | 73 tests passent sur un ordinateur (`cargo test --features gui`) : le lanceur, les réglages, la lecture des fichiers vers la scène, les profils de machine, le panneau et la ligne de commande. Aucun ne touche une machine. |
| Paquets prêts à l'emploi | 🟠 | Un workflow compile le programme pour Windows (64 et 32 bits), Linux et macOS avec les fichiers d'exemple ; aucune version n'a encore été publiée. |

## 🎯 Présentation

* Le **lanceur** : l'interface moderne, ou les programmes classiques de perçage et de fraisage pour une machine avec changement d'outil manuel ou automatique ; ils sont lancés comme les programmes qu'ils sont quand ils se trouvent à côté de celui-ci. Le choix peut être mémorisé.
* La **vue en plan** : la table avec chaque trou par diamètre de foret et les trajets de fraisage par-dessus, déplacée à la souris et zoomée à la molette ; un bouton (ou un double clic) remet la vue en place.
* La **vue 3D de la machine** : rails, table perforée, portique mobile et rack d'outils, avec la table de la taille réglée dans la configuration. On la fait tourner en glissant, on la zoome à la molette et on la déplace avec Maj ou le bouton droit. Les profils de machine (CCD/2, CCD, CCD Premium et CCD XL) viennent des dimensions publiées ; la machine est reconnue par la table de la configuration.
* **Ouvrir des fichiers** : un fichier de perçage, un fichier HP-GL ou une archive ZIP contenant un travail, depuis une fenêtre Ouvrir avec navigateur de dossiers et lecteurs, en déposant les fichiers sur la fenêtre, avec `Ctrl+O`, ou donnés en ligne de commande. Un nom dans une archive qui sortirait du dossier de travail n'est pas listé, et aucun fichier n'est lu au-delà d'une limite de taille.
* **Panélisation** : des copies de la carte en colonnes et en lignes, placées selon l'écart entre cartes, selon la distance de l'une à la suivante, ou autant qu'il en tient sur la table ; les copies sont dessinées et le travail les réalise.
* Le **travail** : il s'exécute avec les moteurs des programmes classiques et leur fichier de machine (`demo` simulateur, `rs232`, `usb`, `node` ou `server`), montre l'avancement et pose à l'opérateur les questions de la machine (la broche, le changement d'outil).
* La **carte et les outils** : une bibliothèque de matériaux de base de nombreux fabricants (Bungard, Fortex, CIF, FR-4 générique, CEM et FR-2, Isola, Panasonic, Shengyi, ITEQ, Ventec, Nelco, Arlon, Rogers, Taconic, Bergquist, DuPont), choisie par matériau, épaisseur, cuivre, faces cuivrées et taille, et mémorisée ; les outils du travail sont listés en perçage ou fraisage avec leur avance et leur vitesse, celui qui travaille est éclairé, et chaque valeur a des valeurs par défaut (selon le diamètre du foret et le matériau de la carte) ou peut être cliquée et saisie.
* Les **réglages** : le thème, la langue, la taille utile de la table, la vitesse de la simulation (un travail dure le temps de la machine, de x0,5 à x50), si le travail demande aux changements d'outil et au démarrage de la broche, et si cette fenêtre s'ouvre directement la prochaine fois. Les boutons s'éclairent sous le pointeur et s'enfoncent quand on les presse.
* **Ponts pour l'extérieur du panneau :** dans la fenêtre Panélisation l'extérieur peut être détouré avec des ponts qui retiennent les cartes jusqu'à leur séparation : combien par côté, quelle largeur et où (répartis, près des extrémités ou ensemble au milieu) ; la vue du plan montre les trajets coupés. Le fabricant, l'épaisseur et la taille de la carte se choisissent aussi dans des listes déroulantes.
* **Étalonnage de la table :** un onglet de la fenêtre Réglages mesure la hauteur de la table sur une grille de points (7 par 5 par défaut), dessine la carte (bleu en bas, rouge en haut) avec l'écart et l'inclinaison en mm par mètre, et la garde avec la machine. Tant que le nœud palpeur n'est pas connecté, le palpeur est simulé et la correction des travaux n'est pas encore appliquée.
* **Le travail reprend après une panne :** pendant qu'un travail tourne, ce qu'il faut pour continuer (le fichier, le panneau, les trous percés) est enregistré en temps réel ; si la fenêtre, l'ordinateur ou la liaison tombe, le démarrage suivant demande s'il faut reprendre où il en était ou l'annuler. La fenêtre s'ouvre agrandie et les boutons du travail sont vifs (vert pour démarrer, bleu pour la pause, rouge pour arrêter).
* **Avis parlés :** en plus des tonalités, la fin du travail et la demande d'outil peuvent être dites par une voix, dans la langue et le genre choisis dans la même liste de sons (`sound_finish = voice:es:f`). Les voix du dossier `voices/` sont synthétiques (Piper) et libres de diffusion, avec une phrase lente et formelle ; il n'y a pas de voix japonaise, ni de voix féminine chinoise ou portugaise, ni d'enregistrement réel d'une personne : mettez vos propres fichiers WAV dans les mêmes dossiers et ils les remplacent. Voir `voices/README.txt`.
* **Un catalogue d'outils :** une fenêtre (depuis celle des Outils) liste 115 forets, fraises droites, fraises à coupe descendante et montante, fraises à denture diamant, fraises en V, à bout sphérique, à pointer et fraises à chanfreiner, toutes avec la queue de 1/8 de pouce (3,175 mm) du mandrin d'une machine Bungard, par type, avec les tours, les avances et la profondeur d'une passe pour commencer ; une fraise en V dit quelle est la largeur de sa rainure à une profondeur. La carte d'exemple a l'isolation de quatre pistes, taillée avec une fraise en V (outil 3 de la table de fraisage).
* **Les travaux suivent la table, et le nœud de la tête de perçage :** une case de l'onglet d'étalonnage fait corriger leur Z au perçage et au fraisage avec la carte de la table (seulement une carte complète, seulement sur demande) ; une fenêtre (depuis celle de Connexion) règle comment on atteint EH3D-NODE-BUNGARD-AUX (non utilisé, direct ou par le serveur), son adresse, son jeton et si sa caméra est affichée, et montre l'image de la caméra, la vitesse et la température de la broche, la sortie de protection et une mesure de la table sous la tête. Par le serveur elle dit que le relais n'existe pas encore, et le palpeur de l'étalonnage est encore simulé.

## 🧩 Rôle dans EH3D

EH3D-BUNGARD-GUI est l'interface moderne des machines Bungard dans la famille EH3D. Elle dépend par chemin des dossiers voisins EH3D-COMMON (la scène, les thèmes et les textes en huit langues de `eh3d-view`, et le lecteur ZIP de `eh3d-archive`), EH3D-DRILLPRO et EH3D-ROUTEPRO (leurs moteurs et leurs programmes classiques) : ils doivent se trouver à côté de celui-ci.

## 🏛️ Architecture

<p align="center">
  <img src="images/architecture.svg" alt="Architecture de EH3D-BUNGARD-GUI : qui l'utilise, ce qu'il contient et de quoi il dépend" width="100%">
</p>

Comment les parties de ce dépôt s'articulent : qui l'utilise, ce qu'il contient et de quoi il dépend. L'image est dessinée à partir de `docs/architecture.json` et vérifiée par rapport au code.

## 🖼️ Captures d'écran

<p align="center">
  <img src="images/screenshots/viewer.png" alt="viewer" width="100%">
</p>

<p align="center"><em>Le visualiseur : le travail sur la table en 2D, la machine en 3D et les commandes du travail.</em></p>

<p align="center">
  <img src="images/screenshots/job-running.png" alt="job running" width="100%">
</p>

<p align="center"><em>Un travail en cours : les trous et trajets faits, la position de l'outil, Pause et Stop.</em></p>

<p align="center">
  <img src="images/screenshots/neon-theme.png" alt="Neon Arcade theme" width="100%">
</p>

<p align="center"><em>Le thème Neon Arcade avec la carte en panneau.</em></p>

<p align="center">
  <img src="images/screenshots/tools.png" alt="tools" width="100%">
</p>

<p align="center"><em>Les outils : avances, vitesses et profondeurs, les trous de chaque foret sur la carte et les trous percés au total.</em></p>

<p align="center">
  <img src="images/screenshots/machine-settings.png" alt="settings" width="100%">
</p>

<p align="center"><em>Les réglages : chaque modèle Bungard, la table et les unités des fichiers.</em></p>

<p align="center">
  <img src="images/screenshots/board.png" alt="board" width="100%">
</p>

<p align="center"><em>La carte : un matériau de la bibliothèque, son épaisseur, son cuivre et sa taille.</em></p>

<p align="center">
  <img src="images/screenshots/themes.png" alt="themes" width="100%">
</p>

<p align="center"><em>Onze thèmes dans la fenêtre des réglages, chacun nommé dans la langue de la fenêtre.</em></p>

<p align="center">
  <img src="images/screenshots/sounds.png" alt="sounds" width="100%">
</p>

<p align="center"><em>La liste des sons de la fin du travail : soixante sons des aigus aux graves, la tonalité standard, aucun ou un fichier à soi.</em></p>

<p align="center">
  <img src="images/screenshots/panel.png" alt="panel" width="100%">
</p>

<p align="center"><em>La fenêtre Panélisation : les copies de la carte et les ponts de l'extérieur, choisis avec des listes déroulantes et des compteurs.</em></p>

<p align="center">
  <img src="images/screenshots/calibration.png" alt="calibration" width="100%">
</p>

<p align="center"><em>L'étalonnage de la table : la carte des hauteurs mesurées sur une grille de points, avec l'écart et l'inclinaison.</em></p>

<p align="center">
  <img src="images/screenshots/paper-theme.png" alt="light theme" width="100%">
</p>

<p align="center"><em>Le thème Papier, un thème clair.</em></p>

<p align="center">
  <img src="images/screenshots/fleet.png" alt="machines and stations" width="100%">
</p>

<p align="center"><em>Les machines et stations du serveur : état, travail et avancement de chaque machine, et la machine à laquelle vont les travaux.</em></p>

## ✨ Fonctionnalités

* Thèmes : Forge (par défaut), Neon Arcade, Glass Pro, le gris classique de Windows et un qui suit les couleurs de Windows.
* Huit langues dans toute la fenêtre : anglais, espagnol, français, italien, allemand, chinois, japonais et portugais européen.
* Profils de machine avec les dimensions publiées et une table réglable : la machine en 3D grandit avec la zone utile de la configuration.
* Une fenêtre des machines et stations : avec la connexion au serveur elle montre toutes les machines du serveur groupées par station, ce que fait chacune, son travail avec l'avancement, les travaux prêts, et choisit la machine à laquelle vont les travaux de la fenêtre ; `--fleet FICHIER.json` montre une réponse enregistrée sans serveur.
* Choisir un modèle de machine dans les réglages ajuste les deux vues à sa table : le plan 2D, la machine 3D (châssis, rails, portique et magasin d'outils de ce modèle) et le zoom repartent avec les nouvelles mesures.
* Travaux dans des archives ZIP : le premier fichier de perçage et le premier de fraisage sont montrés, les autres sont listés pour être chargés d'un clic.
* Panélisation en trois modes (écart, pas, autant qu'il en tient) avec les copies dessinées sur la table.
* Avancement du travail en direct et questions de la machine à l'opérateur.
* Une fenêtre À propos avec la version, la licence et la connexion utilisée.
* Réglages en onglets avec tous les modèles Bungard MTC et ATC (table, changeur d'outils, limites) et les unités des pas et des coordonnées (mm ou mil), le tout avec un effet réel sur le travail.
* Un bouton d'arrêt d'urgence qui peut être affiché, masqué et déplacé ; Start s'éteint pendant le travail et revient avec Pause ou Stop ; la simulation continue en arrière-plan.
* Une vue 2D qui suit chaque outil, fait tourner ensemble les fichiers de perçage et de fraisage et déplace la carte ; une bibliothèque de plus de cent matériaux de base ; des sons à la fin du travail et au changement d'outil.
* Fichiers d'autres programmes : le contour d'une carte Gerber (`.GKO`, `.GM1`, `.GML` ou un `.GBR` qui se déclare profil) et un dessin DXF s'ouvrent comme travaux de fraisage ; les cercles et arcs des fichiers de tracé (`CI`, `AA`, `AR`) sont découpés en cordes selon l'angle de corde de la configuration.
* Un travail qui ne tient pas sur la table est refusé avant de commencer ; la fenêtre Ouvrir liste les derniers fichiers ouverts ; la fenêtre des outils compte les trous de chaque foret sur la carte et dans tous les travaux, avec un bouton qui remet les compteurs à zéro.
* La fenêtre des outils s'ouvre sur un fichier que vous ouvrez (depuis la fenêtre Ouvrir, les récents ou par glisser-déposer), comme les programmes classiques montrent les outils d'un fichier après l'avoir chargé ; une case des réglages la désactive.
* Listes déroulantes là où un choix a beaucoup de valeurs : le thème, la langue, le modèle de la machine et le son de chaque événement se choisissent dans une liste qui défile, au lieu de rangées de boutons. La fin du travail et la demande d'outil ont chacune leur liste de 60 sons, des aigus aux graves (carillons, notes montantes et descendantes, fanfares, bips, alarmes, coups, sirènes), la tonalité standard, aucun ou un fichier à soi ; le choix est joué.

## 🔌 Interfaces

* **Fenêtre :** le lanceur et le visualiseur, à la souris et au clavier (`Entrée` valide le choix du lanceur, `Échap` revient en arrière, `Ctrl+O` ouvre des fichiers).
* **Ligne de commande :** `bungard-gui [FICHIER...] [--drill FICHIER.NCD] [--route FICHIER.PLT] [--zip TRAVAIL.ZIP] [--lang CODE] [--theme NOM] [--data DOSSIER] [--connection FICHIER] [--screen launcher|viewer]` ; les fichiers sans option sont ouverts selon leur extension. `--modal`, `--panel`, `--yaw`, `--shot` et `--start` fixent une fenêtre, le panneau, la rotation de la vue 3D, une image de la fenêtre et le démarrage du travail, pour les images.
* **Machine :** via les moteurs de EH3D-DRILLPRO et EH3D-ROUTEPRO et le fichier de machine qu'ils partagent : le RS-232 de l'ordinateur, un adaptateur USB-RS232, le EH3D-NODE-BUNGARD ou EH3D-SERVER. Sans fichier de machine, le travail s'exécute sur le simulateur.

## ⚙️ Configuration

La connexion est le **fichier de machine** de EH3D-DRILLPRO et EH3D-ROUTEPRO (`machine.cfg`, avec `connection = demo|rs232|usb|node|server` et chaque valeur réglée sur une vraie machine) : `--connection FICHIER` y pointe, et sans lui le travail s'exécute sur le simulateur. `--data DOSSIER` est le dossier des fichiers de machine et d'outils des programmes classiques (`DRILLPRO.DCF`, `DRILLPRO.DTD`, `ROUTEPRO.RCF`, `ROUTEPRO.RTD` ; le dossier `testdata/` contient des exemples). La fenêtre des réglages change la largeur et la profondeur utiles de la table et les enregistre dans ces fichiers. Ce que la fenêtre mémorise (le choix du lanceur, la langue et le thème) est gardé dans un petit fichier du dossier de réglages de l'utilisateur, avec les autres programmes EH3D. `--theme forge|neon|glass|windows-classic|system` et `--lang en|es|it|de|fr|pt|zh|ja` choisissent l'aspect et la langue.

## 🚀 Démarrage rapide

```bash
cargo build --release --features gui
cargo test --features gui
cargo clippy --all-targets --features gui -- -D warnings
cargo fmt --all --check
python -I tools/ci_validate.py
```

Lancez-le avec `run.bat` (Windows) ou `./run.sh` (Linux et macOS) ; les deux compilent d'abord la fenêtre si besoin et, sans argument, ouvrent le lanceur avec la carte de test de `testdata/`. Ils demandent Rust et les dossiers voisins `EH3D-COMMON`, `EH3D-DRILLPRO` et `EH3D-ROUTEPRO`. `cargo run --release --features gui -- --zip TRAVAIL.ZIP` ouvre un travail empaqueté dans une archive ZIP. Sous Linux, la fenêtre demande X11 ou Wayland.

## 🧪 Tests

La suite compte 73 tests (`cargo test --features gui`). Ils vérifient le lanceur et ce qu'il mémorise, les réglages, comment un fichier de perçage ou HP-GL devient la scène, les profils de machine, le panneau et la ligne de commande. Les moteurs qu'il utilise (protocole, mouvement, porte de sécurité) sont testés dans EH3D-DRILLPRO et EH3D-ROUTEPRO. La CI exécute aussi `cargo fmt --check` et `cargo clippy` avec les avertissements en erreurs, et `python -I tools/ci_validate.py` vérifie le manifeste, les versions, le journal des modifications et les huit README.

## 🏗️ Compilation et publication

La version se trouve dans `Cargo.toml` et dans `eh3d.project.json`, et chaque version a son entrée dans [CHANGELOG.md](CHANGELOG.md). `build.bat` / `build.sh` compilent la fenêtre (`cargo build --release --features gui`) et `test.bat` / `test.sh` lancent les tests ; aucun ne change la version.

**Paquets prêts à l'emploi :** le workflow `Release binaries` de ce dépôt compile le programme pour Windows (64 et 32 bits), Linux et macOS (Apple silicon et Intel), chaque paquet avec la licence, ce README et les fichiers de test comme exemples, et son SHA-256. Aucune version n'a encore été publiée.

## 📂 Structure du dépôt

<p align="center">
  <img src="images/repository-structure.svg" alt="Structure du dépôt EH3D-BUNGARD-GUI" width="100%">
</p>

<details>
<summary>Version texte de l'arborescence</summary>

```text
EH3D-BUNGARD-GUI/
├── src/
│   ├── gui.rs                                   la fenêtre : lanceur, vue en plan, vue 3D, avancement du travail
│   ├── gui/extra.rs                             ouvrir des fichiers, panélisation, réglages de la table, À propos, déplacer les vues
│   ├── gui/fleet.rs                             fenêtre des machines et stations du serveur
│   ├── gui/opener.rs  gui/board.rs              fenêtres Ouvrir, carte, outils et réglages
│   ├── jobs.rs                                  un fichier de perçage ou HP-GL transformé en scène dessinée par les vues
│   ├── boards.rs                                la bibliothèque des matériaux de base
│   ├── machines.rs                              les profils des machines Bungard et la zone de travail selon la table
│   ├── launcher.rs                              les choix du lanceur, les programmes classiques et ce qui est mémorisé
│   ├── link.rs                                  la liaison du fichier de la machine : les clés écrites
│   ├── cli.rs                                   la ligne de commande
│   └── lib.rs bin/                              la racine de la bibliothèque et l'exécutable bungard-gui
├── testdata/                                    une petite carte : fichiers de perçage et de contour, et fichiers d'exemple de machine et d'outils
├── docs/                                        les sources des deux images
├── tools/                                       contrôle CI, contrôle du README et outils des images
├── images/                                      bannière, les deux images et les captures d'écran
├── voices/                                      avis parlés
├── .github/                                     workflows de CI et de publication, modèles d'issues et de pull requests
├── build.sh build.bat                           compilent la fenêtre
├── run.sh run.bat                               démarrent la fenêtre, avec les fichiers de test sans argument
├── eh3d.project.json bump_manifest_version.py   manifeste et utilitaire de version
├── Cargo.toml Cargo.lock                        le paquet Rust et ses dépendances verrouillées
├── CHANGELOG.md                                 ce qui a changé dans chaque version
├── SECURITY.md SUPPORT.md                       comment signaler une vulnérabilité et où poser des questions
└── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   comment contribuer, le code de conduite et la licence
```

</details>

## 🗺️ Feuille de route et limites connues

**Limites connues aujourd'hui**

* Aucun travail n'a été piloté depuis cette fenêtre sur une vraie machine : il n'a tourné que sur le simulateur des moteurs de EH3D-DRILLPRO et EH3D-ROUTEPRO.
* La fenêtre de Connexion n'écrit dans le fichier de la machine que les clés de la liaison ; celle des réglages écrit les clés qu'elle offre (la table, le modèle, les unités, les sons et la vitesse de la simulation) et celle des outils les avances, les vitesses et les profondeurs ; le reste des fichiers de machine et d'outils n'est pas modifié ici.
* La pause retient le travail entre deux mouvements, pas au milieu d'un, et la vue 3D ne montre pas la hauteur des trajets.
* Seules les machines CCD et CCD/2 sont pilotées ; les profils de la CCD Premium et de la CCD XL sont dessinés, mais leur propre unité de commande n'est pas prise en charge.
* La fenêtre demande les dossiers voisins EH3D-COMMON, EH3D-DRILLPRO et EH3D-ROUTEPRO à côté d'elle, car elle en dépend par chemin.

**Ensuite**

* Une pause au milieu d'un mouvement, pas seulement entre deux mouvements.
* La hauteur (Z) des trajets dans la vue 3D.
* Charger et enregistrer toute la configuration, pas seulement la taille de la table.
* Un paquet avec les programmes classiques, pour qu'un seul téléchargement contienne tout.
* Un essai sur une vraie machine.

## 🌍 Langues

Le README et les textes existent en anglais, espagnol, français, italien, allemand, chinois, japonais et portugais européen.

## 🔗 Projets liés

* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO) : le programme de perçage ; cette fenêtre utilise son moteur et son programme classique
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO) : le programme de fraisage ; cette fenêtre utilise son moteur et son programme classique
* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON) : la vue (scène, thèmes, textes en huit langues) et le lecteur ZIP qu'utilise cette fenêtre
* [EH3D-NODE-BUNGARD](https://github.com/JuanenRac/EH3D-NODE-BUNGARD) : un nœud ESP32-S3 placé dans le boîtier de la machine, qui envoie les pas par son propre port série
* [EH3D-SERVER](https://github.com/JuanenRac/EH3D-SERVER) : le centre de la famille EH3D : il peut exécuter le travail sur la machine via son nœud

## 📚 Documentation

* [Connexion et réglage](https://github.com/JuanenRac/EH3D-DRILLPRO/blob/main/docs/CONNECTION.md) : le fichier de la machine, les connexions et chaque valeur (EH3D-DRILLPRO)
* [Notes du protocole](https://github.com/JuanenRac/EH3D-DRILLPRO/blob/main/docs/PROTOCOL.md) : ce que l'on sait de la façon dont l'unité de commande est pilotée (EH3D-DRILLPRO)
* [Journal des modifications](CHANGELOG.md)

## 🤝 Contribuer et communauté

* Avant une pull request, lancez `cargo fmt --all --check`, `cargo clippy --all-targets --features gui -- -D warnings`, `cargo test --features gui` et `python -I tools/ci_validate.py` : tout doit passer. Voir [CONTRIBUTING.md](CONTRIBUTING.md) et le [code de conduite](CODE_OF_CONDUCT.md).
* Le README existe en huit langues : un changement de texte se fait dans les huit fichiers, et les deux images sont dessinées à partir de `docs/architecture.json` et `docs/structure.json` par `tools/make_architecture.py` et `tools/make_structure.py`.
* Ne décrivez pas comme fonctionnant sur une machine ce qui n'a été essayé que sur le simulateur.
* Signalez une vulnérabilité en privé : [SECURITY.md](SECURITY.md). Questions, idées et rapports : electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTEUR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENCE

Logiciel : GPL-3.0-or-later. Voir [LICENSE](LICENSE). Les conceptions matérielles de la suite utilisent la CERN OHL-S.
