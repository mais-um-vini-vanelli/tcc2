# Roadmap

Ordem macro segue a metodologia já registrada no TCC1: protótipo de
rasterização antes do protótipo de ray tracing.

## M0 — Núcleo compartilhado

- [x] `Vec3`/`Vec2` (posição, direção, uv), `Rgb` em float
- [x] `Camera` (`Perspective`, `Orthographic`)
- [x] `Light` (`Point`, `Directional`, `Ambient`)
- [x] `Primitive` (`Triangle`, `Mesh`, `Sphere`) com `Material` embutido
- [x] `Parameters` agrupado por estágio (raster, clipping, texturing, acceleration, ray tracing, output)
- [x] trait `Renderer` + `RenderResult`/`RenderPass` (mapa de buffers)
- [x] saída em PPM/PNG (sem lógica de shading ainda)

## M0.5 — Esqueleto fino nos dois modos

- [x] `Rasterizer`: um triângulo, sem clipping, sem shading, cor sólida
- [x] `RayTracer`: uma esfera, sem sombra, sem reflexo, cor sólida
- [x] os dois lendo a mesma `Scene`, sem atalhos específicos de modo
- [x] checar se algum campo da `Scene`/`Parameters` só faz sentido pra um dos dois (sinal de que o design precisa ajustar)

## M1 — Rasterização a fundo

- [x] edge functions (Pineda) com bounding box por triângulo
- [x] clipping de polígonos (Sutherland-Hodgman) contra o frustum, em espaço de câmera
- [x] z-buffer / teste de profundidade (`RenderPass::Depth` já sai do rasterizador)
- [x] coordenadas baricêntricas com correção de perspectiva
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
