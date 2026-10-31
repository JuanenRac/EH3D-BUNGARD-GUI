<p align="center">
  <img src="images/EH3D_BANNER.svg" alt="EH3D-BUNGARD-GUI banner" width="100%">
</p>

# 🖥️ EH3D-BUNGARD-GUI

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | 🇵🇹 <b>Português</b></p>

### A janela moderna para as máquinas Bungard CCD: um lançador, uma vista 2D e outra 3D do trabalho, furação e fresagem juntas, em oito idiomas

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Rust-DEA584.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Tests-117-0f6cbd.svg" alt="Tests">
  <img src="https://img.shields.io/badge/Version-0.0.9-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Verificação de honestidade - o que funciona hoje:** um programa em Rust com uma janela e 117 testes que correm num computador. O lançador oferece a interface moderna ou os programas clássicos de furação e fresagem para uma máquina com troca de ferramenta manual (MTC) ou automática (ATC). O visualizador abre ficheiros de furação, ficheiros de traçado HP-GL e arquivos ZIP de trabalhos (a partir de um explorador de pastas, por arrastar e largar ou pela linha de comandos), desenha a vista em planta com cada furo por diâmetro de broca e os trajetos de fresagem por cima, mostra a máquina em 3D com a mesa do tamanho escolhido na configuração, panelizza a placa e acompanha o trabalho enquanto decorre. O trabalho corre no **simulador** dos motores do EH3D-DRILLPRO e do EH3D-ROUTEPRO; o mesmo ficheiro de máquina que eles leem decide como se chega a uma máquina real (o RS-232 do computador, um adaptador USB-RS232, ou o EH3D-NODE-BUNGARD diretamente ou através do EH3D-SERVER), e **nenhuma dessas vias foi experimentada a partir desta janela**. As janelas de ligação, definições e ferramentas alteram os valores que oferecem, os ficheiros inteiros de máquina e ferramentas ainda não se editam, e a vista 3D não mostra a altura dos trajetos.

---

> **Estado de segurança (safety) e de segurança informática:** este programa não é um sistema de segurança. Não trata de fins de curso nem de uma paragem de emergência, e nunca moveu uma máquina real. Com uma ligação real no ficheiro da máquina, o trabalho moveria a máquina através dos motores do EH3D-DRILLPRO e do EH3D-ROUTEPRO: leia primeiro as suas notas, retire a ferramenta, mantenha uma mão no interruptor da máquina e esteja pronto a parar o trabalho. O projeto é independente: não é feito, recomendado nem apoiado pela Bungard Elektronik, e os nomes de produtos aparecem apenas para dizer com que formatos de ficheiro e máquinas se destina a funcionar.

## 📋 Num relance

| | |
|---|---|
| Versão | `0.0.9` |
| Maturidade | Estrutura inicial |
| Testes | 117 |
| Tecnologias | Rust |
| Em hardware real | Não. Só correu no simulador e num computador; nenhuma máquina foi comandada a partir desta janela |

**Ir para:** [Visão geral](#-visão-geral) · [Arquitetura](#-arquitetura) · [Início rápido](#-início-rápido) · [Testes](#-testes) · [Estrutura](#-estrutura-do-repositório) · [Relacionados](#-projetos-relacionados)

## 🚦 Estado do projeto

Cada linha é uma parte deste repositório e a luz diz até onde chega hoje. É escrita com o que o repositório consegue demonstrar (testes, compilações, execuções), não com o que está previsto.

🟢 Feito e verificado · 🟡 Funciona, com limites conhecidos · 🟠 Feito a meio ou ainda por verificar · 🔴 Por fazer ou a não funcionar

| Área | Estado | O que significa aqui |
|---|---|---|
| Janela e lançador | 🟡 | Uma janela em Rust com um lançador (a interface moderna ou os programas clássicos), cinco temas e oito idiomas; corre num computador e foi vista com imagens do trabalho simulado. |
| Vistas | 🟡 | A vista em planta (furos por diâmetro, trajetos de fresagem) e a vista 3D da máquina com calhas, mesa perfurada, pórtico móvel e arrumo de ferramentas, ajustada ao tamanho da mesa; ambas se arrastam e se ampliam. A vista 3D ainda não mostra a altura dos trajetos. |
| Ficheiros e panelização | 🟡 | Os ficheiros de furação, HP-GL e os arquivos ZIP abrem-se a partir de um explorador, por arrastar e largar ou pela linha de comandos; a placa repete-se em três modos. Experimentado com os pequenos ficheiros de exemplo deste repositório. |
| Definições | 🟠 | A largura e a profundidade úteis da mesa alteram-se e guardam-se nos ficheiros da máquina; o resto dos ficheiros de máquina e ferramentas continua a ser editado com os programas clássicos. |
| Controlo da máquina | 🔴 | O trabalho passa pelos motores do EH3D-DRILLPRO e do EH3D-ROUTEPRO, por isso em princípio estão lá todas as suas ligações; só correu no simulador. A janela de Ligação escolhe a ligação (o RS-232 do computador, um adaptador USB-RS232, o nó EH3D ou o servidor EH3D), experimenta-a e guarda-a no ficheiro da máquina, e Pausa retém o trabalho entre dois movimentos. |
| Testes | 🟡 | Passam 117 testes num computador (`cargo test --features gui`): o lançador, as definições, a leitura dos ficheiros para a cena, os perfis de máquina, o painel e a linha de comandos. Nenhum toca numa máquina. |
| Pacotes já compilados | 🟠 | Um fluxo de trabalho compila o programa para Windows (64 e 32 bits), Linux e macOS com os ficheiros de exemplo; ainda não foi publicada nenhuma versão. |

## 🎯 Visão geral

* O **lançador**: a interface moderna, ou os programas clássicos de furação e fresagem para uma máquina com troca de ferramenta manual ou automática; são iniciados como os programas que são quando estão junto a este. A escolha pode ser memorizada.
* A **vista em planta**: a mesa com cada furo por diâmetro de broca e os trajetos de fresagem por cima, arrastada com o rato e ampliada com a roda; um botão (ou um duplo clique) repõe a vista.
* A **vista 3D da máquina**: calhas, mesa perfurada, pórtico móvel e arrumo de ferramentas, com a mesa do tamanho definido na configuração. Roda-se arrastando, amplia-se com a roda e move-se com Shift ou o botão direito. Os perfis de máquina (CCD/2, CCD, CCD Premium e CCD XL) vêm das medidas publicadas; a máquina é reconhecida pela mesa da configuração.
* **Abrir ficheiros**: um ficheiro de furação, um ficheiro HP-GL ou um arquivo ZIP com um trabalho lá dentro, a partir de uma janela Abrir com explorador de pastas e unidades, largando os ficheiros sobre a janela, com `Ctrl+O`, ou indicados na linha de comandos. Um nome num arquivo que sairia da pasta de trabalho não é listado, e nenhum ficheiro é lido acima de um limite de tamanho.
* **Panelização**: cópias da placa em colunas e linhas, colocadas pelo espaço entre placas, pela distância de uma à seguinte ou tantas quantas caibam na mesa; as cópias são desenhadas e o trabalho executa-as.
* O **trabalho**: corre com os motores dos programas clássicos e o seu ficheiro de máquina (`demo` simulador, `rs232`, `usb`, `node` ou `server`), mostra o progresso e faz ao operador as perguntas da máquina (o fuso, a troca de ferramenta).
* A **placa e as ferramentas**: uma biblioteca de materiais de base de muitos fabricantes (Bungard, Fortex, CIF, FR-4 genérico, CEM e FR-2, Isola, Panasonic, Shengyi, ITEQ, Ventec, Nelco, Arlon, Rogers, Taconic, Bergquist, DuPont), escolhida por material, espessura, cobre, faces de cobre e tamanho, e memorizada; as ferramentas do trabalho são listadas como furação ou fresagem com o seu avanço e a sua velocidade, a que trabalha acende-se, e cada valor tem valores predefinidos (pelo diâmetro da broca e pelo material da placa) ou pode ser clicado e escrito.
* As **definições**: o tema, o idioma, o tamanho útil da mesa, a velocidade da simulação (um trabalho demora o que a máquina demoraria, de x0,5 a x50), se o trabalho pergunta nas trocas de ferramenta e no arranque do fuso, e se esta janela abre diretamente da próxima vez. Os botões acendem-se sob o ponteiro e afundam-se ao premir.
* **Pontes no exterior do painel:** na janela Painelização o exterior pode ser cortado com pontes que seguram as placas até serem partidas: quantas por lado, de que largura e onde (repartidas, perto das extremidades ou juntas ao centro); a vista do plano mostra os percursos cortados. O fabricante, a espessura e o tamanho da placa também se escolhem em listas pendentes.
* **Calibração da mesa:** um separador da janela Definições mede a altura da mesa numa grelha de pontos (7 por 5 por omissão), desenha o mapa (azul em baixo, vermelho em cima) com a diferença e a inclinação em mm por metro, e guarda-o com a máquina. Enquanto o nó apalpador não estiver ligado, o apalpador é simulado e a correção dos trabalhos ainda não é aplicada.
* **O trabalho continua depois de uma falha:** enquanto um trabalho corre, o que precisa para continuar (o ficheiro, o painel, os furos feitos) é guardado em tempo real; se a janela, o computador ou a ligação falhar, o arranque seguinte pergunta se continua por onde ia ou se o cancela. A janela abre maximizada e os botões do trabalho são brilhantes (verde para iniciar, azul para pausar, vermelho para parar).
* **Avisos falados:** além dos tons, o fim do trabalho e o pedido de ferramenta podem ser ditos por uma voz, no idioma e no género escolhidos na mesma lista de sons (`sound_finish = voice:es:f`). As vozes da pasta `voices/` são sintéticas (Piper) e de uso livre, com uma frase lenta e formal; não há voz japonesa, nem voz feminina chinesa ou portuguesa, nem gravação real de uma pessoa: ponha os seus próprios ficheiros WAV nas mesmas pastas e eles substituem-nas. Veja `voices/README.txt`.
* **Um catálogo de ferramentas:** uma janela (a partir da das ferramentas) lista mais de 450 ferramentas com a haste de 1/8 de polegada (3,175 mm) do mandril de uma Bungard, em doze tipos: as brocas desde a série micro de 0,05 mm até 7 mm (pequenas, de gama média e grandes com quebra-aparas), brocas para ranhuras, fresas de diamante, de corte descendente, ascendente, quebra-aparas e com revestimento de diamante, fresas retas, fresas em V, de ponta esférica, brocas de centrar e escareadores. Cada uma tem a sua série, o comprimento do seu gume e as rotações, os avanços e a profundidade de passagem da tabela do fabricante para FR-4; um clique desenha a ferramenta (com as suas medidas) e diz para que serve, e um fuso que roda menos do que a tabela (24 000 rotações por defeito, mudam-se na janela) recebe a mesma ferramenta com os avanços reduzidos em proporção. Uma fresa em V indica a largura do seu sulco a uma profundidade. A placa de exemplo traz o isolamento de quatro pistas, cortado com uma fresa em V (ferramenta 3 da tabela de fresagem).
* **Os trabalhos seguem a mesa, e o nó da cabeça de furação:** uma caixa do separador de calibração faz com que a furação e a fresagem corrijam o seu Z com o mapa da mesa (só um mapa completo, só quando pedido); uma janela (a partir da de Ligação) define como se chega ao EH3D-NODE-BUNGARD-AUX (sem usar, direto ou pelo servidor), o seu endereço e token e se a sua câmara é mostrada, e mostra a imagem da câmara, a velocidade e a temperatura do fuso, a saída de proteção e uma medição da mesa sob a cabeça. Pelo servidor consulta o nó através do servidor, com o endereço, o token e o id da máquina da janela de ligação (o servidor guarda o token do nó); o apalpador da calibração continua simulado.
* **Mais fácil de ler e de mover:** a lista de ferramentas do cartão do trabalho desloca-se em vez de empurrar as camadas para fora da janela, todas as listas pendentes abertas têm uma barra que funciona (arrastar, clicar, roda), um botão da vista 2D move só o desenho sobre a placa e a mesa, cada ferramenta de fresagem tem a sua própria cor nas vistas 2D e 3D (um ficheiro com três fresas mostra três cores), e as linguetas são cortadas no contorno de uma placa mesmo com uma só cópia, e só no contorno, nunca nas cavidades e pistas do interior.
* **Animações discretas:** os ecrãs e as janelas aparecem com um fundido e, enquanto um trabalho corre, o anel do cabeçote respira e emite uma onda, os tracejados dos trajetos ainda por cortar avançam sobre eles, uma luz atravessa a barra de progresso e a ferramenta que trabalha respira na lista. Em repouso nada se mexe.

## 🧩 Papel no EH3D

O EH3D-BUNGARD-GUI é a interface moderna das máquinas Bungard na família EH3D. Depende por caminho das pastas irmãs EH3D-COMMON (a cena, os temas e os textos em oito idiomas do `eh3d-view`, e o leitor de ZIP do `eh3d-archive`), EH3D-DRILLPRO e EH3D-ROUTEPRO (os seus motores e os seus programas clássicos): têm de estar junto a esta.

## 🏛️ Arquitetura

<p align="center">
  <img src="images/architecture.svg" alt="Arquitetura de EH3D-BUNGARD-GUI: quem o usa, o que contém e de que depende" width="100%">
</p>

Como as partes deste repositório se encaixam: quem o usa, o que contém e de que depende. A imagem é desenhada a partir de `docs/architecture.json` e verificada com o código.

## 🖼️ Capturas de ecrã

<p align="center">
  <img src="images/screenshots/viewer.png" alt="viewer" width="100%">
</p>

<p align="center"><em>O visualizador: o trabalho na mesa em 2D, a máquina em 3D e os comandos do trabalho.</em></p>

<p align="center">
  <img src="images/screenshots/job-running.png" alt="job running" width="100%">
</p>

<p align="center"><em>Um trabalho em curso: os furos e trajetos feitos, a posição da ferramenta, Pausa e Stop.</em></p>

<p align="center">
  <img src="images/screenshots/neon-theme.png" alt="Neon Arcade theme" width="100%">
</p>

<p align="center"><em>O tema Neon Arcade com a placa em painel.</em></p>

<p align="center">
  <img src="images/screenshots/tools.png" alt="tools" width="100%">
</p>

<p align="center"><em>As ferramentas: avanços, velocidades e profundidades, os furos de cada broca na placa e os feitos no total.</em></p>

<p align="center">
  <img src="images/screenshots/machine-settings.png" alt="settings" width="100%">
</p>

<p align="center"><em>As definições: cada modelo Bungard, a mesa e as unidades dos ficheiros.</em></p>

<p align="center">
  <img src="images/screenshots/board.png" alt="board" width="100%">
</p>

<p align="center"><em>A placa: um material da biblioteca, a sua espessura, o seu cobre e o seu tamanho.</em></p>

<p align="center">
  <img src="images/screenshots/themes.png" alt="themes" width="100%">
</p>

<p align="center"><em>Onze temas na janela de definições, cada um com o nome no idioma da janela.</em></p>

<p align="center">
  <img src="images/screenshots/sounds.png" alt="sounds" width="100%">
</p>

<p align="center"><em>A lista de sons do fim do trabalho: sessenta sons dos agudos aos graves, o tom padrão, nenhum ou um ficheiro próprio.</em></p>

<p align="center">
  <img src="images/screenshots/panel.png" alt="panel" width="100%">
</p>

<p align="center"><em>A janela Painelização: as cópias da placa e as pontes do exterior, escolhidas com listas pendentes e contadores.</em></p>

<p align="center">
  <img src="images/screenshots/calibration.png" alt="calibration" width="100%">
</p>

<p align="center"><em>A calibração da mesa: o mapa de alturas medido numa grelha de pontos, com a diferença e a inclinação.</em></p>

<p align="center">
  <img src="images/screenshots/paper-theme.png" alt="light theme" width="100%">
</p>

<p align="center"><em>O tema Papel, um claro.</em></p>

<p align="center">
  <img src="images/screenshots/fleet.png" alt="machines and stations" width="100%">
</p>

<p align="center"><em>As máquinas e estações do servidor: estado, trabalho e progresso de cada máquina, e a máquina para onde vão os trabalhos.</em></p>

<p align="center">
  <img src="images/screenshots/head-node.png" alt="nó da cabeça de furação" width="100%">
</p>

<p align="center"><em>O nó da cabeça de furação: a sua câmara, a velocidade e a temperatura do fuso, a saída de proteção e o anel de LEDs à volta da câmara (desligado e três níveis).</em></p>
<p align="center">
  <img src="images/screenshots/quality-check.png" alt="verificação de qualidade" width="100%">
</p>

<p align="center"><em>A verificação de qualidade depois do trabalho: a câmara olha para a placa, cada furo do trabalho fica marcado (verde certo, vermelho errado, âmbar fora da imagem), medem-se os canais fresados e o veredicto é grande como um cartaz.</em></p>
<p align="center">
  <img src="images/screenshots/quality-tolerances.png" alt="tolerâncias" width="100%">
</p>

<p align="center"><em>As tolerâncias do analisador da oficina: tamanho e posição dos furos, redondeza, rebarbas, largura e regularidade dos canais, e o que torna uma foto boa o bastante para julgar.</em></p>

## ✨ Funcionalidades

* Temas: Forge (o predefinido), Neon Arcade, Glass Pro, o cinzento clássico do Windows e um que segue as cores do Windows.
* Oito idiomas em toda a janela: inglês, espanhol, francês, italiano, alemão, chinês, japonês e português de Portugal.
* Perfis de máquina com as medidas publicadas e uma mesa ajustável: a máquina em 3D cresce com a área útil da configuração.
* Uma janela das máquinas e estações: com a ligação ao servidor mostra todas as máquinas do servidor agrupadas por estação, o que cada uma faz, o seu trabalho com o progresso, os trabalhos prontos, e escolhe a máquina para onde vão os trabalhos da janela; `--fleet FICHEIRO.json` mostra uma resposta guardada sem servidor.
* Escolher um modelo de máquina nas definições ajusta as duas vistas à sua mesa: a planta 2D, a máquina 3D (estrutura, calhas, pórtico e armazém de ferramentas desse modelo) e o zoom recomeçam com as novas medidas.
* Trabalhos em arquivos ZIP: mostram-se o primeiro ficheiro de furação e o primeiro de fresagem, e os restantes são listados para carregar com um clique.
* Panelização em três modos (espaço, passo, tantas quantas caibam) com as cópias desenhadas na mesa.
* Progresso do trabalho em direto e as perguntas da máquina ao operador.
* Uma janela Acerca de com a versão, a licença e a ligação em uso.
* Definições em separadores com todos os modelos Bungard MTC e ATC (mesa, trocador de ferramentas, limites) e as unidades dos passos e das coordenadas (mm ou mil), tudo com efeito real no trabalho.
* Um botão de emergência que se pode mostrar, esconder e arrastar; o Start apaga-se enquanto o trabalho corre e volta com Pausa ou Stop; a simulação continua em segundo plano.
* Uma vista 2D que segue cada ferramenta, roda em conjunto os ficheiros de furação e de fresagem e move a placa; uma biblioteca com mais de cem materiais de base; sons no fim do trabalho e na troca de ferramenta.
* Ficheiros de outros programas: o contorno de uma placa Gerber (`.GKO`, `.GM1`, `.GML` ou um `.GBR` que se declara perfil) e um desenho DXF abrem-se como trabalhos de fresagem; os círculos e arcos dos ficheiros de traçado (`CI`, `AA`, `AR`) são cortados em cordas pelo ângulo de corda da configuração.
* Um trabalho que não cabe na mesa é recusado antes de começar; a janela Abrir lista os últimos ficheiros abertos; a janela das ferramentas conta os furos de cada broca na placa e em todos os trabalhos, com um botão que repõe os contadores a zero.
* A janela de ferramentas abre-se sobre um ficheiro que abre (pela janela Abrir, pelos recentes ou largando-o), como os programas clássicos mostram as ferramentas de um ficheiro depois de o carregar; uma caixa nas definições desliga-a.
* Listas pendentes onde uma escolha tem muitos valores: o tema, o idioma, o modelo da máquina e o som de cada evento escolhem-se numa lista que desliza, em vez de filas de botões. O fim do trabalho e o pedido de ferramenta têm cada um a sua lista de 60 sons, dos agudos aos graves (carrilhões, notas a subir e a descer, fanfarras, apitos, alarmes, pancadas, sirenes), o tom padrão, nenhum ou um ficheiro próprio; ao escolher um, ele toca.
* **Luz e verificação de qualidade com a câmara da cabeça:** a janela do nó liga e desliga o anel de LEDs, e a janela Verificação de qualidade olha para a placa depois do trabalho: julga os furos do trabalho (existem, redondos, do tamanho certo, no sítio), os canais fresados (tão largos como a ferramenta, regulares, sem estreitamentos) e a própria foto, marca cada furo na imagem, mostra um veredicto grande como um cartaz (APROVADO, REPROVADO ou não avaliável) e faz o anel dizer o mesmo a cores. Os perfis (padrão, passo fino, protótipo), as tolerâncias, a escala e a sua calibração com um furo conhecido guardam-se num ficheiro e entregam-se ao servidor. O resultado também pode ser dito em voz alta: mais dois sons nas definições, para uma verificação correta e outra que não o é, cada um desligado, um apito, um som do catálogo, um ficheiro teu ou uma voz que diz que a verificação visual da placa está correta ou não está correta, como os outros sons.
* **Uma máquina em 3D que se comporta como a real:** em repouso a cabeça espera em home, e mover a placa na vista em planta nunca a move. Ao começar um trabalho a máquina arranca, faz home, viaja até ao primeiro ponto do trabalho e só então o trabalho começa, com a cabeça a segui-lo, em furação e em fresagem; no fim volta a home. As barras das listas têm as suas setas para cima e para baixo.
* **Dois fusos e um catálogo que os conhece:** a máquina leva o fuso original (60000 rpm) ou o que tem variador de frequência (24000 rpm), escolhido nas definições, cada um com os seus limites (rotação, avanços XY e Z, ferramenta mais grossa) guardados no ficheiro de definições. Na janela das ferramentas cada ferramenta tem um botão Catálogo: escolhe-se uma broca ou fresa e as suas velocidades, avanços e passagens passam para a ferramenta. Se o fuso não os puder dar, a janela diz: sugere os valores que pode usar (a mesma mordida por volta, XY ou Z mais lentos) ou marca a ferramenta como incompatível e não deixa tomá-la. As listas de placas mostram 16 materiais de cada vez.
* **Furação, fresagem ou ambas, ferramenta a ferramenta:** a furação e a fresagem do trabalho ativam-se ou desativam-se no painel esquerdo e na janela das ferramentas (com as duas: primeiro a furação e depois a fresagem, uma a seguir à outra, sem a máquina voltar a home pelo meio), e o ponto de cada ferramenta desativa-a para o trabalho a deixar de fora.
* **Vários ficheiros de uma vez e exemplos profissionais:** a janela Abrir escolhe dois ou mais ficheiros com Ctrl+clique ou Shift+clique (furação e fresagem juntas; vários do mesmo tipo formam um só trabalho, com as ferramentas numeradas pelo diâmetro). A pasta `examples` do pacote traz quatro desenhos de complexidade real, cada um com o seu ficheiro de furação e de fresagem: uma placa de microcontrolador de 32 bits (barras de pinos, entalhe USB, anel de vias do LQFP), um andar de entrada analógico de quatro canais (amplificadores DIP, BNC e bornes, anéis de guarda, uma ranhura de isolamento), um amplificador e filtro de 2,4 GHz (SMA de borda, cercas de vias, costura da blindagem, fresagem de isolamento ao longo da linha de 50 ohms) e uma fonte de 5 V 3 A (TO-220, eletrolíticos, vias térmicas).

## 🔌 Interfaces

* **Janela:** o lançador e o visualizador, com o rato e o teclado (`Enter` aceita a escolha do lançador, `Esc` volta atrás, `Ctrl+O` abre ficheiros).
* **Linha de comandos:** `bungard-gui [FICHEIRO...] [--drill FICHEIRO.NCD] [--route FICHEIRO.PLT] [--zip TRABALHO.ZIP] [--lang CÓDIGO] [--theme NOME] [--data PASTA] [--connection FICHEIRO] [--screen launcher|viewer]`; os ficheiros sem opção abrem-se como diz a sua extensão. `--modal`, `--panel`, `--yaw`, `--shot` e `--start` definem uma janela, o painel, a rotação da vista 3D, uma imagem da janela e o arranque do trabalho, para as imagens.
* **Máquina:** através dos motores do EH3D-DRILLPRO e do EH3D-ROUTEPRO e do ficheiro de máquina que partilham: o RS-232 do computador, um adaptador USB-RS232, o EH3D-NODE-BUNGARD ou o EH3D-SERVER. Sem ficheiro de máquina o trabalho corre no simulador.

## ⚙️ Configuração

A ligação é o **ficheiro da máquina** do EH3D-DRILLPRO e do EH3D-ROUTEPRO (`machine.cfg`, com `connection = demo|rs232|usb|node|server` e cada valor afinado numa máquina real): `--connection FICHEIRO` aponta para ele, e sem um o trabalho corre no simulador. `--data PASTA` é a pasta com os ficheiros de máquina e ferramentas dos programas clássicos (`DRILLPRO.DCF`, `DRILLPRO.DTD`, `ROUTEPRO.RCF`, `ROUTEPRO.RTD`; a pasta `testdata/` tem exemplos). A janela de definições altera a largura e a profundidade úteis da mesa e guarda-as nesses ficheiros. O que a janela memoriza (a escolha do lançador, o idioma e o tema) fica num pequeno ficheiro na pasta de definições do utilizador, com os outros programas EH3D. `--theme forge|neon|glass|windows-classic|system` e `--lang en|es|it|de|fr|pt|zh|ja` escolhem o aspeto e o idioma.

## 🚀 Início rápido

```bash
cargo build --release --features gui
cargo test --features gui
cargo clippy --all-targets --features gui -- -D warnings
cargo fmt --all --check
python -I tools/ci_validate.py
```

Execute-o com `run.bat` (Windows) ou `./run.sh` (Linux e macOS); ambos compilam primeiro a janela se for preciso e, sem argumentos, abrem o lançador com a placa de teste de `testdata/`. Precisam de Rust e das pastas irmãs `EH3D-COMMON`, `EH3D-DRILLPRO` e `EH3D-ROUTEPRO`. `cargo run --release --features gui -- --zip TRABALHO.ZIP` abre um trabalho empacotado num arquivo ZIP. No Linux a janela precisa de X11 ou Wayland.

## 🧪 Testes

A bateria tem 117 testes (`cargo test --features gui`). Verificam o lançador e o que memoriza, as definições, como um ficheiro de furação ou HP-GL se torna a cena, os perfis de máquina, o painel e a linha de comandos. Os motores que usa (protocolo, movimento, porta de segurança) são testados no EH3D-DRILLPRO e no EH3D-ROUTEPRO. A CI também executa `cargo fmt --check` e `cargo clippy` com os avisos como erros, e `python -I tools/ci_validate.py` verifica o manifesto, as versões, o registo de alterações e os oito README.

## 🏗️ Compilação e publicação

A versão está em `Cargo.toml` e em `eh3d.project.json`, e cada versão tem a sua entrada em [CHANGELOG.md](CHANGELOG.md). `build.bat` / `build.sh` compilam a janela (`cargo build --release --features gui`) e `test.bat` / `test.sh` executam os testes; nenhum altera a versão.

**Pacotes já compilados:** o fluxo `Release binaries` deste repositório compila o programa para Windows (64 e 32 bits), Linux e macOS (Apple silicon e Intel), cada pacote com a licença, este README e os ficheiros de teste como exemplos, e o seu SHA-256. Ainda não foi publicada nenhuma versão.

## 📂 Estrutura do repositório

<p align="center">
  <img src="images/repository-structure.svg" alt="Estrutura do repositório EH3D-BUNGARD-GUI" width="100%">
</p>

<details>
<summary>Versão em texto da árvore</summary>

```text
EH3D-BUNGARD-GUI/
├── src/
│   ├── gui.rs                                   a janela: lançador, vista em planta, vista 3D, progresso do trabalho
│   ├── gui/extra.rs                             abrir ficheiros, panelização, definições da mesa, Acerca de, mover as vistas
│   ├── gui/fleet.rs                             janela das máquinas e estações do servidor
│   ├── inspection.rs gui/inspect.rs             a verificação de qualidade do trabalho com a câmara
│   ├── gui/machine3d.rs                         a máquina da vista 3D: arranque, home, primeiro ponto, trabalho
│   ├── gui/opener.rs  gui/board.rs              janelas Abrir, placa, ferramentas e definições
│   ├── jobs.rs                                  um ficheiro de furação ou HP-GL convertido na cena que as vistas desenham
│   ├── boards.rs                                a biblioteca de materiais de base
│   ├── machines.rs                              os perfis das máquinas Bungard e a área de trabalho a partir da mesa
│   ├── launcher.rs                              as opções do lançador, os programas clássicos e o que fica memorizado
│   ├── link.rs                                  a ligação do ficheiro da máquina: que chaves escreve
│   ├── cli.rs                                   a linha de comandos
│   ├── lib.rs bin/                              a raiz da biblioteca e o executável bungard-gui
│   └── spindle.rs                               os dois fusos, os seus limites e se uma ferramenta serve
├── testdata/                                    uma placa pequena: ficheiros de furação e contorno e os ficheiros de exemplo de máquina e ferramentas
├── docs/                                        as fontes das duas imagens
├── tools/                                       verificação de CI, verificação do README e ferramentas das imagens
├── images/                                      faixa, as duas imagens e as capturas de ecrã
├── voices/                                      avisos falados
├── .github/                                     fluxos de CI e de publicação, modelos de issues e pull requests
├── build.sh build.bat                           compilam a janela
├── run.sh run.bat                               arrancam a janela, com os ficheiros de teste se não houver argumentos
├── eh3d.project.json bump_manifest_version.py   manifesto e utilitário de versão
├── Cargo.toml Cargo.lock                        o pacote Rust e as suas dependências fixadas
├── CHANGELOG.md                                 o que mudou em cada versão
├── SECURITY.md SUPPORT.md                       como comunicar uma vulnerabilidade e onde perguntar
└── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   como contribuir, o código de conduta e a licença
```

</details>

## 🗺️ Roteiro e limites conhecidos

**Limites conhecidos hoje**

* Nenhum trabalho foi comandado a partir desta janela numa máquina real: só correu no simulador dos motores do EH3D-DRILLPRO e do EH3D-ROUTEPRO.
* A janela de Ligação escreve no ficheiro da máquina apenas as chaves da ligação; a de definições escreve as chaves que oferece (a mesa, o modelo, as unidades, os sons e a velocidade da simulação) e a de ferramentas os avanços, as velocidades e as profundidades; o resto dos ficheiros de máquina e ferramentas não se edita aqui.
* A pausa retém o trabalho entre dois movimentos, não a meio de um, e a vista 3D não mostra a altura dos trajetos.
* Só se comandam as máquinas CCD e CCD/2; os perfis da CCD Premium e da CCD XL são desenhados, mas a sua própria unidade de controlo não é suportada.
* A janela precisa das pastas irmãs EH3D-COMMON, EH3D-DRILLPRO e EH3D-ROUTEPRO junto a si, porque depende delas por caminho.

**A seguir**

* Uma pausa a meio de um movimento, não só entre dois movimentos.
* A altura (Z) dos trajetos na vista 3D.
* Carregar e guardar toda a configuração, não apenas o tamanho da mesa.
* Empacotar com os programas clássicos, para que uma só transferência tenha tudo.
* Um teste numa máquina real.

## 🌍 Idiomas

O README e os textos existem em inglês, espanhol, francês, italiano, alemão, chinês, japonês e português europeu.

## 🔗 Projetos relacionados

* [EH3D-DRILLPRO](https://github.com/JuanenRac/EH3D-DRILLPRO): o programa de furação; esta janela usa o seu motor e o seu programa clássico
* [EH3D-ROUTEPRO](https://github.com/JuanenRac/EH3D-ROUTEPRO): o programa de fresagem; esta janela usa o seu motor e o seu programa clássico
* [EH3D-COMMON](https://github.com/JuanenRac/EH3D-COMMON): a vista (cena, temas, textos em oito idiomas) e o leitor de ZIP que esta janela usa
* [EH3D-NODE-BUNGARD](https://github.com/JuanenRac/EH3D-NODE-BUNGARD): um nó ESP32-S3 que fica na caixa da máquina e envia os passos pela sua própria porta série
* [EH3D-SERVER](https://github.com/JuanenRac/EH3D-SERVER): o centro da família EH3D: pode executar o trabalho na máquina através do seu nó

## 📚 Documentação

* [Ligação e afinação](https://github.com/JuanenRac/EH3D-DRILLPRO/blob/main/docs/CONNECTION.md): o ficheiro da máquina, as ligações e cada valor (EH3D-DRILLPRO)
* [Notas do protocolo](https://github.com/JuanenRac/EH3D-DRILLPRO/blob/main/docs/PROTOCOL.md): o que se sabe sobre como a unidade de controlo é comandada (EH3D-DRILLPRO)
* [Registo de alterações](CHANGELOG.md)

## 🤝 Contribuir e comunidade

* Antes de um pull request execute `cargo fmt --all --check`, `cargo clippy --all-targets --features gui -- -D warnings`, `cargo test --features gui` e `python -I tools/ci_validate.py`: tudo tem de passar. Consulte [CONTRIBUTING.md](CONTRIBUTING.md) e o [código de conduta](CODE_OF_CONDUCT.md).
* O README existe em oito idiomas: uma alteração ao texto faz-se nos oito ficheiros, e as duas imagens são desenhadas a partir de `docs/architecture.json` e `docs/structure.json` com `tools/make_architecture.py` e `tools/make_structure.py`.
* Não descreva como a funcionar numa máquina o que só foi experimentado no simulador.
* Comunique uma vulnerabilidade em privado: [SECURITY.md](SECURITY.md). Perguntas, ideias e relatórios: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTOR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENÇA

Software: GPL-3.0-or-later. Ver [LICENSE](LICENSE). Os projetos de hardware da suite usam a CERN OHL-S.
