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

As cenas, as texturas e os exemplos já compilados não são versionados. Baixe
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

O editor monta e salva cenas no formato `.tcc2`. Algumas cenas de exemplo estão em
`examples/assets/*.tcc2`, dentro do pacote.

### Renderizar cena sem instalar Rust: `viewer.exe`

É um executável que abre uma cena salva pelo editor e renderiza com a
biblioteca. Rode **da raiz do repositório**:

```powershell
.\target\release\examples\viewer.exe examples\assets\Scene.tcc2 --renderer rasterizer
```

Ou para renderizar por ray tracing:

```powershell
.\target\release\examples\viewer.exe examples\assets\Scene.tcc2 --renderer raytracer
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

## Publicando uma nova versão

### Pré-requisitos

- Rust estável (ver [Compilar e rodar a biblioteca localmente](#compilar-e-rodar-a-biblioteca-localmente)).

### 1. Assets (uma vez só)

As cenas e as texturas não são versionadas, e o workflow dos exemplos as pega de um release
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
