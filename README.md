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

Publicadas em `https://<projeto>.pages.dev`.

### O pacote

As cenas, as texturas e os exemplos já compilados não são versionados. Baixe
`exemplos.zip` de `https://<dominio-r2>/exemplos.zip` e descompacte na raiz do
repositório, que é onde os caminhos de dentro do zip se encaixam:

```powershell
Expand-Archive exemplos.zip -DestinationPath .
```

Aparecem `examples/assets/` e `target/release/examples/*.exe`.

### Editor

Baixe `editor.zip`, descompacte em qualquer lugar e rode o `editor.exe`. Se a
janela abrir preta, force o Vulkan:

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
- Node 18+, para as docs.
- Godot 4.6 com os *export templates* da mesma versão
  (Editor → Gerenciar Modelos de Exportação → Baixar e Instalar).

### 1. Editor

Compile a dll:

```powershell
cargo build --release -p tcc2_godot
```

Abra `editor/project.godot` e use Projeto → Exportar → Exportar Projeto, com o preset
`Windows Desktop`.

Confirme que a pasta tem `editor.exe`, `editor.pck` e `tcc2_godot.dll`.

### 2. Docs

Da raiz do repositório:

```powershell
npm run build --prefix docs
```

O site estático sai em `docs/.vitepress/dist`.

### 3. Pacote dos exemplos

Compile os exemplos:

```powershell
cargo build --release --features io --examples
```

Gerar exemplo.zip com os arquivos não versionados:

```powershell
tar -a -c -f exemplos.zip examples/assets target/release/examples/*.exe
```

### 4. Upload

- `docs/.vitepress/dist` → Cloudflare Pages, *Create project* → *Direct upload*: arraste a
  pasta (nas atualizações, arraste de novo).
- `exemplos.zip` e `editor.zip` → bucket do R2, com *Public Development URL* habilitada.
