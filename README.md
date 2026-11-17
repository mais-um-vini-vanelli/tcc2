# tcc2

Rasterizador e ray tracer em Rust, com um editor de cenas em Godot e as docs da teoria dos
algoritmos implementados.

## Para rodar

### O repositório

```powershell
git clone <repo> tcc2
```

**Todos os comandos abaixo rodam da raiz dele**: os exemplos procuram `examples/assets/` a
partir do diretório atual.

### Docs

Publicadas em `https://mais-um-vini-vanelli.github.io/tcc2/`.

### O pacote

As texturas, os modelos e os exemplos já compilados não são versionados. Baixe
`exemplos.zip` do release mais recente (aba *Releases*) e descompacte **na raiz do
repositório**:

```powershell
Expand-Archive exemplos.zip -DestinationPath .
```

### Editor

Baixe `editor.zip` do mesmo release, descompacte em qualquer lugar e rode o `editor.exe`.
Se a janela abrir preta, force o Vulkan:

```powershell
.\editor.exe --rendering-driver vulkan
```

O editor monta e salva cenas no formato `.tcc2`.

### Renderizar cena sem instalar Rust: `viewer.exe`

É um executável que abre uma cena salva pelo editor e renderiza com a
biblioteca. Rode **da raiz do repositório**, passando a cena que você salvou:

```powershell
.\target\release\examples\viewer.exe {minha_cena.tcc2} --renderer rasterizer
```

Ou para renderizar por ray tracing:

```powershell
.\target\release\examples\viewer.exe {minha_cena.tcc2} --renderer raytracer
```

O título da janela diz qual dos dois gerou a imagem. `Esc` fecha a janela.

### Outros exemplos, sem instalar nada

Os binários já compilados estão em `target/release/examples/`, e rodam do mesmo jeito que o
`viewer`:

| Exemplo | O que faz |
| --- | --- |
| `aliasing` | Janela: textura em xadrez com e sem mipmap |
| `tonemap` | Janela: comparação de operadores de tonemap |
| `bvh` | PNGs: mapa de calor do custo de travessia da BVH |
| `raster_vs_raytrace` | PNGs: a mesma cena pelos dois renderizadores |
| `normal_maps` | PNGs: normal mapping nas texturas de tijolo e casca |

Os PNGs vão para `examples/renders/`.

### Compilar e rodar a biblioteca localmente

As ferramentas de build da Microsoft primeiro:

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

E depois o Cargo + Rust:

```powershell
winget install Rustlang.Rustup
```

Reabra o terminal, e confirme que funcionou (o projeto usa a edição 2024, que pede 1.85 ou mais
novo):

```powershell
rustc --version
```

Rodando um exemplo:

```powershell
cargo run --release --example aliasing
```

E com parâmetros:

```powershell
cargo run --release --features io --example viewer -- --renderer raytracer
```

## Atividades

### 1. Montar e renderizar uma cena

O `editor.exe` já abre com uma cena simples: o nó raiz `Scene`, uma câmera
`Perspective`, uma luz `Directional` e uma esfera. O `Help` no canto inferior direito
mostra os comandos básicos.

O objetivo é criar uma cena com chão, dois objetos, duas luzes e a câmera enquadrando tudo.

1. Clique em `+` na hierarquia (canto superior esquerdo) e escolha `Geometry > Cube`. O cubo é
   criado como filho do que estiver selecionado.
2. Transforme o cubo emm um chão: com ele selecionado, arraste o gizmo que é um quadrado pra esticar/achatar ele. Também pode mover e girar ele com os gizmos.
4. Adicione `Geometry > Sphere` e `Light > Point` na raiz, e posicione eles na cena.
5. Selecione a luz e ajuste `color` e `intensity` nas properties (canto inferior esquerdo). Altere o raio da esfera pelas properties também.
6. Selecione a esfera e adicione `Geometry > Cube` com ela selecionada, para o cubo
   ficar como filho dela. Gire e escale a esfera pra ver que o filho é alterado também.
7. Olhe a preview de como a cena vai ficar renderizada: selecione o botão de `Camera POV` no overlay (barra do canto superior direito).
8. Salve a cena com com o botão `Save` do overlay.
9. Renderize a cena salva, passando o arquivo que você salvou no passo anterior:

```powershell
.\target\release\examples\viewer.exe {minha_cena.tcc2} --renderer rasterizer
```

Ou renderize por ray tracing (faz mais diferença em cenas com transparência e reflexão):

```powershell
.\target\release\examples\viewer.exe {minha_cena.tcc2} --renderer raytracer
```

Se quiser desenhar cenas mais elaboradas, pode arrastar os modelos de examples\assets\models pro editor pra importar eles.

### 2. Parâmetros de cena

Os parâmetros do render ficam no nó raiz: selecione `Scene` na hierarquia e olhe as
properties, separadas em `RASTERIZATION`, `SHADING`, `RAY TRACING` e `OUTPUT`.

O objetivo é ver o que os parâmetros mudam na imagem e no custo do render. Altere eles, 
salve e renderize:

```powershell
.\target\release\examples\viewer.exe {minha_cena.tcc2} --renderer rasterizer
```

Além da imagem, o viewer imprime as estatísticas no terminal e grava um PNG por buffer em
`examples/renders/`: `color`, `depth`, `normal` e `uv` no rasterizador, `color`, `ray_count`
e `bvh_heatmap` no ray tracer.

1. Desligue `backface cull`: o interior dos objetos aparece, e `backfaces_culled` zera no
   terminal.
2. Desligue `depth test`: os objetos se atravessam, porque quem é desenhado por último ganha
   o pixel.
3. Mude `acceleration` entre `brute_force`, `bvh_median` e `bvh_sah` e compare
   `primitive_tests`, `bvh_nodes_visited` e `bvh_build_time` no terminal. A imagem é a mesma.

### 3. Parâmetros de material

Selecione um objeto com geometria: a seção `Material` das properties tem um canal pra
`Albedo`, `Metallic`, `Roughness` e `Emissive`, cada um com um `Kind` que escolhe entre
`Constant` (um valor) e `Map` (uma textura), mais `Normal`, `Transparency` e `Refractive
index`.

O objetivo é ver o que cada canal faz muda na renderização do objeto. Como na atividade anterior, altere eles, salve e renderize.

1. Mude `Roughness` de 0 a 1: o brilho especular abre e perde intensidade.
2. Mude `Metallic` de 0 a 1: o difuso some e o especular assume a cor do `Albedo`.
3. Coloque uma cor para `Emissive` e aumente `Strength`: o objeto brilha sem depender de luz.
4. Troque o `Kind` do `Albedo` pra `Map` e escolha uma textura.
5. Aumente `Transparency` e mude `Refractive index` entre 1.0, 1.5 (vidro) e 2.4 (diamante).
   Renderize dos dois modos: reflexo e refração só existem no ray tracer.

### 4. As previews do editor

O objetivo é ver algumas partes das pipelines, antes de ver no código.

1. Ligue `Pick Triangle`, passe o mouse sobre um objeto (o triângulo sob o cursor acende) e
   clique nele. O resto da cena some, e o triângulo escolhido aparece em cada etapa:
   local (vermelho tracejado), mundo (branco tracejado), o frustum da câmera (amarelo), o
   recorte contra ele (verde), o plano de tela (azul), as linhas de projeção (roxo) e o
   triângulo final em tela (lilás).
2. Ligue `Trace Ray`: o viewport vai pro ponto de vista da câmera de render e uma lupa
   mostra os pixels ampliados. Clique em um pixel pra ver os raios que saem dele: o primário
   (branco), os de sombra (amarelo), os de reflexão (azul) e os de refração (lilás), com as
   normais dos acertos em verde. Fica mais legal em objetos com reflexão (`Roughness` < 0.1
   e `Metallic` alto) ou refração (`Transparency` alta), que é onde os raios secundários
   aparecem.
3. Ligue `BVH`: as caixas da árvore aparecem, uma cor por nível, e dá de usar `+` e `-` para mostrar mais ou
   menos níveis. Troque o `acceleration` entre `bvh_median` e `bvh_sah` pra ver a diferença
   na árvore.
   
## Publicando uma nova versão

### Pré-requisitos

- Rust estável (ver [Compilar e rodar a biblioteca localmente](#compilar-e-rodar-a-biblioteca-localmente)).

### 1. Assets (uma vez só)

As texturas e os modelos não são versionados, e o workflow dos exemplos os pega de um release
fixo na tag `assets`:

```powershell
tar -a -c -f assets.zip examples/assets
```

Crie o release na tag `assets` e anexe o zip. Depois, trocar cena ou textura é substituir
esse arquivo.

### 2. Docs

`.github/workflows/docs.yml` compila e publica no Pages. Rode em *Actions → docs → Run
workflow*.

### 3. O release

Crie o release da versão no GitHub, com a tag (`v1.0.0`) e as notas: os dois workflows
abaixo só anexam arquivos nele, nenhum cria tag.

### 4. Exemplos

`.github/workflows/examples.yml` compila no Windows, baixa os assets, e anexa o
`exemplos.zip` no release. Rode em *Actions → examples → Run workflow* e informe a versão.

### 5. Editor

`.github/workflows/editor.yml` compila a dll, exporta com o preset `Windows Desktop` e
anexa o `editor.zip` no mesmo release. Rode em *Actions → editor → Run workflow*.

Na primeira rodada ele baixa os *export templates* (~1,3 GB) e demora bem mais que os
outros; depois eles vêm do cache, até a versão do Godot mudar.
