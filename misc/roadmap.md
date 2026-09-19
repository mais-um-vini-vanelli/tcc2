# Roadmap

Ordem macro segue a metodologia já registrada no TCC1: protótipo de
rasterização antes do protótipo de ray tracing.

## M0 — Núcleo compartilhado

- [x] `Vec3`/`Vec2` (posição, direção, uv), `Rgb` em float
- [x] `Camera` (`Perspective`, `Orthographic`)
- [x] `Light` (`Point`, `Directional`, `Ambient`)
- [x] `Primitive` (`Triangle`, `Mesh`, `Sphere`) com `Material` embutido
- [x] `Parameters` agrupado por estágio (raster, clipping, texturing, acceleration, ray tracing, output)
- [x] trait `Renderer` + `RenderResult`/`RenderPass` (um `Option<_>` por passe)
- [x] ~~saída em PPM/PNG~~ — a lib não grava imagem; o exemplo `dois_modos` mostra como escrever PNG a partir dos buffers

## M0.5 — Esqueleto fino nos dois modos

- [x] `Rasterizer`: um triângulo, sem clipping, sem shading, cor sólida
- [x] `RayTracer`: uma esfera, sem sombra, sem reflexo, cor sólida
- [x] os dois lendo a mesma `Scene`, sem atalhos específicos de modo
- [x] checar se algum campo da `Scene`/`Parameters` só faz sentido pra um dos dois (sinal de que o design precisa ajustar)

## M1 — Rasterização a fundo

- [x] edge functions (Pineda) com bounding box por triângulo
- [x] clipping de polígonos (Sutherland-Hodgman) contra o frustum, em espaço de câmera
- [x] z-buffer / teste de profundidade (`RenderPass::Depth` já sai do rasterizador)
- [x] coordenadas baricêntricas com correção de perspectiva (`perspective_correct`)
- [x] suporte a `Mesh` (não só `Triangle` solto)
- [x] tesselação (`Primitive::tessellate`): o rasterizador só vê malha, e `Cube`/`Sphere` viram triângulos (resolução da esfera fixa)
- [x] backface culling (`backface_cull`)

## M1.5 — Transforms e shading

O modelo de iluminação escolhido (Cook-Torrance GGX com metallic/roughness ao
lado de Lambert/Phong), o `Material` reparametrizado e as armadilhas estão em
[design_api.md](design_api.md), seção "Modelo de iluminação".

### Transforms

Teoria em [teoria/transform.md](../teoria/transform.md), o primeiro arquivo de
teoria que vale pros dois pipelines.

- [x] `Object { geometry, material, transform }` e `Scene.objects: Vec<Object>`;
  `Material` sai do `Primitive` e da `Mesh`, que passam a ser geometria pura
- [x] `Transform { translation, rotation (Euler XYZ em graus), scale }`, com
  identidade como `Default` — sem `Option<Transform>`
- [x] aplicação **passo a passo** (`scale` → `rotate` → `translate`), sem `Mat4`
- [x] `apply_normal` com escala recíproca: é a forma legível do que a inversa
  transposta faz
- [x] `undo_point` (a inversa, desfeita na ordem contrária)
- [x] rasterizador transforma a **geometria**, ray tracer transforma o **raio**
  com a inversa e devolve o resultado ao mundo — mesmo problema, sentidos
  opostos (item 8 de [diferenciais_possiveis.md](diferenciais_possiveis.md)); a
  `Sphere` continua analítica e ganha elipsoides sem tesselar. A direção do raio
  local fica **sem normalizar**, pra que o `t` do acerto valha nos dois espaços
- [x] regra de composição escrita: os campos do primitivo (`center`, `radius`,
  `half_edge`) são geometria em espaço local, o transform do objeto vem por
  cima

### Cor de saída (antes do shading)

Correção do que já existe, e pré-requisito para qualquer modelo de iluminação
render imagem parecida com a de um renderer de referência. Teoria em
[teoria/color.md](../teoria/color.md):

- [x] `Rgb::to_bytes()` codificando sRGB e `from_bytes()` decodificando — hoje o
  valor linear vai direto pro byte do PNG, o que escurece todo meio-tom
- [x] decodificar só o que é cor: albedo é cor, roughness/metallic/normal são
  dado. Não precisa de campo — o `Value<T>` do material já diz qual é qual
  (`Value<Rgb>` decodifica, `Value<f32>` e `normal_map` não). Nada a fazer aqui
  até os mapas de M2 existirem; a distinção já está no tipo

### Shading

Em duas levas: a BRDF de Cook-Torrance é um braço de `match` numa função
pequena, então Lambert/Blinn-Phong pode ir sozinho primeiro.

Primeira leva — **implementada**:

- [x] `shade()` em módulo próprio (`src/shading.rs`), consumido pelos dois modos —
  define a fronteira do código compartilhado antes de textura empilhar por cima.
  Forma: `shade()` itera as luzes e resolve visibilidade/atenuação, e a
  reflectância fica isolada numa função `(material, n, v, l) -> Rgb`, para a
  BRDF nova entrar como braço de `match` sem tocar no resto. A visibilidade
  ainda é sempre 1: raio de sombra é M3
- [x] difuso (Lambert) e especular **Blinn-Phong** — com o vetor `h` do meio do
  caminho, não o vetor de reflexão do Phong de 1975: é o mesmo `h` que GGX usa,
  e é para ele que vale a relação de expoente com `roughness`. Phong clássico
  fica como possível quinta variante depois, se valer como conteúdo
- [x] **(antecipado)** `Material` já reparametrizado: `roughness` e `metallic`
  entram mesmo com só Phong lendo (o expoente sai de `roughness`, em uma linha);
  `shininess` e `reflectivity` saem — o segundo aproveitando que ninguém lê
  ainda
- [x] **(antecipado)** `Value<T> { Constant(T), Map(Texture) }` como tipo de cada
  canal do material, com `sample(uv)` — é o que faz a textura entrar sem que o
  `shade()` ganhe um `if` por canal, e trocar depois mexeria em toda cena de
  exemplo
- [x] **(antecipado)** `Mul<Rgb>` e `lerp` no `Rgb` — Phong com luz colorida já
  precisa da multiplicação componente a componente
- [x] **(antecipado)** a subseção "Cor de saída" acima, na parte que entrou (o
  sRGB dos dois lados; o estágio de exposição/tonemap ficou de fora por decisão)
- [x] os tipos de `Light`: `Point` (com atenuação), `Directional`, `Ambient`
- [x] `ShadingOptions { model: Flat | Gouraud | Phong }`
- [x] amostragem de textura *nearest*, com a uv já interpolada e `Texture::texel`
  que já existem. Mipmap e filtros ficam em M4; o que se ganha
  aqui é a cena de xadrez, que é a imagem que melhor mostra
  `perspective_correct` ligado e desligado

## M2 — Ray tracing a fundo

Teoria em [teoria/raytracer.md](../teoria/raytracer.md), da seção "O raio e o
triângulo" para baixo. Refinamentos da implementação em
[design_api.md](design_api.md), seção "Refinamentos vindos da implementação de
M2".

- [x] geração de raios primários a partir da câmera
- [x] interseção raio-triângulo (Möller–Trumbore ou equivalente) — e com ela
  `Mesh` e `Cube`, que é resolvido pelas doze faces da mesma tesselação que o
  rasterizador consome, sem construir a `Mesh` a cada raio
- [x] interseção raio-esfera (equação algébrica direta)
- [x] modelo recursivo de Whitted: raios de sombra, reflexão, refração
- [x] `max_depth` controlando a recursão
- [x] suporte aos tipos de `Light` no cálculo de iluminação
- [x] reflexão sem `reflectivity`: a intensidade do raio de espelho vem do
  Fresnel do material, e o raio só é disparado enquanto `roughness < 0.1` —
  acima disso, Whitted não dá conta do lóbulo GGX com um raio só, e a
  divergência entre o que a BRDF descreve e o que o modelo consegue traçar vira
  cena de demonstração em M5
- [x] `shading::shade` continua sendo o termo **direto** de Whitted, e a
  recursão fica no `trace()` do ray tracer. A assinatura mudou nos três pontos
  previstos (`Camera` virou `view: Vec3`, entrou a visibilidade por callback,
  `f0`/`fresnel_schlick` viraram públicas) — ver "O `shade()` sob o modelo de
  Whitted" em [design_api.md](design_api.md)
- [x] deslocamento por epsilon nos raios secundários (o *shadow acne*), e
  reflexão interna total caindo em reflexo em vez de virar caso de erro

## M3 — Aceleração

Teoria em [teoria/bvh.md](../teoria/bvh.md). Refinamentos da implementação em
[design_api.md](design_api.md), seção "Refinamentos vindos da implementação de
M3".

- [x] BVH em dois níveis, com a mesma construção: a de baixo (BLAS) sobre os
  triângulos de cada `Mesh`, em espaço local; a de cima (TLAS) sobre os
  `Object`, em espaço de mundo, com a caixa de cada um vinda dos 8 cantos da
  AABB local (`Primitive::bounds`). Na folha da TLAS o raio passa pelo
  `undo_ray`; `Mesh` desce pela BLAS, e os outros primitivos chamam `intersect`
  direto
- [x] as duas árvores são construídas no início do `render()` e percorridas pelo
  ray tracer, com `Primitive::intersect` intacto — ver "Onde a BVH mora" em
  [design_api.md](design_api.md)
- [x] construção *top-down* da BVH, partindo na mediana dos centroides pelo eixo
  mais comprido
- [x] travessia da BVH durante interseção de raios, com o filho mais perto
  primeiro e o limite encolhendo pelos dois níveis
- raio de sombra parando no primeiro acerto (*any-hit*) — implementado e
  depois **removido**: o modo de busca a mais atravessava a travessia e a força
  bruta por ~1% dos testes desta cena; ver
  `teoria/bvh.md#a-sombra-so-precisaria-de-um`
- [x] toggle `RayTracingOptions.acceleration.bvh` (ligado/desligado) pra
  comparar tempo de render — o exemplo `dois_modos` imprime os dois
- [x] `RenderPass::RayCount` e `RenderPass::BvhHeatmap`

## M4 — Texturização

Decisões e o que ficou de fora em [design_api.md](design_api.md), seção
"Texturização (M4)".

- [x] interpolação de UV por triângulo — já vinha de M1, com correção de
  perspectiva
- [x] geração da pirâmide de mipmaps (Williams, 1983), dentro de `Texture::new`,
  aceitando lado fora de potência de dois
- [x] seleção do nível de mip pela derivada de tela: diferença finita da uv nos
  pixels vizinhos, **só no rasterizador**
- [x] filtros: nearest, bilinear, trilinear
- [x] os mapas do material, como `Value::Map` em `albedo`, `roughness` e
  `metallic`
- ~~`Texture::from_file` / `Texture::from_bytes`~~ — **fora**: estava no roadmap
  por engano, a lib não lê arquivo de imagem

## M5 — Refatoração e correções gerais

- [x] usar equações de plano pra ray-tracing de cubo; adicionar uvs

- [x] SAH (*Surface Area Heuristic*) com baldes na construção da BVH, ao lado da
  mediana, escolhido por `AccelerationOptions.split`; na TLAS, o custo de uma
  malha é o custo esperado da BLAS dela — ver `teoria/bvh.md#construcao-pelo-sah`

- [x] `OutputOptions { exposure, tonemap: None | Reinhard }`

- Prerequisitos pra bater com glTF:
  ? scene graph
    # pode ficar como flat array
  ? 1 material per face
    # pode ser contornado importando cada primitive em uma mesh diferente
  [x] spotlights
  [x] unidades de medida luzes
  [x] `Material.emissive` (`Value<Rgb>`, sem `KHR_materials_emissive_strength`),
    somado no `shade` como o `Le` da equação; `Object.casts_shadows` junto
  - normal map, normal por vértice, tangentes por vértice

- permitir acompanhar 1 pixel

- Cook-Torrance com GGX, Smith height-correlated e Fresnel de Schlick, na
  parametrização metallic/roughness (o subconjunto do Principled do Blender que
  o glTF 2.0 padronizou), como quarta variante `CookTorrance`
- Quaternions

- `wireframe` e `msaa` como toggles de `RasterOptions`
- toggle pra renderizar bvh
- `RenderPass::Depth`, `Normal`, `Uv` implementados nos dois modos

- (talvez) permitir scale = 0
- (talvez) filtro anisotrópico
- (talvez) tonemap ACES

## M6 — Cena em `.glb`

O formato é desenhado em [design_api.md](design_api.md) antes do código, sem
depender do editor de M6: o editor é um produtor do formato, não a definição
dele.

- `Scene` → `.glb` e `.glb` → `Scene` fora do núcleo, em módulo à parte e atrás
  de feature, como `import`: o core continua sem dependências
- objetos como lista **plana** de nós. A hierarquia de um `.glb` de terceiros é
  achatada na importação: a subárvore vira uma `Mesh` só, com o transform
  aplicado nos vértices (que absorvem até cisalhamento), ou vários objetos
- `Sphere` e `Cube` gravados como malha tesselada, com uma extensão própria
  (`TCC2_primitive`, não `KHR_`) dizendo qual primitivo exato é. Fora de
  `extensionsRequired`, pra que outros visualizadores mostrem a malha
- `Material` em `pbrMetallicRoughness`, com `transparency` e `refractive_index`
  em `KHR_materials_transmission` e `KHR_materials_ior`. Os mapas de metallic
  (canal B) e roughness (canal G) são empacotados numa textura só, e só o albedo
  é codificado em sRGB
- `Camera` como nó olhando para −Z: `look_at` e `up` saem da rotação do nó. Na
  ortográfica, `xmag`/`ymag` são metade de `width`/`height`
- `Light::Point` e `Light::Directional` em `KHR_lights_punctual`, com a
  intensidade gravada crua, sem converter para candela/lux. `Ambient` não existe
  no glTF e vai na extensão própria
- `Transform` com Euler XYZ ↔ quaternion. A volta dá a mesma rotação, mas não
  necessariamente os mesmos ângulos
- `Parameters` opcional no arquivo, como preset e fora da `Scene`: a importação
  devolve `(Scene, Option<Parameters>)`
- o exemplo `gltf_raytracing` passa a usar a importação da lib
- scene to json for inspecting

## M7 — Editor de cenas em Godot

Para quem está começando em Computação Gráfica, que não deveria precisar da
interface do Blender pra montar uma cena.

- adicionar primitivos da lib como nós próprios, com os campos dela (raio,
  meia-aresta), além de câmera, luzes e `.glb` importado
- cena plana, sem objetos aninhados; `.glb` importado é achatado como em M5
- `rotation_order` XYZ nos nós (o padrão do Godot é YXZ), pra que os ângulos do
  inspetor sejam os do `Transform`
- preview neutro, sem prometer a imagem da lib
- exportação para o formato de M5 com `GLTFDocument` e uma
  `GLTFDocumentExtension` que grava `TCC2_primitive`, a intensidade crua das
  luzes, `Ambient` e o preset de `Parameters`

## M8 — Validação

- aplicar a lib num exercício real de disciplina de Computação Gráfica
- coletar feedback dos alunos/uso
- comparar resultados com literatura/implementações consolidadas — a mesma cena
  no Blender (Principled BSDF) é o padrão-ouro barato para o shading, e as
  diferenças esperadas já estão listadas em "Modelo de iluminação" de
  [design_api.md](design_api.md)
- redigir conclusões do TCC2 a partir dos dados coletados
