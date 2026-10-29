<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-BUNGARD-GUI banner" width="100%">
</p>

# 🖥️ EH3D-BUNGARD-GUI

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | 🇨🇳 <b>中文</b> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### 面向 Bungard CCD 机器的现代窗口：启动器、作业的 2D 与 3D 视图、钻孔和铣削合二为一，支持八种语言

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Tests-39-0f6cbd.svg" alt="Tests">
  <img src="https://img.shields.io/badge/Version-0.0.2-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**诚实性检查 - 今天实际能运行的内容：**一个带窗口的 Rust 程序，有 39 项可在电脑上运行的测试。启动器提供现代界面，或用于手动 (MTC) 或自动 (ATC) 换刀机器的经典钻孔与铣削程序。查看器可从文件夹浏览器、拖放或命令行打开钻孔文件、HP-GL 绘图文件和包含作业的 ZIP 压缩包，在平面视图中按钻头尺寸绘出每个孔并在其上叠加铣削路径，用配置中选定尺寸的工作台以 3D 显示机器，对电路板拼板，并在作业运行时跟踪进度。作业运行在 EH3D-DRILLPRO 和 EH3D-ROUTEPRO 引擎的**模拟器**上；如何连接真实机器（电脑的 RS-232、USB-RS232 适配器，或直接或经由 EH3D-SERVER 连接 EH3D-NODE-BUNGARD）由它们读取的同一个机器文件决定，而**这些方式都没有在此窗口中试过**。目前还没有连接界面、暂停功能，也不能编辑机器和刀具文件，3D 视图也不显示路径的高度。

---

> **安全（safety）与信息安全现状：**本程序不是安全系统。它不处理限位开关或紧急停止，也从未驱动过真实机器。若在机器文件中配置真实连接，作业会通过 EH3D-DRILLPRO 和 EH3D-ROUTEPRO 的引擎移动机器：请先阅读它们的说明，取下刀具，一只手放在机器的电源开关上，并随时准备停止作业。本项目是独立的：它不是由 Bungard Elektronik 制造、认可或支持的，出现产品名称只是为了说明它打算配合哪些文件格式和机器使用。

## 📋 速览

| | |
|---|---|
| 版本 | `0.0.2` |
| 成熟度 | 脚手架 |
| 测试 | 39 |
| 技术栈 | Rust |
| 真实硬件 | 否。只在模拟器和电脑上运行过；从未通过此窗口驱动过任何机器 |

**跳转到：** [概述](#-概述) · [架构](#-架构) · [快速开始](#-快速开始) · [测试](#-测试) · [结构](#-仓库结构) · [相关项目](#-相关项目)

## 🚦 项目状态

每一行是本仓库的一个部分，灯号表示它今天做到了哪一步。内容依据仓库能够展示的证据（测试、构建、运行），而不是计划。

🟢 已完成并验证 · 🟡 可用，有已知限制 · 🟠 部分完成或尚未验证 · 🔴 未完成或不可用

| 领域 | 状态 | 在这里的含义 |
|---|---|---|
| 窗口与启动器 | 🟡 | 带启动器（现代界面或经典程序）、五种主题和八种语言的 Rust 窗口；可在电脑上运行，并用模拟作业的图片查看过。 |
| 视图 | 🟡 | 平面视图（按钻头尺寸显示孔、铣削路径）和机器的 3D 视图（导轨、带孔工作台、移动龙门架、刀架），随工作台尺寸调整；两者都可拖动和缩放。3D 视图尚不显示路径的高度。 |
| 文件与拼板 | 🟡 | 钻孔文件、HP-GL 文件和 ZIP 压缩包可从浏览器、拖放或命令行打开；电路板可按三种模式重复。已用本仓库的小型示例文件试过。 |
| 设置 | 🟠 | 可更改工作台的可用宽度和深度并保存到机器文件；机器和刀具文件的其余部分仍用经典程序编辑。 |
| 机器控制 | 🔴 | 作业通过 EH3D-DRILLPRO 和 EH3D-ROUTEPRO 的引擎，因此原则上具备它们的所有连接；只在模拟器上运行过。连接窗口可选择连接方式（计算机的 RS-232、USB-RS232 适配器、EH3D 节点或 EH3D 服务器），测试并保存到机器文件；暂停会在两次移动之间保持作业。 |
| 测试 | 🟡 | 电脑上通过 39 项测试（`cargo test --features gui`）：启动器、设置、把文件读入场景、机器配置、面板和命令行。没有任何一项接触真实机器。 |
| 预编译包 | 🟠 | 工作流为 Windows（64 位和 32 位）、Linux 和 macOS 构建带示例文件的程序；尚未发布版本。 |

## 🎯 概述

* **启动器**：现代界面，或用于手动或自动换刀机器的经典钻孔与铣削程序；当它们位于本程序旁边时，会作为原本的程序启动。选择可以被记住。
* **平面视图**：按钻头尺寸显示每个孔、并叠加铣削路径的工作台，用鼠标拖动、滚轮缩放；按钮（或双击）可使视图复位。
* **机器的 3D 视图**：导轨、带孔工作台、移动龙门架和刀架，工作台尺寸取自配置。拖动可旋转，滚轮可缩放，按住 Shift 或用右键可平移。机器配置（CCD/2、CCD、CCD Premium 和 CCD XL）来自公布的尺寸；机器由配置中的工作台判定。
* **打开文件**：钻孔文件、HP-GL 文件或包含作业的 ZIP 压缩包，可通过带文件夹浏览器和驱动器的“打开”窗口、把文件拖到窗口上、`Ctrl+O` 或命令行打开。压缩包中会跑出工作文件夹的名称不会被列出，超过大小限制的文件不会被读取。
* **拼板**：把电路板按列和行复制，按板间距、相邻板的距离或工作台能放下的最大数量排列；副本会被绘出，作业也会加工它们。
* **作业**：使用经典程序的引擎及其机器文件运行（`demo` 模拟器、`rs232`、`usb`、`node` 或 `server`），显示进度，并向操作员提出机器的问题（主轴、换刀）。
* **电路板和刀具**：包含众多厂商（Bungard、Fortex、CIF、通用 FR-4、CEM 和 FR-2、Isola、Panasonic、Shengyi、ITEQ、Ventec、Nelco、Arlon、Rogers、Taconic、Bergquist、DuPont）基材的库，按材料、厚度、铜箔、覆铜面数和尺寸选择并记住；作业的刀具按钻孔或铣削列出其进给和转速，正在工作的会亮起；每个数值都有默认值（依据钻头直径和电路板材料），也可以点击后输入。
* **设置**：主题、语言、工作台可用尺寸、模拟速度（作业所用时间与机器相同，从 x0.5 到 x50）、作业在换刀和主轴启动时是否询问，以及下次是否直接打开此窗口。按钮在指针下会发亮，按下时会下沉。

## 🧩 在 EH3D 中的角色

EH3D-BUNGARD-GUI 是 EH3D 家族中 Bungard 机器的现代界面。它按路径依赖相邻的文件夹 EH3D-COMMON（`eh3d-view` 的场景、主题和八种语言的文本，以及 `eh3d-archive` 的 ZIP 读取器）、EH3D-DRILLPRO 和 EH3D-ROUTEPRO（它们的引擎和经典程序）：这些文件夹必须与本文件夹并列。

## 🏛️ 架构

<p align="center">
  <img src="images/architecture.svg" alt="EH3D-BUNGARD-GUI 的架构：谁在使用、包含什么、依赖什么" width="100%">
</p>

本仓库各部分如何协作：谁在使用它、它包含什么、它依赖什么。图片由 `docs/architecture.json` 绘制，并与代码核对。

## ✨ 功能

* 主题：Forge（默认）、Neon Arcade、Glass Pro、经典 Windows 灰色，以及跟随 Windows 颜色的主题。
* 整个窗口支持八种语言：英语、西班牙语、法语、意大利语、德语、中文、日语和欧洲葡萄牙语。
* 带公布尺寸的机器配置和可调工作台：3D 中的机器随配置的可用区域变大。
* ZIP 压缩包中的作业：显示第一个钻孔文件和第一个铣削文件，其余的列出，点击即可载入。
* 三种拼板模式（间距、步距、尽可能多）并把副本绘在工作台上。
* 作业进度实时显示，机器向操作员提出的问题。
* 显示版本、许可证和所用连接的“关于”窗口。

## 🔌 接口

* **窗口：**启动器和查看器，用鼠标和键盘操作（`Enter` 确认启动器的选择，`Esc` 返回，`Ctrl+O` 打开文件）。
* **命令行：**`bungard-gui [文件...] [--drill FILE.NCD] [--route FILE.PLT] [--zip JOB.ZIP] [--lang CODE] [--theme NAME] [--data DIR] [--connection FILE] [--screen launcher|viewer]`；没有选项的文件按其扩展名打开。`--modal`、`--panel`、`--yaw`、`--shot` 和 `--start` 为制图而设定窗口、面板、3D 视图的旋转、窗口图片以及作业的开始。
* **机器：**通过 EH3D-DRILLPRO 和 EH3D-ROUTEPRO 的引擎及它们共用的机器文件：电脑的 RS-232、USB-RS232 适配器、EH3D-NODE-BUNGARD 或 EH3D-SERVER。没有机器文件时，作业在模拟器上运行。

## ⚙️ 配置

连接由 EH3D-DRILLPRO 和 EH3D-ROUTEPRO 的**机器文件**决定（`machine.cfg`，含 `connection = demo|rs232|usb|node|server` 以及在真实机器上调好的每个值）：`--connection FILE` 指向它，没有它时作业在模拟器上运行。`--data DIR` 是经典程序的机器和刀具文件所在的文件夹（`DRILLPRO.DCF`、`DRILLPRO.DTD`、`ROUTEPRO.RCF`、`ROUTEPRO.RTD`；`testdata/` 中有示例）。设置窗口可更改工作台的可用宽度和深度，并保存到这些文件。窗口记住的内容（启动器的选择、语言和主题）保存在用户设置文件夹里的一个小文件中，与其他 EH3D 程序放在一起。`--theme forge|neon|glass|windows-classic|system` 和 `--lang en|es|it|de|fr|pt|zh|ja` 选择外观和语言。

## 🚀 快速开始

```bash
cargo build --release --features gui
cargo test --features gui
cargo clippy --all-targets --features gui -- -D warnings
cargo fmt --all --check
python -I tools/ci_validate.py
```

用 `run.bat`（Windows）或 `./run.sh`（Linux 和 macOS）运行；两者都会在需要时先构建窗口，没有参数时用 `testdata/` 中的测试板打开启动器。需要 Rust 以及相邻的文件夹 `EH3D-COMMON`、`EH3D-DRILLPRO` 和 `EH3D-ROUTEPRO`。`cargo run --release --features gui -- --zip JOB.ZIP` 打开打包在 ZIP 压缩包中的作业。在 Linux 上窗口需要 X11 或 Wayland。

## 🧪 测试

测试套件有 39 项测试（`cargo test --features gui`）。它们检查启动器及其记住的内容、设置、钻孔或 HP-GL 文件如何变成场景、机器配置、面板和命令行。它使用的引擎（协议、运动、安全门）在 EH3D-DRILLPRO 和 EH3D-ROUTEPRO 中测试。CI 还运行 `cargo fmt --check` 和把警告当作错误的 `cargo clippy`，`python -I tools/ci_validate.py` 检查清单、版本、更新日志和八个 README。

## 🏗️ 构建与发布

版本号位于 `Cargo.toml` 和 `eh3d.project.json`，每个版本在 [CHANGELOG.md](CHANGELOG.md) 中都有条目。`build.bat` / `build.sh` 构建窗口（`cargo build --release --features gui`），`test.bat` / `test.sh` 运行测试；两者都不会更改版本。

**预编译包：**本仓库的 `Release binaries` 工作流为 Windows（64 位和 32 位）、Linux 和 macOS（Apple 芯片和 Intel）构建程序，每个包含许可证、本 README、作为示例的测试文件及其 SHA-256。尚未发布版本。

## 📂 仓库结构

<p align="center">
  <img src="images/repository-structure.svg" alt="EH3D-BUNGARD-GUI 的仓库结构" width="100%">
</p>

<details>
<summary>目录树的文本版</summary>

```text
EH3D-BUNGARD-GUI/
├── src/
│   ├── gui.rs                                   窗口：启动器、平面视图、3D 视图、作业进度
│   ├── gui/extra.rs                             打开文件、拼板、工作台设置、关于、移动视图
│   ├── gui/opener.rs  gui/board.rs              打开、电路板、刀具和设置窗口
│   ├── jobs.rs                                  把钻孔或 HP-GL 文件转换为视图绘制的场景
│   ├── boards.rs                                众多厂商的基材库
│   ├── machines.rs                              Bungard 机器的配置文件，以及按工作台尺寸得出的工作区
│   ├── launcher.rs                              启动器的选项、经典程序以及记住的设置
│   ├── link.rs                                  机器文件中的连接：写入哪些键
│   ├── cli.rs                                   命令行
│   └── lib.rs bin/                              库的根文件和可执行文件 bungard-gui
├── testdata/                                    一块小电路板：钻孔和轮廓文件，以及示例机器和刀具文件
├── docs/                                        两张图片的来源
├── tools/                                       CI 检查、README 检查和图片工具
├── images/                                      横幅图和两张图片
├── .github/                                     CI 与发布工作流，Issue 和拉取请求模板
├── build.sh build.bat                           构建窗口
├── run.sh run.bat                               启动窗口，无参数时使用测试文件
├── eh3d.project.json bump_manifest_version.py   清单和版本工具
├── Cargo.toml Cargo.lock                        Rust 包及其锁定的依赖
├── CHANGELOG.md                                 每个版本的变化
├── SECURITY.md SUPPORT.md                       如何报告漏洞以及在哪里提问
└── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   如何贡献、行为准则和许可证
```

</details>

## 🗺️ 路线图与已知限制

**目前已知的限制**

* 从未通过此窗口在真实机器上驱动过作业：只在 EH3D-DRILLPRO 和 EH3D-ROUTEPRO 引擎的模拟器上运行过。
* 连接窗口只把连接的键写入机器文件；设置窗口只更改工作台尺寸，机器和刀具文件在此不做其他编辑。
* 暂停只在两次移动之间保持作业，不会在移动中途暂停；3D 视图也不显示路径的高度。
* 只驱动 CCD 和 CCD/2 机器；CCD Premium 和 CCD XL 的配置会被绘出，但不支持它们自己的控制单元。
* 窗口需要相邻的文件夹 EH3D-COMMON、EH3D-DRILLPRO 和 EH3D-ROUTEPRO，因为它按路径依赖它们。

**下一步**

* 在移动中途暂停，而不只是在两次移动之间。
* 3D 视图中路径的高度 (Z)。
* 载入和保存完整配置，而不仅是工作台尺寸。
* 与经典程序一起打包，一次下载即可获得全部。
* 在真实机器上测试。

## 🌍 语言

README 和文本提供英语、西班牙语、法语、意大利语、德语、汉语、日语和欧洲葡萄牙语版本。

## 🔗 相关项目

* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO)：钻孔程序；本窗口使用它的引擎和经典程序
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO)：铣削程序；本窗口使用它的引擎和经典程序
* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON)：本窗口使用的视图（场景、主题、八种语言的文本）和 ZIP 读取器
* [EH3D-NODE-BUNGARD](https://github.com/JuanenRac/EH3D-NODE-BUNGARD)：装在机器机箱内、通过自己的串口发出步进信号的 ESP32-S3 节点
* [EH3D-SERVER](https://github.com/JuanenRac/EH3D-SERVER)：EH3D 家族的中心：可通过其节点在机器上运行作业

## 📚 文档

* [连接与调整](https://github.com/JuanenRac/EH3D-DRILLPRO/blob/main/docs/CONNECTION.md)：机器文件、连接方式和每个值（EH3D-DRILLPRO）
* [协议笔记](https://github.com/JuanenRac/EH3D-DRILLPRO/blob/main/docs/PROTOCOL.md)：关于如何驱动控制单元的已知信息（EH3D-DRILLPRO）
* [更新日志](CHANGELOG.md)

## 🤝 参与贡献与社区

* 提交拉取请求之前，运行 `cargo fmt --all --check`、`cargo clippy --all-targets --features gui -- -D warnings`、`cargo test --features gui` 和 `python -I tools/ci_validate.py`：全部必须通过。参见 [CONTRIBUTING.md](CONTRIBUTING.md) 和[行为准则](CODE_OF_CONDUCT.md)。
* README 有八种语言：文本的改动要在八个文件中同时进行，两张图片由 `tools/make_architecture.py` 和 `tools/make_structure.py` 根据 `docs/architecture.json` 和 `docs/structure.json` 绘制。
* 不要把只在模拟器上试过的内容写成在机器上可用。
* 请私下报告漏洞：[SECURITY.md](SECURITY.md)。问题、想法和报告：electrohobby3d@gmail.com（[SUPPORT.md](SUPPORT.md)）。

## 👤 作者

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 许可证

软件：GPL-3.0-or-later。见 [LICENSE](LICENSE)。套件的硬件设计使用 CERN OHL-S。
