# Possíveis diferenciais para afastar o TCC2 dos correlatos

Ideias levantadas para reduzir a sobreposição com os correlatos já mapeados em
[../pesquisa_produtos_correlatos.md](../pesquisa_produtos_correlatos.md)
(TinyRenderer, Nori, Scratchapixel, Rayground, Virtual Ray Tracer, ray tracers em
Rust). Ideia central: deslocar o eixo do trabalho de "reimplementar algoritmos
clássicos já publicados em toda parte" para algo que nenhum correlato entrega.

## 1. Blindar o diferencial arquitetural, não algorítmico

Todo correlato já implementa os mesmos algoritmos clássicos (edge functions,
Whitted, mipmaps). Ninguém oferece uma **representação de cena única
compartilhada** entre os dois pipelines com troca de modo em runtime.

- Reposicionar a contribuição central do TCC como "o design da API/arquitetura
  que unifica rasterização e ray tracing sob a mesma cena", tratando os
  algoritmos como meio, não como fim.
- Desloca a pergunta da banca de "por que reimplementar Pineda/Whitted" para
  "como você desenhou a interface comum" — pergunta que nenhum correlato
  responde, porque nenhum deles tem os dois modos na mesma API.
- Reforça a justificativa já presente no projeto (Akenine-Möller et al., 2018,
  citando rasterização vs ray tracing como duas ordens de resolução do problema
  de visibilidade) ao torná-la algo experimentável, não só teórico.

## 6. Progressão didática em camadas ("caixa branca" configurável)

Permitir ligar/desligar otimizações e componentes individualmente, observando o
efeito visual e de desempenho em tempo real:

- BVH ligado/desligado (efeito no tempo de renderização em cenas complexas);
- correção de perspectiva ligada/desligada (efeito visual de distorção em
  triângulos inclinados);
- mipmaps ligados/desligados (efeito de aliasing em texturas);
- clipping ligado/desligado (efeito de artefatos fora do frustum).

Isso é diferente de "implementar o algoritmo final e mostrar a imagem pronta" —
vira uma ferramenta para **demonstrar por que cada técnica existe**, comparando
o antes/depois. Nenhum correlato faz isso: todos mostram apenas a versão final
já otimizada (TinyRenderer, Scratchapixel) ou a versão completa sem alternância
de componentes (Nori).

## 7. Aprofundar no que ninguém cobre bem: construção de BVH/LBVH

Nenhum correlato trata a construção de LBVH (Apetrei, 2014) como conteúdo
didático incremental:

- Nori deixa a estrutura de aceleração como exercício em aberto para o aluno
  resolver sozinho, sem material de apoio;
- Scratchapixel e TinyRenderer não chegam a abordar BVH/LBVH;
- os ray tracers em Rust encontrados (dps, ranveeraggarwal, matklad, tray_rust)
  não têm foco pedagógico na construção da estrutura de aceleração, tratam-na
  como implementação de suporte.

Posicionar a construção de BVH linear como o capítulo mais aprofundado/original
do trabalho — não só "usar BVH para acelerar", mas explicar e demonstrar o
processo de construção passo a passo — cobre uma lacuna real na literatura
didática disponível.

## 8. Comparar pontos onde os dois paradigmas divergem conceitualmente

Além de validar onde rasterização e ray tracing devem *concordar* (mesma cena,
mesmo resultado — ver `RenderPass::Depth` em
[design_api.md](design_api.md)), há valor didático simétrico em expor
deliberadamente os pontos onde os dois pipelines **resolvem o mesmo problema de
formas fundamentalmente diferentes**:

- **Sombras** — no ray tracer, sombra é só mais um raio (raio de sombra, parte
  natural da recursão de Whitted); no rasterizer, não existe "de graça" — exige
  uma técnica à parte (shadow mapping, um segundo passe de renderização a
  partir da luz). É um ponto de contraste direto entre "resolver por
  simulação" vs "resolver por truque de engenharia".
- **Interseção com esfera** — trivial para o ray tracer (equação algébrica
  direta, `Sphere` como primitiva de primeira classe); inexistente para o
  rasterizer, que só enxerga triângulos e precisaria tesselar a esfera antes de
  desenhar (ver seção Objetos em [design_api.md](design_api.md)).

Como a API já une os dois modos sob o mesmo trait `Renderer` e a mesma `Scene`,
esses contrastes não precisam de exemplos separados — bastam cenas simples
(uma esfera, um objeto projetando sombra) renderizadas nos dois modos lado a
lado, deixando a diferença visível e explicável a partir da própria estrutura
da lib. Isso transforma limitações de cada pipeline em conteúdo pedagógico
propositalmente exposto, em vez de detalhes escondidos de implementação.

## Combinação recomendada

Os itens 1, 6 e 7 se reforçam: uma API unificada (1) que permite ligar/desligar
componentes (6), usada para explicar em profundidade a parte mais negligenciada
pelos correlatos — a construção de BVH (7) — monta um argumento de justificativa
mais forte do que qualquer um isolado.
