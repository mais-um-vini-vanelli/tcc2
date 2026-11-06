# Apresentação — TCC2

## Roteiro

- Biblioteca em Rust (núcleo do trabalho)
- Editor em Godot
- Documentação com Vitepress + Plotly

---

# Biblioteca em Rust

## Problema

- Aprender computação gráfica é difícil porque *OpenGL, Vulkan e DirectX abstraem parte da renderização* (TODO: quais partes?)
- *Shaders rodam na GPU*, então não se depura com breakpoints como de costume
- Muito do código para usar essas APIs é *boilerplate sem relação com a renderização em si*, como as chamadas para sincronizar dados entre a CPU e a GPU (TODO: quais chamadas fazem isso?)

## Objetivos

1. Software renderer que facilite entender o funcionamento interno das pipelines de renderização
1. Implementar algoritmos clássicos: recorte, rasterização por edge functions, mipmaps, Whitted
1. Projetar uma API simples e coerente com o que aparece em engines de produção
1. Documentar os principais algoritmos implementados, com visualizações interativas
1. Validar a ferramenta em cenário de ensino

## O que ela precisa permitir

- Montar e renderizar cenas sem dificuldade
- Encontrar as implementações de cada parte da pipeline facilmente
- Permitir depurar os algoritmos normalmente, com breakpoints

## Rasterização x ray tracing

- Resolvem o mesmo problema: desenhar uma cena 3D na superfície 2D da tela, mas em sentidos contrários:
  - A rasterização projeta o triângulo na tela e escreve a cor dele nos pixels cobertos
  - O ray tracing traça um raio saindo da câmera e passando pelo centro de cada pixel e escreve a cor do que ele intercepta

## Ray tracing de Whitted

- É o algoritmo clássico de ray tracing (TODO: segundo quem?)
- Simula o trajeto dos raios de luz como no mundo real, mas na direção contrária: saindo da câmera/olho e chegando até a fonte de luz
- Por simular a travessia da luz, ele entrega reflexo, refração e sombra por padrão, sem técnicas adicionais

## Escolhas de arquitetura

- *Parte core em Rust puro, sem dependências*. As funções para abrir e interpretar arquivos de textura e de cena só são compiladas se ligar a feature flag `io` 
- *Uma classe por modo: Rasterizer e Raytracer*
- *Um método render() em cada uma (entry point único e sem estado)*, para permitir renderizar a mesma cena dos dois modos
- *Os dois modos focam na renderização: devolve a imagem como array de pixels, sem código interagindo com o SO pra mostrar a imagem*

## Pipeline de rasterização

1. Achatar o grafo de cena em listas de geometria, luzes e câmera
1. Converter a geometria em triângulos
1. Para cada triângulo:
  1. Aplicar os transforms para levar os vértices de espaço local para espaço de mundo
  1. Levar os vértices de espaço de mundo para espaço de câmera
  1. Fazer o recorte contra o frustum de visão
  1. Projetar os vértices para espaço de tela
  1. Descartar os triângulos de costas para a câmera
  1. Percorrer os pixels dentro da bounding box (fragmentos) e testar quais fazem parte do triângulo
  1. Para cada fragmento:
    1. Interpolar os atributos dos vértices
    1. Teste de profundidade contra o z-buffer
    1. Fazer o shading do fragmento, em espaço de mundo

## Pipeline de ray tracing

1. Achatar o grafo de cena em listas de geometria, luzes e câmera
1. Construir a TLAS e as BLAS
1. Para cada pixel:
  1. Gerar um raio primário
  1. Percorrer as BVHs para descobrir quais objetos ele intercepta
  1. Levar o raio para espaço local e testar interseção com a fórmula da geometria
  1. Fazer o shading do ponto de acerto mais próximo
  1. Se o material tem reflexão ou refração, lançar os raios secundários e somar a cor deles no pixel (processo recursivo)

## Shading

1. Amostrar o material no ponto: albedo, metallic, roughness, emissive e normal:
  1. Se são escalares, usar o valor diretamente
  1. Se são texturas, interpolar as uvs dos 3 vértices e amostrar a textura
1. Somar a luz ambiente
1. Para cada luz:
  1. Calcular a direção e a radiância que chega
  1. Ignorar a luz se ela chega por trás da superfície ou está coberta por outro objeto
  1. Descobrir quanta luz virou difusa e quanta virou especular com fresnel
  1. Para materiais metalicos, cortar o brilho difuso e usar a cor do metal para o brilho especular
  1. Usar a roughness para descobrir o tamanho do brilho especular

## Mipmaps

- O problema: um pixel não enxerga um ponto da textura, e sim uma área. Com o triângulo pequeno na tela, essa área cobre muitos texels, e amostrar um deles devolve um valor qualquer de dentro dela, em vez da média — que é o que o olho enxerga de longe.
- Daria de calcular essa média a cada frame durante o cálculo da cor dos pixels, mas é mais barato gerar versões pré filtradas uma vez só e apenas amostrar essas versões a cada frame.
- Na geração são gerados vários níveis; cada nível tem metade do lado do anterior, e cada texel é a média de um bloco 2×2 até um nível de 1x1
- Na hora de ler a textura é escolhido o nível de tamanho mais parecido com o do triângulo na tela: 
  - Calculamos quanto a uv anda quando o pixel anda um passo em x e um em y (a footprint da uv)
  - Os dois deslocamentos são convertidos para texels, e o nível escolhido é o log₂ do maior: aquele em que um texel tem +/- o tamanho de um pixel
  - O rasterizador consegue calcular fácil o footprint comparando os uvs em pixels visinhos do triângulo; o ray tracer precisaria de outras formas de conseguir essa diferença, que não implementamos

## Cor e saída

- Aplicar a `exposure` para controlar o quão clara é a cena, como a exposição de uma câmera
- Se habilitado, aplicar o `tonemap`
- Converter a imagem para sRGB para converter os componentes de float para byte

## BVHs

- São árvores de caixas AABB que organizam seus itens no espaço: todos os filhos de um nó estão dentro
da caixa AABB dele
- Economiza testes de interseção: se um raio não intercepta um nó pai, não precisa nem testar os filhos
- São usadas em dois níveis: 
  - Uma BLAS por mesh, em espaço local
  - Uma TLAS para todos os objetos da cena, em espaço de mundo
- Não precisam ser construídas a cada frame, principalmente no caso da BLAS
- Na lib, podem ser construídas usando a mediana dos centroides no eixo mais comprido (`BvhMedian`, modo ingênuo), ou SAH com baldes
(`BvhSah`)

## 14. Exemplo com os dois modos

- Comentar sobre sombras, reflexão e refração

## 15. Estatísticas de renderização

- Descrever imagens
- Os renderers também devolvem alguns outros stats, como quantidade de testes de cobertura, quantidade de raios lançados e tempo de construção das BVHs
- Também aceita hooks para depuração: `on_fragment`, `on_triangle`, `on_ray`, `on_segment`

---

# Editor

## Justificativa e estrutura

- *Facilitar a montagem e edição de cenas*: montar via código não é intuitivo e editores prontos como o Blender são difíceis de usar
- Permitir adicionar visualizações específicas da biblioteca
- Godot 4 + GDExtension para chamar a lib em Rust
- Exporta para um formato interno para a lib consumir

## BVH preview

- Uma cor por nível
- Profundidade ajustável
- Mostra o resultado da heurística de construção: mediana vs SAH

## Trace ray

- Raios gerados para desenhar um pixel
- Cor por tipo: primário, sombra, reflexão, refração
- Normal no ponto de acerto; tracejado nos raios que não acerta nada

## Triangle preview

- Mostra o triângulo nos diferentes estágios da renderização
- Local → mundo → câmera → recorte → NDC → tela

## Importação de glb

- Importação no editor (`GLTFDocument`), não na lib
- Na importação são feitas as conversões para o formato da lib:
  - Intensidade de luz em lúmen vira watt
  - Ângulos em radiano viram grau (fov da câmera, cone do spot)
  - O fator de cor do glTF é multiplicado dentro da própria textura
- Gera avisos pra campos não suportados pela lib:
  - Cor por vértice ignorada
  - Offset ou escala de uv descartados
- Permite pegar assets prontos e renderizar na lib

## Formato `.tcc2`

- Tentamos usar o .glb como formato para persistência, mas tinha alguns problemas:
  - As primitivas da nossa lib precisavam ir em extensions, mesma coisa pra `Parameters`
  - Tem uma estrutura mais complexa, com accessors, buffer views, surfaces, samplers, que precisava ser interpretada em Rust e na lib
  - Compatibilidade com o ecossistema glTF não é objetivo do trabalho, e usar o padrão glb indicava que os arquivos gerados podem ser lidos por editores que reconhecem o padrão
- No final, um formato próprio e simplificado ficou melhor, com menos código pra gerar e interpretar e mais fácil de personalizar

- Segue a mesma ideia do glb: um arquivo binário com um cabeçalho JSON + blob para as meshes e texturas
- O cabeçalho JSON espelha o scene graph: objetos com seus campos, transform e objetos filhos
- Listas de `meshes` e `images` no JSON, referenciando o blob por offset + length

---

# Documentação

## Justificativa e estrutura

- Fácil de editar
- Permite visualizações 2D/3D
- Vitepress + Plotly
- *.md e *.vue

---

# Conclusão

## Pré-projeto x projeto

- Atendidos
- Alterados
- Adicionados

- Requisitos atendidos: rasterização, ray tracing, objeto/câmera/luz, textura com mipmaps, Rust, núcleo sem dependência
- Trocado: LBVH (Apetrei, 2014), um dos quatro correlatos, por BVH em dois níveis com SAH — o LBVH existe para construção paralela em GPU, e a lib é CPU em uma thread
- Cresceu: metallic/roughness, normal mapping, spot, exposição e tonemap
- Fora do escopo original: editor, formato próprio e documentação

## 23. Pré-projeto x projeto

- Atendidos:
  - Algoritmos clássicos: Sutherland-Hodgman, edge functions (Pineda), mipmaps (Williams), Whitted
  - API simples para os dois modos
  - Rasterização, ray tracing, objetos/câmeras/luzes, texturização com mipmaps
  - Rust, com código documentado
  - Validação com alunos
- Alterados:
  - LBVH (Apetrei, 2014) por BVH com mediana ou SAH (MacDonald e Booth, 1990)
  - Seleção de mipmap só na rasterização
- Adicionados:
  - Editor em Godot, formato `.tcc2` e documentação com visualizações interativas
  - Parte core sem dependência do SO
  - Instrumentação: buffers de inspeção, contadores e hooks sem efeito no render
  - Exposição, tonemap e conversão para sRGB

## Possíveis extensões

- Cook-Torrance GGX
- Shadow mapping, refrações e reflexões no rasterizador
- Backends de renderização com Vulkan

---

# Demonstração

- Rodar exemplos da lib em Rust e descrever os resultados
- Editor: 
  - Montar cena -> exportar pra .tcc2 -> renderizar pela lib
  - Mostrar previews: BVH, trace ray e triangle preview
- Docs: mostrar estrutura e algumas figuras interativas
