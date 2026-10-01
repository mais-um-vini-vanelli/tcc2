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
  [x] normal map, normal por vértice, tangentes por vértice

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

## M6 — Cena em `*.tcc2.glb`

O formato está documentado em `src/glb.rs`, e a regra é uma só: **o glTF padrão
manda sempre que ele representa o dado**; a extensão própria guarda o que não
cabe em glTF nenhum. Assim uma ferramenta externa abre o arquivo e vê a cena
inteira, e a lib lê de volta o que gravou.

- [x] `Scene` → `.glb` e `.glb` → `Scene` fora do núcleo, em `src/glb.rs` e
  atrás da feature `glb`: o core continua sem dependências. A API é
  `encode(&Scene, Option<&Parameters>) -> Vec<u8>` e
  `decode(&[u8]) -> (Scene, Option<Parameters>)` — bytes, não caminho, que a lib
  não toca em arquivo
- [x] objetos como lista **plana** de nós, sem `children`
- ~~a hierarquia de um `.glb` de terceiros é achatada na importação~~ —
  **fora**: o decoder de M6 assume cena gerada pela lib. Quem lê `.glb`
  arbitrário é o carregador dos exemplos (`examples/comum/mod.rs`), que já faz
  esse achatamento
- [x] `Sphere` e `Cube` gravados como malha tesselada, com uma extensão própria
  dizendo qual primitivo exato é. Fora de `extensionsRequired`, pra que outros
  visualizadores mostrem a malha. A extensão é **`TCC2_extras`** (uma só, pra
  tudo que é da lib), e não `TCC2_primitive`: junto do primitivo ela guarda
  `Light::Ambient` e `Object.casts_shadows`, que também não têm lugar no glTF
- [x] `Material` em `pbrMetallicRoughness`, com `transparency` e
  `refractive_index` em `KHR_materials_transmission` e `KHR_materials_ior`. Os
  mapas de metallic (canal B) e roughness (canal G) são empacotados numa textura
  só, e só o albedo e o emissivo são codificados em sRGB
- [x] **(a mais)** o emissivo passa de 1 e o `emissiveFactor` do glTF não: o
  excesso vira `KHR_materials_emissive_strength`. Sem isso o emissivo de
  `dois_modos` seria cortado na gravação
- [x] **(a mais)** o truque do fator × textura: um canal constante vira *fator*
  com o texel em 1.0, e um canal com mapa vira *texel* com o fator em 1.0. É o
  que faz a constante voltar exata apesar do PNG de 8 bits, e o que desfaz o
  empacotamento de metallic/roughness sem precisar de campo extra
- [x] `Camera` como nó olhando para −Z: `look_at` e `up` saem da rotação do nó.
  Na ortográfica, `xmag`/`ymag` são metade de `width`/`height`. Como só a
  orientação é gravada, os dois voltam reescritos — outro par de valores, a
  mesma base
- [x] `Light::Point`, `Directional` e `Spot` em `KHR_lights_punctual`, com a
  intensidade gravada crua, sem converter para candela/lux. `Ambient` não existe
  no glTF e vai na extensão própria — ~~o que a tira do lugar que ocupava em
  `Scene.lights` e a devolve no fim da lista~~: desde M7.2b ela é um **nó** com
  `TCC2_extras` em cima, em vez de uma entrada numa lista no documento, e volta
  no lugar dela na árvore
- [x] `Transform` com Euler XYZ ↔ quaternion. A volta dá a mesma rotação, mas não
  necessariamente os mesmos ângulos
- [x] `Parameters` opcional no arquivo, como preset e fora da `Scene`: a
  importação devolve `(Scene, Option<Parameters>)`
- ~~o exemplo `gltf_raytracing` passa a usar a importação da lib~~ — **fora**:
  ele carrega `.glb` de terceiros, e o decoder daqui assume cena da lib. O
  exemplo do formato é o `formato`, que monta uma cena, grava, lê de volta,
  compara campo a campo e renderiza as duas
- ~~scene to json for inspecting~~ — **coberto pelo formato**: o chunk 0 de um
  `.glb` já é a cena inteira em JSON, e `npx gltf-transform inspect
  renders/cena.tcc2.glb` (ou qualquer leitor de GLB) a imprime

O arquivo do exemplo passa pelo validador da Khronos com **zero erros**. Sobra um
aviso, `MESH_PRIMITIVE_GENERATED_TANGENT_SPACE`, na malha que tem normal map e
não tem `TANGENT` — e ele está certo: é exatamente a `Mesh { tangents: None }` da
lib, que calcula a base pela uv do triângulo na hora, e gravar tangentes
inventadas ali quebraria a ida e volta.

## M7 — Editor de cenas em Godot

Para quem está começando em Computação Gráfica, que não deveria precisar da
interface do Blender pra montar uma cena.

Em três etapas, e a ordem importa: primeiro o editor, que fecha sozinho sem
gravar nada da lib; depois o scene graph na lib, que é o que a árvore do editor
cobra do outro lado; e só então a gravação em `.tcc2.glb`, que é onde o arquivo
de terceiro aparece e precisa ser normalizado.

### M7.1 — Editor, sem persistir a cena em `.tcc2.glb`

A cena é salva na `.tscn` do próprio Godot. A ida para o formato de M6 é M7.3, e
até lá o editor é avaliado pelo que mostra, não pelo que grava.

- [x] adicionar primitivos da lib como nós próprios, com os campos dela (raio,
  meia-aresta), além de câmera, luzes e `.glb` importado
- [x] `rotation_order` XYZ nos nós (o padrão do Godot é YXZ), pra que os ângulos do
  inspetor sejam os do `Transform`
- [x] os campos que o glTF não tem já aparecem no inspetor — `casts_shadows`,
  `Light::Ambient`, o preset de `Parameters` — mesmo sem destino ainda: são eles
  que viram `TCC2_extras` em M7.3
- [x] preview neutro, sem prometer a imagem da lib
- [x] hierarquia livre, objeto dentro de objeto: o editor é uma árvore do Godot, e é
  essa árvore que pede o scene graph de M7.2. Proibir o aninhamento só porque a
  `Scene` de hoje é plana seria espremer o editor pra caber na lib, e não o
  contrário

### M7.2 — Scene graph na lib

A hierarquia entra na `Scene`, e o achatamento vira um passo explícito: a
geometria de um lado, a cadeia de transforms do outro.

Teoria em [teoria/transform.md](../teoria/transform.md), seção "A cadeia da
árvore".

- [x] a lista plana de `Object` vira árvore — um nó com `transform`, `children` e
  o conteúdo (geometria + material). O achatamento de M6 ("objetos como lista
  plana de nós, sem `children`") fica substituído por isto
- [x] **(a mais)** `Object.geometry` vira `Option<Primitive>`: o nó sem
  geometria só agrupa, e é o `Tcc2Node` do editor e o nó sem `mesh` do glTF.
  Nada de tipo novo — `Object::group()` monta um, e `Scene::flatten` não emite
  instância pra ele
- [x] `flatten()` percorre a árvore uma vez e devolve o que os dois
  renderizadores já consomem: uma lista de instâncias. A geometria **não é
  copiada** — cada instância aponta para ela e carrega o **array de transforms**
  do nó até a raiz
- [x] composição sem `Mat4`: a decisão de M1.5 (`scale` → `rotate` →
  `translate` passo a passo) continua valendo, porque a cadeia é aplicada elo a
  elo, do nó para a raiz. Pai com rotação e filho com escala não uniforme não
  colapsa num único TRS, e guardar o array evita introduzir a matriz só por
  causa desse caso; o preço é um laço curto por ponto em vez de uma
  multiplicação. O array mora num tipo próprio, `transform::Chain`, com o mesmo
  conjunto de `apply_*`/`undo_*` do `Transform`
- [x] `undo_point`/`undo_ray` andam a mesma cadeia ao contrário, e
  `apply_normal` aplica a escala recíproca elo a elo — a folha da TLAS e a
  `Sphere` analítica continuam como estão
- ~~a BLAS passa a ser construída **uma vez por geometria**, não uma por
  instância~~ — **fora**: continua uma por instância. A separação entre
  geometria e transform só foi até a `Instance`, que **empresta** a geometria em
  vez de copiá-la; o `Object` continua **dono** do `Primitive` dele, e o
  `flatten` visita cada nó uma vez, então duas instâncias nunca apontam para a
  mesma geometria e não há o que compartilhar. Chegou a existir uma deduplicação
  por ponteiro aqui, e ela nunca disparava: em toda cena dava *n* instâncias e
  *n* geometrias distintas. O ganho depende de a geometria poder ser
  compartilhada — `Rc<Primitive>` no `Object`, ou uma tabela de geometrias na
  `Scene` com o nó guardando um índice —, que é mudança de API e fica para
  quando o `.glb` de terceiro pedir: o glTF já reusa `mesh` entre nós, e hoje o
  `decode.rs` clona uma por nó
- [x] `encode.rs`: os nós saem com `children`, e o `transform` de cada um é o do
  elo, não o acumulado — é o que um `.glb` de terceiro já parece
- [x] `decode.rs`: o laço de hoje olha só os nós de raiz da cena padrão e
  ignoraria qualquer filho; passa a descer recursivamente, remontando a árvore
- [x] a ida e volta do exemplo `formato` tem que continuar batendo campo a
  campo, agora com cena aninhada. A cena dele ganhou um grupo que gira com um
  cubo de escala não uniforme dentro, e uma esfera pendurada no cubo — a cadeia
  de três elos e o caso que não colapsa num TRS. O `confere()` do exemplo
  compara a forma da árvore, o primitivo de cada nó e o `casts_shadows`, e
  depois a cadeia de cada instância pelo efeito dela em pontos de prova (o
  Euler volta com a mesma rotação, não com os mesmos ângulos). O arquivo
  aninhado continua passando pelo validador da Khronos com zero erros
- [x] **(a mais)** `tests/chain.rs`: a cadeia batendo com a composição feita à
  mão, `undo_*` desfazendo `apply_*`, o `t` valendo nos dois espaços, a normal
  continuando perpendicular sob escala não uniforme encadeada, e o grupo sumindo
  no `flatten`
- ~~câmera e luz continuam **fora** da árvore~~ — **entraram**:
  o preço de ficarem fora era o editor não poder pendurar uma luz num objeto

### M7.3 — Persistência da cena em `.tcc2.glb`

A gravação mora em `editor/tcc2_export.gd`, e a decisão que abre o marco é onde
ela mora: no editor, e não numa GDExtension que chamasse o `encode.rs` daqui.
Uma GDExtension **não** evitaria a duplicação, só a mudaria de lugar — a árvore
do editor não é uma `Scene`, ela guarda *referências* (`{source, mesh, surface}`
e `{source, image}`) para arquivos de terceiro, então o lado Rust teria que
reabrir esses arquivos com um importador permissivo e ler imagem, que são as
duas coisas que a lib não faz por regra. E é o Godot que já normaliza tudo isso
na entrada.

- ~~exportação com `GLTFDocument` e uma `GLTFDocumentExtension`~~ — **fora**: o
  JSON é montado à mão. O que sobraria para o `GLTFDocument` fazer é o que ele
  faz *do jeito dele*, e cada um desses seria desfeito depois — a conversão de
  unidade da intensidade da luz, o empacotamento de metallic/roughness, o truque
  do fator × textura, `transmission` e `ior`, que o `StandardMaterial3D` nem tem.
  O que o `GLTFDocument` faz de verdade aqui é a **leitura**: toda malha e toda
  imagem entra por ele, e é isso que normaliza o arquivo arbitrário. O que se
  escreve à mão em troca é mecânico: accessor, bufferView, o contêiner GLB e o
  PNG, que o Godot já resolve em uma chamada cada
- [x] `TCC2_extras` com o primitivo analítico e o `casts_shadows`, a intensidade
  crua das luzes, `Ambient` como nó e o preset de `Parameters` no documento
- [x] a hierarquia do editor sai direto em `children`, já que M7.2 ensinou a
  árvore para os dois lados do formato
- [x] o arquivo gerado passa pelo validador da Khronos com **zero erros e zero
  avisos**, como o exemplo de M6. Sobram notas de severidade *info*: a
  `TCC2_extras` que o validador não conhece, e o `TANGENT`/`TEXCOORD_0` que a
  esfera e o cubo do Godot trazem sem material que os use
- [x] a conferência que prende os dois codificadores é o exemplo `editor`:
  `editor/tests/export_test.gd` monta uma cena e grava, e
  `cargo run --example editor --features glb` lê de volta pelo `decode` estrito,
  confere campo a campo (preset, câmera, as quatro luzes, os primitivos exatos,
  a cadeia de três elos, `casts_shadows`, transmissão/ior, o emissivo acima de
  1) e depois reescreve com o `encode.rs` para ver se a forma muda. A câmera é
  comparada pela **base**, não pelos números: só a orientação é gravada, então
  `look_at`/`up` voltam como outro par — a mesma ressalva de M6
- [x] o sentido dos triângulos: o importador de glTF do Godot troca os dois
  últimos índices de cada triângulo para virar o *counter-clockwise* do glTF no
  *clockwise* do Godot, e a gravação desfaz isso. Conferido na esfera exportada,
  com as 4096 faces não degeneradas apontando para fora e concordando com o
  `NORMAL`
- a imagem sai sempre em PNG, que é o único formato que o `decode.rs` lê (a
  feature `glb` só liga `image/png`). Um `.glb` de terceiro cheio de JPEG em 4K
  incha bastante na ida — o `tome.glb`, 45 MiB, vira 76 MiB. Ligar `image/jpeg`
  na lib e deixar o JPEG passar direto é a saída, e é mudança do lado Rust
- normalização do que vem de fora — era aqui que estava o trabalho, porque o
  `.glb` importado no editor é arbitrário e o decoder da lib é estrito:
  - [x] **onde ela roda**: no editor, com o `decode.rs` ficando estrito como
    está. A regra de M6 ("o decoder da lib lê o que a lib gravou") fica de pé, e
    o caminho de importação de cena de terceiro ao lado do `decode` não precisou
    existir
  - [x] textura fora de `R8G8B8` (RGBA, cinza, 16 bits, JPEG), que fazia o
    `pop_image` de `decode.rs` devolver "imagem não está em RGB8": resolvida por
    um `Image.convert(FORMAT_RGB8)` antes do `save_png_to_buffer`. Os bytes vão
    como vieram — um albedo de glTF já está codificado em sRGB, que é
    exatamente o que o decoder desfaz, e normal e metallic/roughness já são
    crus —, então não há conversão de cor na passagem
  - [x] malha com mais de um `primitive`: cada um vira um `Object`, que é o
    contorno previsto em M5 para "1 material per face". Já vinha do
    `Tcc2Import._mesh()` de M7.1
  - [x] atributos que faltam ou sobram: `NORMAL`/`TEXCOORD_0`/`TANGENT` só são
    gravados quando existem, e `TEXCOORD_1`, cor por vértice e skin saem com um
    aviso nomeando o nó — descartados de forma explícita, não silenciosa. Os
    avisos voltam numa lista para quem chamou, em vez de morrerem no log
  - [x] material fora do subconjunto: normalizado por construção, porque a
    gravação monta o material do zero a partir do `Tcc2Material` — `alphaMode`
    sempre opaco, `doubleSided` sempre ligado (o ray tracer acerta a face de
    trás), sem oclusão, sem `KHR_texture_transform` e com o sampler padrão. O
    que o Godot tinha a mais já havia sido avisado e largado na importação

- [x] **a volta pelo editor**: o `tcc2_import.gd` lê o `TCC2_extras` junto com o
  resto, então um `.tcc2.glb` reabre como a cena que era — esfera e cubo voltam
  dos campos exatos, `Ambient` vira nó de luz de novo, `casts_shadows` volta, e
  o preset de `Parameters` volta para a raiz. O Godot entrega o documento cru em
  `GLTFState.json` e o import já andava por índice de nó, então tudo isso é
  aditivo: um `.glb` de terceiro simplesmente não tem esses campos e o caminho
  antigo continua valendo. Fica de fora, de propósito:
  - o `center` de `Sphere`/`Cube`: a lib guarda no primitivo e o editor posiciona
    pelo transform do nó. Somar na posição seria errado assim que o nó tivesse
    rotação ou escala própria, então ele é **descartado com aviso**. Tudo que o
    editor grava tem `center` zero, então só um arquivo escrito pela lib esbarra
    nisso
  - o `"triangle"`: o editor não tem esse *kind*, então continua malha — que é o
    mesmo triângulo de qualquer jeito
  - `transmission` e `ior`: o importador do Godot ignora as duas extensões, então
    elas são lidas do json cru
  - metallic/roughness constante ao lado de um mapa: num arquivo da lib o canal
    mapeado sempre tem fator exatamente 1.0, então um fator diferente de 1.0 *é*
    a constante. Num arquivo arbitrário o fator multiplica o mapa, e lá a leitura
    antiga continua. Uma constante que é exatamente 1.0 volta como mapa sólido —
    mesmo valor, mesmos bytes na regravação, só o widget do inspetor muda
- [x] **espaço de cor dos valores constantes** — um `Color` do editor é sRGB (sai
  de um color picker e alimenta um `StandardMaterial3D`), e o fator do glTF, a
  cor de uma luz punctual e o `Rgb` da lib são lineares. A conversão faltava nos
  dois sentidos, e sem ela toda cor constante mudava a cada ida e volta: o
  importador do Godot já devolve `baseColorFactor`/`emissiveFactor` em sRGB, e a
  gravação escrevia o `Color` cru como se fosse linear. Agora converte nos cinco
  lugares em que uma cor cruza a fronteira (albedo, emissivo, luz punctual,
  `Ambient` e o fundo do preset), com o emissivo dividindo a força **em espaço
  linear**, que é onde o glTF multiplica. Conferido: gravar → importar → gravar
  dá o mesmo arquivo em precisão de f32, que é a que o glTF carrega
- [x] o exemplo `editor` roda nos dois arquivos — o que sai do editor e o que
  volta por ele (`cargo run --example editor --features glb -- <arquivo>`), e os
  dois passam pelo validador da Khronos com zero erros e zero avisos

## M8 — Documentação em VitePress

A lib já tem a teoria escrita em `teoria/*.md`; falta o lugar onde ela é lida na
ordem, com a figura ao lado do texto. O site é um projeto Node.js separado, e a
regra do núcleo não muda: nada de JS entra no crate, e `cargo build` continua
funcionando sem Node instalado.

- projeto VitePress em `docs/`, com `package.json` próprio e `node_modules`/
  build no `.gitignore`
- o conteúdo é o que já existe: `teoria/*.md` são os capítulos. O roadmap e as
  notas de `misc/` ficam de fora — são nota de projeto, não documentação
- ? como os `teoria/*.md` chegam no `srcDir` do VitePress sem virarem cópia que
  desatualiza: apontar o `srcDir` para a raiz com `includes`, ou um passo de
  build que copia. Duplicar o arquivo à mão está fora
- referência da API vem do `cargo doc`, servida ao lado e linkada — o rustdoc
  dos tipos já é bom, e reescrever assinatura em Markdown envelhece
- **Plotly** nas figuras que são função ou dado medido, no lugar da imagem
  estática: distribuição GGX por `roughness`, Fresnel de Schlick por ângulo, a
  curva do sRGB, atenuação por distância, custo do SAH por balde, e os tempos de
  render com e sem BVH que o exemplo `dois_modos` já imprime
- as três figuras de hoje (`teoria/figuras/*.html`) são páginas avulsas escritas
  à mão; viram componente dentro da página do capítulo, e o que for gráfico de
  função passa para Plotly
- as imagens de `renders/` que ilustram um trecho (xadrez com e sem
  `perspective_correct`, o `BvhHeatmap`, os filtros de mipmap) entram como par
  antes/depois, geradas pelos exemplos e não à mão
- publicação em página estática (GitHub Pages ou equivalente), com o build
  rodando a partir do repositório

## M9 — *(opcional)* Backend de rasterização em Vulkan

Fora do caminho crítico do TCC: só entra se M7 e M8 fecharem com folga. O valor
é didático — mostrar onde o rasterizador escrito à mão em M1 corresponde ao que
a GPU faz em hardware, e onde não corresponde.

- feature flag própria (`vulkan`), no mesmo molde da `glb`: o núcleo continua
  sem dependência, e quem não liga a flag nem compila o módulo
- um terceiro `Renderer` implementando a mesma trait, lendo a mesma `Scene` e os
  mesmos `Parameters` — se algum campo não couber, é sinal sobre o design, como
  foi em M0.5 com os dois primeiros modos
- a correspondência é o conteúdo: vertex/fragment shader contra a tesselação e o
  `shade()` da lib, z-buffer contra o teste de profundidade, o clipping que o
  hardware faz sozinho contra o Sutherland-Hodgman de M1, e o sampler contra a
  pirâmide de mipmaps de M4
- o que não corresponde fica dito: `RenderPass::RayCount` e `BvhHeatmap` não
  existem do lado da GPU, e a imagem não vai bater pixel a pixel com a do
  rasterizador em CPU
- ? escopo mínimo que ainda ensina alguma coisa: só o caminho opaco com uma luz
  e textura, ou o material inteiro. O primeiro é o que cabe num marco opcional

## M10 — Validação

- aplicar a lib num exercício real de disciplina de Computação Gráfica
- coletar feedback dos alunos/uso
- comparar resultados com literatura/implementações consolidadas — a mesma cena
  no Blender (Principled BSDF) é o padrão-ouro barato para o shading, e as
  diferenças esperadas já estão listadas em "Modelo de iluminação" de
  [design_api.md](design_api.md)
- redigir conclusões do TCC2 a partir dos dados coletados
