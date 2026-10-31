<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-BUNGARD-GUI banner" width="100%">
</p>

# 🖥️ EH3D-BUNGARD-GUI

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | 🇨🇳 <b>中文</b> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### 面向 Bungard CCD 机器的现代窗口：启动器、作业的 2D 与 3D 视图、钻孔和铣削合二为一，支持37 种语言

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Tests-134-0f6cbd.svg" alt="Tests">
  <img src="https://img.shields.io/badge/Version-0.1.2-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**诚实性检查 - 今天实际能运行的内容：**一个带窗口的 Rust 程序，有 134 项可在电脑上运行的测试。启动器提供现代界面，或用于手动 (MTC) 或自动 (ATC) 换刀机器的经典钻孔与铣削程序。查看器可从文件夹浏览器、拖放或命令行打开钻孔文件、HP-GL 绘图文件和包含作业的 ZIP 压缩包，在平面视图中按钻头尺寸绘出每个孔并在其上叠加铣削路径，用配置中选定尺寸的工作台以 3D 显示机器，对电路板拼板，并在作业运行时跟踪进度。作业运行在 EH3D-DRILLPRO 和 EH3D-ROUTEPRO 引擎的**模拟器**上；如何连接真实机器（电脑的 RS-232、USB-RS232 适配器，或直接或经由 EH3D-SERVER 连接 EH3D-NODE-BUNGARD）由它们读取的同一个机器文件决定，而**这些方式都没有在此窗口中试过**。连接、设置和刀具窗口只更改它们提供的值，完整的机器和刀具文件还不能编辑，3D 视图也不显示路径的高度。

---

> **安全（safety）与信息安全现状：**本程序不是安全系统。它不处理限位开关或紧急停止，也从未驱动过真实机器。若在机器文件中配置真实连接，作业会通过 EH3D-DRILLPRO 和 EH3D-ROUTEPRO 的引擎移动机器：请先阅读它们的说明，取下刀具，一只手放在机器的电源开关上，并随时准备停止作业。本项目是独立的：它不是由 Bungard Elektronik 制造、认可或支持的，出现产品名称只是为了说明它打算配合哪些文件格式和机器使用。

## 📋 速览

| | |
|---|---|
| 版本 | `0.1.2` |
| 成熟度 | 脚手架 |
| 测试 | 134 |
| 技术栈 | Rust |
| 真实硬件 | 否。只在模拟器和电脑上运行过；从未通过此窗口驱动过任何机器 |

**跳转到：** [概述](#-概述) · [架构](#-架构) · [快速开始](#-快速开始) · [测试](#-测试) · [结构](#-仓库结构) · [相关项目](#-相关项目)

## 🚦 项目状态

每一行是本仓库的一个部分，灯号表示它今天做到了哪一步。内容依据仓库能够展示的证据（测试、构建、运行），而不是计划。

🟢 已完成并验证 · 🟡 可用，有已知限制 · 🟠 部分完成或尚未验证 · 🔴 未完成或不可用

| 领域 | 状态 | 在这里的含义 |
|---|---|---|
| 窗口与启动器 | 🟡 | 带启动器（现代界面或经典程序）、五种主题和37 种语言的 Rust 窗口；可在电脑上运行，并用模拟作业的图片查看过。 |
| 视图 | 🟡 | 平面视图（按钻头尺寸显示孔、铣削路径）和机器的 3D 视图（导轨、带孔工作台、移动龙门架、刀架），随工作台尺寸调整；两者都可拖动和缩放。3D 视图尚不显示路径的高度。 |
| 文件与拼板 | 🟡 | 钻孔文件、HP-GL 文件和 ZIP 压缩包可从浏览器、拖放或命令行打开；电路板可按三种模式重复。已用本仓库的小型示例文件试过。 |
| 设置 | 🟠 | 可更改工作台的可用宽度和深度并保存到机器文件；机器和刀具文件的其余部分仍用经典程序编辑。 |
| 机器控制 | 🔴 | 作业通过 EH3D-DRILLPRO 和 EH3D-ROUTEPRO 的引擎，因此原则上具备它们的所有连接；只在模拟器上运行过。连接窗口可选择连接方式（计算机的 RS-232、USB-RS232 适配器、EH3D 节点或 EH3D 服务器），测试并保存到机器文件；暂停会在两次移动之间保持作业。 |
| 测试 | 🟡 | 电脑上通过 134 项测试（`cargo test --features gui`）：启动器、设置、把文件读入场景、机器配置、面板和命令行。没有任何一项接触真实机器。 |
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
* **拼板外轮廓的连接点：** 在拼板窗口中，外轮廓可以留连接点铣削，连接点在掰开之前固定各块板：每边几个、多宽、在哪里（均匀分布、靠近两端或集中在中间）；平面视图显示切开的路径。板材厂商、厚度和尺寸也可在下拉列表中选择。
* **工作台校准：** 设置窗口的一个标签页在点阵上测量工作台高度（默认 7 x 5），绘制高度图（蓝色低、红色高），显示高低差和每米倾斜（毫米），并随机床保存。探头节点接入前探头为模拟，作业的修正尚未应用。
* **崩溃后作业可继续：** 作业运行时，继续所需的内容（文件、拼板、已钻的孔）会实时保存；如果窗口、电脑或连接中断，下次启动会询问是从中断处继续还是取消。窗口以最大化打开，作业按钮色彩鲜明（绿色开始、蓝色暂停、红色停止）。
* **语音提示：** 除了音调，作业结束和请求刀具也可以由语音说出，语言和性别在同一声音列表中选择（`sound_finish = voice:es:f`）。`voices/` 文件夹中的语音是合成的（Piper）且可自由分发，句子缓慢而正式；27 种语言有语音（前 7 种，加上保加利亚语、捷克语、丹麦语、荷兰语、爱沙尼亚语、芬兰语、希腊语、匈牙利语、拉脱维亚语、挪威语、波兰语、罗马尼亚语、斯洛伐克语、斯洛文尼亚语、瑞典语、乌克兰语、俄语、阿尔巴尼亚语、巴斯克语和巴西葡萄牙语；新语音的性别没有听过，请核对），其余语言（日语、克罗地亚语、爱尔兰语、马耳他语、加泰罗尼亚语、加利西亚语、塞尔维亚语、冰岛语和立陶宛语）没有可自由分发的语音，会播放音调；没有中文或葡萄牙语的女声，也没有真人录音：把你自己的 WAV 文件放进相同的文件夹即可替换。见 `voices/README.txt`。
* **刀具目录：** 一个窗口（从刀具窗口打开）列出 450 多把刀具，均为 Bungard 夹头使用的 1/8 英寸（3.175 mm）刀柄，分十二类：从 0.05 mm 微型系列到 7 mm 的钻头（小号、中号和带断屑槽的大号）、长孔钻头、菱形纹、下切、上切、断屑和金刚石涂层铣刀、平头铣刀、V 型刀、球头铣刀、定位钻和倒角钻。每把刀具都有系列、刃长，以及制造商 FR-4 表中的转速、进给和每刀深度；点击一把即可绘出刀具（含尺寸）并说明用途，转速低于表中数值的主轴（默认 24 000 转，可在窗口中修改）会得到进给按比例降低的同一把刀具。V 型刀会给出某一深度下的槽宽。 示例板带有四条线路的隔离，用 V 型刀切出（铣削表的 3 号刀具）。
* **作业跟随工作台，以及钻头节点：** 校准页的一个复选框让钻孔和铣削用工作台高度图修正 Z（仅限完整的图，仅在要求时）；一个窗口（从连接窗口打开）设置如何连接 EH3D-NODE-BUNGARD-AUX（不使用、直连或经服务器）、它的地址和令牌以及是否显示其摄像头，并显示摄像头画面、主轴转速和温度、保护输出以及钻头下方工作台的一次测量。经服务器时，会用连接窗口里的地址、令牌和机器编号通过服务器查询节点（节点的令牌由服务器保管）；校准用的探头仍是模拟的。
* **更易查看和移动：** 作业卡片中的刀具列表可滚动，不再把图层挤出窗口；每个打开的下拉列表都有可用的滚动条（拖动、点击、滚轮）；2D 视图中的按钮只移动板子和工作台上的设计；每把铣刀在 2D 和 3D 视图中都有自己的颜色（含三把铣刀的文件会显示三种颜色）；即使只有一份副本，连接筋也会切在板子外轮廓上，且只切外轮廓，不会切内部的型腔和走线。
* **克制的动画：** 界面和窗口打开时淡入；作业运行时，刀头的圆环会呼吸并发出涟漪，待切割路径的虚线沿路径流动，一道光掠过进度条，正在工作的刀具在列表中呼吸。静止时不会有任何动画。

## 🧩 在 EH3D 中的角色

EH3D-BUNGARD-GUI 是 EH3D 家族中 Bungard 机器的现代界面。它按路径依赖相邻的文件夹 EH3D-COMMON（`eh3d-view` 的场景、主题和37 种语言的文本，以及 `eh3d-archive` 的 ZIP 读取器）、EH3D-DRILLPRO 和 EH3D-ROUTEPRO（它们的引擎和经典程序）：这些文件夹必须与本文件夹并列。

## 🏛️ 架构

<p align="center">
  <img src="images/architecture.svg" alt="EH3D-BUNGARD-GUI 的架构：谁在使用、包含什么、依赖什么" width="100%">
</p>

本仓库各部分如何协作：谁在使用它、它包含什么、它依赖什么。图片由 `docs/architecture.json` 绘制，并与代码核对。

## 🖼️ 屏幕截图

<p align="center">
  <img src="images/screenshots/viewer.png" alt="viewer" width="100%">
</p>

<p align="center"><em>查看器：工作台上的 2D 作业、3D 机器以及作业控制。</em></p>

<p align="center">
  <img src="images/screenshots/job-running.png" alt="job running" width="100%">
</p>

<p align="center"><em>运行中的作业：已完成的孔和路径、刀具位置，以及暂停和停止。</em></p>

<p align="center">
  <img src="images/screenshots/neon-theme.png" alt="Neon Arcade theme" width="100%">
</p>

<p align="center"><em>带拼板的 Neon Arcade 主题。</em></p>

<p align="center">
  <img src="images/screenshots/tools.png" alt="tools" width="100%">
</p>

<p align="center"><em>刀具：进给、转速和深度，电路板上每个钻头的孔数以及累计钻孔数。</em></p>

<p align="center">
  <img src="images/screenshots/machine-settings.png" alt="settings" width="100%">
</p>

<p align="center"><em>设置：每种 Bungard 型号、工作台和文件的单位。</em></p>

<p align="center">
  <img src="images/screenshots/board.png" alt="board" width="100%">
</p>

<p align="center"><em>电路板：库中的材料、厚度、铜箔和尺寸。</em></p>

<p align="center">
  <img src="images/screenshots/themes.png" alt="themes" width="100%">
</p>

<p align="center"><em>设置窗口中的十一种主题，每种都以窗口语言命名。</em></p>

<p align="center">
  <img src="images/screenshots/sounds.png" alt="sounds" width="100%">
</p>

<p align="center"><em>作业结束声音的列表：六十个声音，从高音到低音，另有标准音调、无声或自己的文件。</em></p>

<p align="center">
  <img src="images/screenshots/panel.png" alt="panel" width="100%">
</p>

<p align="center"><em>拼板窗口：板子的副本和外轮廓的连接点，通过下拉列表和计数器选择。</em></p>

<p align="center">
  <img src="images/screenshots/calibration.png" alt="calibration" width="100%">
</p>

<p align="center"><em>工作台校准：在点阵上测得的高度图，含高低差和倾斜。</em></p>

<p align="center">
  <img src="images/screenshots/paper-theme.png" alt="light theme" width="100%">
</p>

<p align="center"><em>纸张主题，一种浅色主题。</em></p>

<p align="center">
  <img src="images/screenshots/fleet.png" alt="machines and stations" width="100%">
</p>

<p align="center"><em>服务器的机床和工位：每台机床的状态、作业和进度，以及作业发往的机床。</em></p>

<p align="center">
  <img src="images/screenshots/head-node.png" alt="钻头节点" width="100%">
</p>

<p align="center"><em>钻头节点：摄像头、主轴转速和温度、保护输出，以及摄像头周围的 LED 环形灯（关和三档亮度）。</em></p>
<p align="center">
  <img src="images/screenshots/quality-check.png" alt="质量检查" width="100%">
</p>

<p align="center"><em>加工后的质量检查：摄像头查看电路板，作业中的每个孔都被标出（绿色正确、红色错误、琥珀色在画面外），测量铣削沟槽，结论大如招牌。</em></p>
<p align="center">
  <img src="images/screenshots/quality-tolerances.png" alt="公差" width="100%">
</p>

<p align="center"><em>工坊分析器的公差：孔的尺寸和位置、圆度、毛刺、沟槽的宽度和均匀度，以及图像清晰到可以判断的标准。</em></p>

## ✨ 功能

* 主题：Forge（默认）、Neon Arcade、Glass Pro、经典 Windows 灰色，以及跟随 Windows 颜色的主题。
* 整个窗口支持37 种语言：英语、西班牙语、法语、意大利语、德语、中文、日语和欧洲葡萄牙语。
* 带公布尺寸的机器配置和可调工作台：3D 中的机器随配置的可用区域变大。
* 机床和工位窗口：连接服务器时，按工位分组显示服务器的所有机床、每台的状态、作业及进度、就绪作业，并选择本窗口作业发往的机床；`--fleet FILE.json` 无需服务器即可显示已保存的回答。
* 在设置中选择机床型号后，两个视图都会适应其工作台：2D 平面图、3D 机床（该型号的机架、导轨、龙门和刀库）和缩放都按新尺寸重新开始。
* ZIP 压缩包中的作业：显示第一个钻孔文件和第一个铣削文件，其余的列出，点击即可载入。
* 三种拼板模式（间距、步距、尽可能多）并把副本绘在工作台上。
* 作业进度实时显示，机器向操作员提出的问题。
* 显示版本、许可证和所用连接的“关于”窗口。
* 分页设置，包含所有 Bungard MTC 和 ATC 型号（工作台、换刀器、限位）以及步进和坐标的单位（mm 或 mil），全部对加工产生实际作用。
* 可显示、隐藏和拖动的急停按钮；作业运行时 Start 熄灭，暂停或停止后恢复；模拟在后台继续运行。
* 跟随每把刀具的 2D 视图，可同时旋转钻孔和铣削文件并移动板子；包含一百多种基材的库；作业结束和换刀时的提示音。
* 其他程序的文件：Gerber 板的外形（`.GKO`、`.GM1`、`.GML`，或自称轮廓的 `.GBR`）和 DXF 图纸可作为铣削作业打开；绘图文件的圆和圆弧（`CI`、`AA`、`AR`）按配置中的弦角切成弦。
* 超出工作台的作业在开始前即被拒绝；“打开”窗口列出最近打开的文件；刀具窗口统计电路板上和所有作业中每个钻头的孔数，并有重新开始计数的按钮。
* 打开文件时（通过“打开”窗口、最近文件或拖放），刀具窗口会在文件上方打开，就像经典程序在加载文件后显示其刀具一样；设置中的复选框可将其关闭。
* 在选项很多的地方使用下拉列表：主题、语言、机床型号和每个事件的声音都从可滚动的列表中选择，而不是一排排按钮。作业结束和请求刀具各有一份 60 个声音的列表，从高音到低音（钟声、上行和下行音阶、号角、哔声、警报、敲击、警笛），另有标准音调、无声或自己的文件；选中即播放。
* **通过钻头摄像头控制灯光并进行质量检查：** 节点窗口可打开和关闭 LED 环形灯；“质量检查”窗口在加工后查看电路板：判断作业中的孔（是否存在、是否圆、尺寸是否正确、位置是否准确）、铣削沟槽（与刀具同宽、均匀、无变窄）以及图像本身，在图像上标出每个孔，以招牌大小显示结论（合格、不合格或无法判定），并让环形灯用颜色显示同样的结果。配置（标准、细间距、原型）、公差、比例及用已知孔校准的结果保存在文件中并交给服务器。结果还可以用声音播报：设置中新增两个声音，分别用于检查合格和不合格，每个都可以关闭，或选择提示音、目录中的声音、你自己的文件，或一段说明电路板目视检查合格或不合格的语音，与其他声音相同。
* **行为与真实机床一致的 3D 机床：** 静止时机头停在原点，在平面视图中移动电路板绝不会移动它。作业开始时机床先启动、回原点，移动到作业的第一个点，之后才开始加工，机头随加工移动（钻孔和铣削均如此）；结束后再回到原点。列表的滚动条带有上下箭头。
* **两种主轴和了解它们的目录：** 机床配备原装主轴（60000 转/分）或带变频器的主轴（24000 转/分），在设置中选择，各自的限制（转速、XY 与 Z 进给、最粗刀具）保存在设置文件中。在刀具窗口中，每把刀都有“目录”按钮：选择钻头或铣刀，其转速、进给和分层深度会写入该刀具。若主轴无法达到，窗口会提示：给出可用的建议值（每转切削量不变、XY 或 Z 更慢），或将刀具标为不兼容并禁止采用。电路板列表一次显示 16 种材料。
* **钻孔、铣削或两者，逐把刀具选择：** 在左侧面板和刀具窗口中可开关作业的钻孔和铣削（两者都开：先钻孔后铣削，依次进行，中间机床不回原点），点击每把刀具的圆点可将其关闭，作业会跳过它。
* **一次打开多个文件与专业示例：** “打开”窗口可用 Ctrl+点击或 Shift+点击选择两个或更多文件（钻孔和铣削一起；同类的多个文件合成一个作业，刀具按直径编号）。软件包的 `examples` 文件夹提供四个真实复杂度的设计，各带钻孔和铣削文件：32 位微控制器板（排针、USB 缺口、LQFP 过孔环）、四通道模拟前端（DIP 放大器、BNC 与接线端子、保护环、隔离槽）、2.4 GHz 放大器与滤波器（边缘 SMA、过孔围栏、屏蔽罩缝合、沿 50 欧姆线的隔离铣削）以及 5 V 3 A 电源（TO-220、电解电容、散热过孔）。
* **37 种语言：** 整个窗口支持英语、西班牙语、意大利语、德语、法语、葡萄牙语、中文、日语、保加利亚语、克罗地亚语、捷克语、丹麦语、荷兰语、爱沙尼亚语、芬兰语、希腊语、匈牙利语、爱尔兰语、拉脱维亚语、立陶宛语、马耳他语、波兰语、罗马尼亚语、斯洛伐克语、斯洛文尼亚语、瑞典语、挪威语、冰岛语、乌克兰语、俄语、塞尔维亚语、阿尔巴尼亚语、加泰罗尼亚语、加利西亚语、巴斯克语、巴西葡萄牙语和加拿大法语。 前八种之外的 29 种语言由模型翻译，等待母语者审校；某种语言缺少的文本以英文显示。 机床节点的网页面板同样支持全部 37 种；机床屏幕（HMI）目前仍只支持前八种。
* **Windows 上没有黑色窗口：** 带窗口的程序（本程序、DRILLPRO、ROUTEPRO 和 MOTIONCNC）在 Windows 上不再打开控制台；从终端启动时仍会在终端中输出帮助和错误。**可移动的窗口：** 主窗口之上的每个窗口都可拖动其顶部移动，右上角有 X 关闭按钮。

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

测试套件有 134 项测试（`cargo test --features gui`）。它们检查启动器及其记住的内容、设置、钻孔或 HP-GL 文件如何变成场景、机器配置、面板和命令行。它使用的引擎（协议、运动、安全门）在 EH3D-DRILLPRO 和 EH3D-ROUTEPRO 中测试。CI 还运行 `cargo fmt --check` 和把警告当作错误的 `cargo clippy`，`python -I tools/ci_validate.py` 检查清单、版本、更新日志和八个 README。

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
│   ├── gui/fleet.rs                             服务器机床和工位窗口
│   ├── inspection.rs gui/inspect.rs             通过摄像头检查加工质量
│   ├── gui/machine3d.rs                         3D 视图的机床：启动、回原点、第一个点、加工
│   ├── gui/opener.rs  gui/board.rs              打开、电路板、刀具和设置窗口
│   ├── jobs.rs                                  把钻孔或 HP-GL 文件转换为视图绘制的场景
│   ├── boards.rs                                众多厂商的基材库
│   ├── machines.rs                              Bungard 机器的配置文件，以及按工作台尺寸得出的工作区
│   ├── launcher.rs                              启动器的选项、经典程序以及记住的设置
│   ├── link.rs                                  机器文件中的连接：写入哪些键
│   ├── cli.rs                                   命令行
│   ├── lib.rs bin/                              库的根文件和可执行文件 bungard-gui
│   └── spindle.rs                               两种主轴、它们的限制，以及刀具是否适用
├── testdata/                                    一块小电路板：钻孔和轮廓文件，以及示例机器和刀具文件
├── docs/                                        两张图片的来源
├── tools/                                       CI 检查、README 检查和图片工具
├── images/                                      横幅图、两张图片和屏幕截图
├── voices/                                      语音提示
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
* 连接窗口只把连接的键写入机器文件；设置窗口写入它提供的键（工作台、型号、单位、声音和模拟速度），刀具窗口写入进给、转速和深度；机器和刀具文件的其余部分在此不做编辑。
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
* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON)：本窗口使用的视图（场景、主题、37 种语言的文本）和 ZIP 读取器
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
