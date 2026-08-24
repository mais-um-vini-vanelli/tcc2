# Roadmap

Ordem macro segue a metodologia já registrada no TCC1: protótipo de
rasterização antes do protótipo de ray tracing. A única inserção em relação a
um roadmap puramente sequencial é o M0.5 — um esqueleto fino nos dois modos,
logo após o núcleo compartilhado, pra validar cedo que a mesma `Scene`/trait
`Renderer` (ver [design_api.md](design_api.md)) serve pros dois pipelines
antes de investir semanas aprofundando só um deles.

## M0 — Núcleo compartilhado

- `Vec3`/`Vec2` (posição, direção, uv), `Rgb` em float
- `Camera` (`Perspective`, `Orthographic`)
- `Light` (`Point`, `Directional`, `Ambient`)
- `Primitive` (`Triangle`, `Mesh`, `Sphere`) com `Material` embutido
- `Parameters` agrupado por estágio (raster, clipping, texturing, acceleration, ray tracing, output)
- trait `Renderer` + `RenderResult`/`RenderPass` (mapa de buffers)
- saída em PPM/PNG (sem lógica de shading ainda)

## M0.5 — Esqueleto fino nos dois modos

- `Rasterizer`: um triângulo, sem clipping, sem shading, cor sólida
- `RayTracer`: uma esfera, sem sombra, sem reflexo, cor sólida
- os dois lendo a mesma `Scene`, sem atalhos específicos de modo
- checar se algum campo da `Scene`/`Parameters` só faz sentido pra um dos dois (sinal de que o design precisa ajustar)

## M1 — Rasterização a fundo

- edge functions (Pineda) com bounding box por triângulo
- clipping de polígonos (Sutherland-Hodgman) contra o frustum
- z-buffer / teste de profundidade
- coordenadas baricêntricas com correção de perspectiva
- suporte a `Mesh` (não só `Triangle` solto)
- backface culling
- `wireframe` e `msaa` como toggles de `RasterOptions`

## M2 — Texturização

- interpolação de UV por triângulo
- `Texture::from_file` / `Texture::from_bytes`
- geração da pirâmide de mipmaps (Williams, 1983)
- seleção de nível de mip por distância/derivada de tela
- filtros: nearest, bilinear, trilinear

## M3 — Ray tracing a fundo

- geração de raios primários a partir da câmera
- interseção raio-triângulo (Möller–Trumbore ou equivalente)
- interseção raio-esfera (equação algébrica direta)
- modelo recursivo de Whitted: raios de sombra, reflexão, refração
- `max_depth` controlando a recursão
- suporte aos três tipos de `Light` no cálculo de iluminação

## M4 — Aceleração

- construção de BVH linear (LBVH, Apetrei 2014)
- travessia da BVH durante interseção de raios
- toggle `AccelOptions.bvh` (ligado/desligado) pra comparar tempo de render
- `RenderPass::RayCount` e `RenderPass::BvhHeatmap`

## M5 — Recursos cross-pipeline

- `RenderPass::Depth`, `Normal`, `Uv` implementados nos dois modos
- ferramenta de diff de imagem entre `Rasterizer` e `RayTracer` sobre a mesma cena
- cenas de demonstração dos pontos de divergência (sombra, esfera) do item 8 de [diferenciais_possiveis.md](diferenciais_possiveis.md)
- toggles "caixa branca" documentados (efeito visual de cada `Parameters` ligado/desligado)

## M6 — Import e cenas

- parser de `.obj` (geometria: vértices, normais, uv, faces)
- parser de `.mtl` (material + caminho de textura)
- resolução de caminho relativo do `.mtl`/texturas ao diretório do `.obj`
- serialização de `Scene`/`Parameters` via `serde` (RON/JSON)
- pacote de cenas de benchmark (com números de referência: triângulos, tempo por modo)

## M7 — Validação

- aplicar a lib num exercício real de disciplina de Computação Gráfica
- coletar feedback dos alunos/uso
- comparar resultados com literatura/implementações consolidadas
- redigir conclusões do TCC2 a partir dos dados coletados
