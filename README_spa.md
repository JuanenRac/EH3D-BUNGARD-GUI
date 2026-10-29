<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-BUNGARD-GUI banner" width="100%">
</p>

# 🖥️ EH3D-BUNGARD-GUI

<p align="center"><a href="README.md">🇺🇸 English</a> | 🇪🇸 <b>Español</b> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### La ventana moderna para las máquinas Bungard CCD: un lanzador, una vista 2D y otra 3D del trabajo, taladrado y fresado juntos, en ocho idiomas

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Tests-32-0f6cbd.svg" alt="Tests">
  <img src="https://img.shields.io/badge/Version-0.0.1-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Comprobación de honestidad - lo que se ejecuta hoy:** un programa en Rust con una ventana y 32 pruebas que se ejecutan en un ordenador. El lanzador ofrece la interfaz moderna o los programas clásicos de taladrado y fresado para una máquina con cambio de herramienta manual (MTC) o automático (ATC). El visor abre ficheros de taladrado, ficheros de trazado HP-GL y archivos ZIP de trabajos (desde un explorador de carpetas, arrastrando y soltando o desde la línea de órdenes), dibuja la vista en planta con cada agujero por diámetro de broca y las trayectorias de fresado encima, muestra la máquina en 3D con la mesa del tamaño elegido en la configuración, paneliza la placa y sigue el trabajo mientras corre. El trabajo se ejecuta en el **simulador** de los motores de EH3D-DRILLPRO y EH3D-ROUTEPRO; el mismo fichero de máquina que ellos leen decide cómo se llega a una máquina real (el RS-232 del ordenador, un adaptador USB-RS232, o el EH3D-NODE-BUNGARD directamente o a través de EH3D-SERVER), y **ninguna de esas vías se ha probado desde esta ventana**. Todavía no hay pantalla de conexión, ni pausa, ni edición de los ficheros de máquina y herramientas, y la vista 3D no muestra la altura de las trayectorias.

---

> **Estado de seguridad (safety) y de seguridad informática:** este programa no es un sistema de seguridad. No gestiona finales de carrera ni una parada de emergencia, y nunca ha movido una máquina real. Con una conexión real en el fichero de la máquina, el trabajo movería la máquina a través de los motores de EH3D-DRILLPRO y EH3D-ROUTEPRO: lee antes sus notas, saca la herramienta, ten una mano en el interruptor de la máquina y está listo para parar el trabajo. El proyecto es independiente: no lo ha hecho, respaldado ni soportado Bungard Elektronik, y los nombres de producto aparecen solo para decir con qué formatos de fichero y máquinas está pensado para funcionar.

## 📋 De un vistazo

| | |
|---|---|
| Versión | `0.0.1` |
| Madurez | Andamiaje |
| Pruebas | 32 |
| Pila tecnológica | Rust |
| En hardware real | No. Se ha ejecutado solo en el simulador y en un ordenador; no se ha manejado ninguna máquina desde esta ventana |

**Ir a:** [Resumen](#-resumen) · [Arquitectura](#-arquitectura) · [Inicio rápido](#-inicio-rápido) · [Pruebas](#-pruebas) · [Estructura](#-estructura-del-repositorio) · [Relacionados](#-proyectos-relacionados)

## 🚦 Estado del proyecto

Cada línea es una parte de este repositorio y la luz dice hasta dónde llega hoy. Se escribe con lo que el repositorio puede demostrar (pruebas, compilaciones, ejecuciones), no con lo previsto.

🟢 Hecho y verificado · 🟡 Funciona, con límites conocidos · 🟠 Hecho a medias o sin verificar · 🔴 Sin hacer o sin funcionar

| Área | Estado | Qué significa aquí |
|---|---|---|
| Ventana y lanzador | 🟡 | Una ventana en Rust con un lanzador (la interfaz moderna o los programas clásicos), cinco temas y ocho idiomas; funciona en un ordenador y se ha mirado con imágenes del trabajo simulado. |
| Vistas | 🟡 | La vista en planta (agujeros por diámetro, trayectorias de fresado) y la vista 3D de la máquina con raíles, mesa perforada, puente móvil y almacén de herramientas, ajustada al tamaño de la mesa; ambas se arrastran y se amplían. La vista 3D aún no muestra la altura de las trayectorias. |
| Ficheros y panelización | 🟡 | Los ficheros de taladrado, HP-GL y los archivos ZIP se abren desde un explorador, arrastrando y soltando o desde la línea de órdenes; la placa se repite en tres modos. Probado con los pequeños ficheros de muestra de este repositorio. |
| Ajustes | 🟠 | El ancho y el fondo útiles de la mesa se cambian y se guardan en los ficheros de la máquina; el resto de los ficheros de máquina y herramientas se sigue editando con los programas clásicos. |
| Control de la máquina | 🔴 | El trabajo pasa por los motores de EH3D-DRILLPRO y EH3D-ROUTEPRO, así que en principio están todas sus conexiones; solo ha corrido en el simulador. La ventana de Conexión elige el enlace (el RS-232 del ordenador, un adaptador USB-RS232, el nodo EH3D o el servidor EH3D), lo prueba y lo guarda en el fichero de la máquina, y Pausa detiene el trabajo entre dos movimientos. |
| Pruebas | 🟡 | Pasan 32 pruebas en un ordenador (`cargo test --features gui`): el lanzador, los ajustes, la lectura de los ficheros a la escena, los perfiles de máquina, el panel y la línea de órdenes. Ninguna toca una máquina. |
| Paquetes ya compilados | 🟠 | Un flujo de trabajo compila el programa para Windows (64 y 32 bits), Linux y macOS con los ficheros de muestra; todavía no se ha publicado ninguna versión. |

## 🎯 Resumen

* El **lanzador**: la interfaz moderna, o los programas clásicos de taladrado y fresado para una máquina con cambio de herramienta manual o automático; se arrancan como los programas que son cuando están junto a este. La elección se puede recordar.
* La **vista en planta**: la mesa con cada agujero por diámetro de broca y las trayectorias de fresado encima, que se arrastra con el ratón y se amplía con la rueda; un botón (o un doble clic) devuelve la vista a su sitio.
* La **vista 3D de la máquina**: raíles, mesa perforada, puente móvil y almacén de herramientas, con la mesa del tamaño fijado en la configuración. Se gira arrastrando, se amplía con la rueda y se mueve con Mayús o el botón derecho. Los perfiles de máquina (CCD/2, CCD, CCD Premium y CCD XL) salen de las medidas publicadas; la máquina se deduce de la mesa de la configuración.
* **Abrir ficheros**: un fichero de taladrado, un fichero HP-GL o un archivo ZIP con un trabajo dentro, desde una ventana Abrir con explorador de carpetas y unidades, soltando los ficheros sobre la ventana, con `Ctrl+O`, o dados en la línea de órdenes. Un nombre dentro de un archivo que saldría de la carpeta de trabajo no se lista, y no se lee ningún fichero por encima de un límite de tamaño.
* **Panelización**: copias de la placa en columnas y filas, colocadas por la separación entre placas, por la distancia de una a la siguiente o tantas como quepan en la mesa; las copias se dibujan y el trabajo las hace.
* El **trabajo**: se ejecuta con los motores de los programas clásicos y su fichero de máquina (`demo` simulador, `rs232`, `usb`, `node` o `server`), muestra el progreso mientras corre y plantea al operador las preguntas de la máquina (el husillo, el cambio de herramienta).

## 🧩 Papel en EH3D

EH3D-BUNGARD-GUI es la interfaz moderna de las máquinas Bungard en la familia EH3D. Depende por ruta de las carpetas hermanas EH3D-COMMON (la escena, los temas y los textos en ocho idiomas de `eh3d-view`, y el lector de ZIP de `eh3d-archive`), EH3D-DRILLPRO y EH3D-ROUTEPRO (sus motores y sus programas clásicos): deben estar junto a esta.

## 🏛️ Arquitectura

<p align="center">
  <img src="images/architecture.svg" alt="Arquitectura de EH3D-BUNGARD-GUI: quién lo usa, qué contiene y de qué depende" width="100%">
</p>

Cómo encajan las partes de este repositorio: quién lo usa, qué contiene y de qué depende. La imagen se dibuja desde `docs/architecture.json` y se comprueba contra el código.

## ✨ Funciones

* Temas: Forge (el predeterminado), Neon Arcade, Glass Pro, el gris clásico de Windows y uno que sigue los colores de Windows.
* Ocho idiomas en toda la ventana: inglés, español, francés, italiano, alemán, chino, japonés y portugués de Portugal.
* Perfiles de máquina con las medidas publicadas y una mesa ajustable: la máquina en 3D crece con el área útil de la configuración.
* Trabajos en archivos ZIP: se muestran el primer fichero de taladrado y el primero de fresado, y los demás se listan para cargarlos con un clic.
* Panelización en tres modos (separación, paso, tantas como quepan) con las copias dibujadas en la mesa.
* Progreso del trabajo en vivo y las preguntas de la máquina al operador.
* Una ventana Acerca de con la versión, la licencia y la conexión en uso.

## 🔌 Interfaces

* **Ventana:** el lanzador y el visor, con el ratón y el teclado (`Enter` acepta la elección del lanzador, `Esc` vuelve atrás, `Ctrl+O` abre ficheros).
* **Línea de órdenes:** `bungard-gui [FICHERO...] [--drill FICHERO.NCD] [--route FICHERO.PLT] [--zip TRABAJO.ZIP] [--lang CÓDIGO] [--theme NOMBRE] [--data DIR] [--connection FICHERO] [--screen launcher|viewer]`; los ficheros sin opción se abren como dice su extensión. `--modal`, `--panel`, `--yaw`, `--shot` y `--start` fijan una ventana, el panel, el giro de la vista 3D, una imagen de la ventana y el inicio del trabajo, para las imágenes.
* **Máquina:** a través de los motores de EH3D-DRILLPRO y EH3D-ROUTEPRO y el fichero de máquina que comparten: el RS-232 del ordenador, un adaptador USB-RS232, el EH3D-NODE-BUNGARD o EH3D-SERVER. Sin fichero de máquina el trabajo se ejecuta en el simulador.

## ⚙️ Configuración

La conexión es el **fichero de la máquina** de EH3D-DRILLPRO y EH3D-ROUTEPRO (`machine.cfg`, con `connection = demo|rs232|usb|node|server` y cada valor afinado en una máquina real): `--connection FICHERO` apunta a él, y sin uno el trabajo se ejecuta en el simulador. `--data DIR` es la carpeta con los ficheros de máquina y herramientas de los programas clásicos (`DRILLPRO.DCF`, `DRILLPRO.DTD`, `ROUTEPRO.RCF`, `ROUTEPRO.RTD`; la carpeta `testdata/` tiene muestras). La ventana de ajustes cambia el ancho y el fondo útiles de la mesa y los guarda en esos ficheros. Lo que la ventana recuerda (la elección del lanzador, el idioma y el tema) se guarda en un pequeño fichero en la carpeta de ajustes del usuario, con los demás programas de EH3D. `--theme forge|neon|glass|windows-classic|system` y `--lang en|es|it|de|fr|pt|zh|ja` eligen el aspecto y el idioma.

## 🚀 Inicio rápido

```bash
cargo build --release --features gui
cargo test --features gui
cargo clippy --all-targets --features gui -- -D warnings
cargo fmt --all --check
python -I tools/ci_validate.py
```

Ejecútalo con `run.bat` (Windows) o `./run.sh` (Linux y macOS); ambos compilan la ventana primero si hace falta y, sin argumentos, abren el lanzador con la placa de prueba de `testdata/`. Necesitan Rust y las carpetas hermanas `EH3D-COMMON`, `EH3D-DRILLPRO` y `EH3D-ROUTEPRO`. `cargo run --release --features gui -- --zip TRABAJO.ZIP` abre un trabajo empaquetado en un archivo ZIP. En Linux la ventana necesita X11 o Wayland.

## 🧪 Pruebas

La suite tiene 32 pruebas (`cargo test --features gui`). Comprueban el lanzador y lo que recuerda, los ajustes, cómo un fichero de taladrado o HP-GL se convierte en la escena, los perfiles de máquina, el panel y la línea de órdenes. Los motores que usa (protocolo, movimiento, puerta de seguridad) se prueban en EH3D-DRILLPRO y EH3D-ROUTEPRO. La CI también ejecuta `cargo fmt --check` y `cargo clippy` con los avisos como errores, y `python -I tools/ci_validate.py` comprueba el manifiesto, las versiones, el registro de cambios y los ocho README.

## 🏗️ Compilación y publicación

La versión vive en `Cargo.toml` y en `eh3d.project.json`, y cada versión tiene su entrada en [CHANGELOG.md](CHANGELOG.md). `build.bat` / `build.sh` compilan la ventana (`cargo build --release --features gui`) y `test.bat` / `test.sh` ejecutan las pruebas; ninguno cambia la versión.

**Paquetes ya compilados:** el flujo `Release binaries` de este repositorio compila el programa para Windows (64 y 32 bits), Linux y macOS (Apple silicon e Intel), cada paquete con la licencia, este README y los ficheros de prueba como ejemplos, y su SHA-256. Todavía no se ha publicado ninguna versión.

## 📂 Estructura del repositorio

<p align="center">
  <img src="images/repository-structure.svg" alt="Estructura del repositorio EH3D-BUNGARD-GUI" width="100%">
</p>

<details>
<summary>Versión en texto del árbol</summary>

```text
EH3D-BUNGARD-GUI/
├── src/
│   ├── gui.rs                                   la ventana: lanzador, vista en planta, vista 3D, progreso del trabajo
│   ├── gui/extra.rs                             abrir ficheros, panelización, ajustes de la mesa, Acerca de, mover las vistas
│   ├── gui/opener.rs  gui/board.rs              ventanas Abrir, placa, herramientas y ajustes
│   ├── jobs.rs                                  un fichero de taladrado o HP-GL convertido en la escena que dibujan las vistas
│   ├── boards.rs                                la biblioteca de materiales base de muchos fabricantes
│   ├── machines.rs                              los perfiles de las máquinas Bungard y el área de trabajo según la mesa
│   ├── launcher.rs                              las opciones del lanzador, los programas clásicos y lo que se recuerda
│   ├── link.rs                                  la conexión del fichero de máquina: qué claves escribe
│   ├── cli.rs                                   la línea de órdenes
│   └── lib.rs bin/                              la raíz de la biblioteca y el ejecutable bungard-gui
├── testdata/                                    una placa pequeña: ficheros de taladrado y contorno, y los ficheros de muestra de máquina y herramientas
├── docs/                                        las fuentes de las dos imágenes
├── tools/                                       comprobación de CI, comprobación del README y herramientas de las imágenes
├── images/                                      banner y las dos imágenes
├── .github/                                     flujos de CI y de publicación, plantillas de issues y pull requests
├── build.sh build.bat                           compilan la ventana
├── run.sh run.bat                               arrancan la ventana, con los ficheros de prueba si no se dan argumentos
├── eh3d.project.json bump_manifest_version.py   manifiesto y utilidad de versión
├── Cargo.toml Cargo.lock                        el paquete de Rust y sus dependencias fijadas
├── CHANGELOG.md                                 qué cambió en cada versión
├── SECURITY.md SUPPORT.md                       cómo informar de una vulnerabilidad y dónde preguntar
└── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   cómo contribuir, el código de conducta y la licencia
```

</details>

## 🗺️ Hoja de ruta y límites conocidos

**Límites conocidos hoy**

* Ningún trabajo se ha manejado desde esta ventana en una máquina real: solo ha corrido en el simulador de los motores de EH3D-DRILLPRO y EH3D-ROUTEPRO.
* La ventana de Conexión solo escribe en el fichero de la máquina las claves del enlace; la ventana de ajustes solo cambia el tamaño de la mesa, y los ficheros de máquina y herramientas no se editan de otro modo aquí.
* La pausa detiene el trabajo entre dos movimientos, no en mitad de uno, y la vista 3D no muestra la altura de las trayectorias.
* Solo se manejan las máquinas CCD y CCD/2; los perfiles de la CCD Premium y la CCD XL se dibujan, pero su propia unidad de control no está soportada.
* La ventana necesita las carpetas hermanas EH3D-COMMON, EH3D-DRILLPRO y EH3D-ROUTEPRO junto a ella, porque depende de ellas por ruta.

**Siguiente**

* Una pausa en mitad de un movimiento, no solo entre dos movimientos.
* La altura (Z) de las trayectorias en la vista 3D.
* Cargar y guardar toda la configuración, no solo el tamaño de la mesa.
* Empaquetar con los programas clásicos, para que una sola descarga lo tenga todo.
* Una prueba en una máquina real.

## 🌍 Idiomas

El README y los textos existen en inglés, español, francés, italiano, alemán, chino, japonés y portugués de Portugal.

## 🔗 Proyectos relacionados

* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO): el programa de taladrado; esta ventana usa su motor y su programa clásico
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO): el programa de fresado; esta ventana usa su motor y su programa clásico
* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON): la vista (escena, temas, textos en ocho idiomas) y el lector de ZIP que usa esta ventana
* [EH3D-NODE-BUNGARD](https://github.com/JuanenRac/EH3D-NODE-BUNGARD): un nodo ESP32-S3 que va en la caja de la máquina y saca los pasos por su propio puerto serie
* [EH3D-SERVER](https://github.com/JuanenRac/EH3D-SERVER): el centro de la familia EH3D: puede ejecutar el trabajo en la máquina a través de su nodo

## 📚 Documentación

* [Conexión y ajuste](https://github.com/JuanenRac/EH3D-DRILLPRO/blob/main/docs/CONNECTION.md): el fichero de la máquina, las conexiones y cada valor (EH3D-DRILLPRO)
* [Notas del protocolo](https://github.com/JuanenRac/EH3D-DRILLPRO/blob/main/docs/PROTOCOL.md): lo que se sabe de cómo se maneja la unidad de control (EH3D-DRILLPRO)
* [Registro de cambios](CHANGELOG.md)

## 🤝 Contribuir y comunidad

* Antes de una pull request ejecuta `cargo fmt --all --check`, `cargo clippy --all-targets --features gui -- -D warnings`, `cargo test --features gui` y `python -I tools/ci_validate.py`: todo debe pasar. Consulta [CONTRIBUTING.md](CONTRIBUTING.md) y el [código de conducta](CODE_OF_CONDUCT.md).
* El README existe en ocho idiomas: un cambio en el texto se hace en los ocho archivos, y las dos imágenes se dibujan a partir de `docs/architecture.json` y `docs/structure.json` con `tools/make_architecture.py` y `tools/make_structure.py`.
* No describas como funcionando en una máquina lo que solo se ha probado en el simulador.
* Informa de una vulnerabilidad en privado: [SECURITY.md](SECURITY.md). Preguntas, ideas e informes: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTOR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENCIA

Software: GPL-3.0-or-later. Véase [LICENSE](LICENSE). Los diseños de hardware de la suite usan CERN OHL-S.
