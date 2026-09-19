# BVH — teoria

A aceleração do ray tracer, na ordem em que o pipeline a usa. Os pontos do
código que dependem de cada seção apontam pra cá com um comentário
`// Teoria: bvh.md#secao`.

## Por que acelerar

Sem aceleração, todo raio testa todo objeto da cena — e toda face de toda
malha. O custo de um raio é linear no tamanho da cena, e o render inteiro é
isso multiplicado por pixels e por raios por pixel:

```text
320 × 240 pixels  ×  ~3 raios por pixel (primário + sombras)  ×  100 mil triângulos
  ≈ 2 · 10¹⁰ testes de Möller-Trumbore
```

A saída é a mesma de qualquer busca: organizar os dados antes de perguntar. Os
primitivos que estão perto uns dos outros são agrupados, e cada grupo é
envolvido numa caixa. Se o raio não cruza a caixa, nada lá dentro pode ser
acertado, e um único teste descarta o grupo inteiro. Aplicado de novo dentro de
cada grupo, isso vira uma árvore de caixas — a *bounding volume hierarchy* —, e
o custo de um raio cai para perto de `log n` quando a árvore é boa.

```text
render():
  -> uma BLAS por malha, sobre as faces, em espaço local   (Construção pela mediana,
                                                            Construção pelo SAH)
  -> a TLAS sobre os objetos, em espaço de mundo           (Dois níveis)

para cada raio:
  -> desce a TLAS, caixa por caixa                         (A travessia,
                                                            O teste dos slabs)
  -> na folha, o raio vai pro espaço local do objeto       (transform.md)
  -> malha desce a própria BLAS; o resto chama intersect
```

A BVH não muda nenhuma resposta. O acerto mais próximo é o mesmo com ela ligada
ou desligada (`AccelerationOptions.bvh`); o que muda é quantas perguntas são
feitas até chegar nele. É por isso que ela empilha por cima de
`Primitive::intersect` sem mexer nele: a aceleração decide *quais* primitivos
testar, não como.

O contraste com o rasterizador vale aqui também. O rasterizador percorre os
triângulos, e o custo dele é linear no tamanho da cena por construção: cada
triângulo é projetado uma vez, e não há o que buscar. O ray tracer parte do
pixel e pergunta *o que este raio encontra* — uma busca espacial, e é para
busca que estrutura de dados paga.

## A caixa envolvente

A caixa usada é a alinhada aos eixos (*axis-aligned bounding box*, AABB): o
menor e o maior valor em cada eixo, seis números.

Ela não é a que melhor se ajusta ao objeto — uma caixa orientada (OBB) abraça
melhor um objeto inclinado, e uma esfera é ainda mais barata de testar. A AABB
ganha no que a árvore faz o tempo todo: o teste contra o raio é barato (ver a
seção seguinte) e juntar duas caixas é trivial, o mínimo dos mínimos e o
máximo dos máximos. É o meio-termo que praticamente todo ray tracer usa.

Cada primitivo sabe a própria caixa, em espaço local (`Primitive::bounds`):

```text
Triangle   mínimo e máximo dos três vértices
Mesh       mínimo e máximo de todos os vértices
Sphere     centro ± raio, nos três eixos
Cube       centro ± meia aresta
```

A caixa vazia é a que tem `min = +∞` e `max = −∞`: juntá-la a qualquer outra
devolve a outra, então ela é o ponto de partida de toda união.

## O teste dos slabs

Kay e Kajiya (1986). Uma AABB é a interseção de três *fatias* (*slabs*): em
cada eixo, a região entre os dois planos paralelos que a limitam. Um plano
`x = k` é cruzado pelo raio `O + t·D` quando `O.x + t·D.x = k`, ou seja
`t = (k − O.x) / D.x`. Nos dois planos do eixo `x`:

```text
t0 = (min.x − O.x) / D.x
t1 = (max.x − O.x) / D.x
```

e está dentro da fatia `x` entre esses dois `t`. Se `D.x` é negativo o raio
cruza o `max` antes do `min`, então a entrada é o menor dos dois e a saída o
maior. Estar dentro da caixa é estar dentro das três fatias **ao mesmo tempo**,
ou seja, na interseção dos três intervalos:

```text
entrada = max(entrada_x, entrada_y, entrada_z)
saida   = min(saida_x,   saida_y,   saida_z)

acertou a caixa  ⟺  entrada ≤ saida
```

Em 2D, com o raio subindo para a direita:

```text
               min.x         max.x
                 │             │
                 │             │      ╱
     max.y ──────┼─────────────┼────╳────   sai da fatia y
                 │             │  ╱
                 │             │╱
                 │            ╱│            sai da fatia x
                 │          ╱  │
     min.y ──────┼────────╳────┼─────────   entra na fatia y
                 │      ╱      │
                 │    ╱        │
                 │  ╱          │
                 │╱            │
                ╱│             │            entra na fatia x
```

A entrada na caixa é a **última** entrada (na fatia `y`), e a saída é a
**primeira** saída (da fatia `x`). Se o raio saísse de uma fatia antes de
entrar na outra, a entrada ficaria depois da saída, e ele teria passado ao lado.

Três detalhes que o código herda de graça:

- **O intervalo começa em `t_min..t_max`, não em `−∞..∞`.** Assim a mesma
  comparação descarta a caixa que está atrás da origem do raio e a que está
  depois do acerto mais próximo achado até ali — que é o que a travessia usa
  para podar.
- **A divisão por zero é proposital.** Se a direção tem componente 0 em um
  eixo, é porque o raio passa paralelo a ele; então ele só está dentro dos dois
  planos se a origem está dentro dos dois planos. A divisão é de `f32` e segue a
  IEEE 754: divisão por 0 não causa panic e devolve `±∞` dependendo do sinal do
  numerador. Visualize os planos `x_min = 0` e `x_max = 1`, e um raio com
  `D.x = +0`:

  ```text
  dentro,  O.x = 0.5 : (−∞, ∞)
  fora(−), O.x = −1  : (∞, ∞)
  fora(+), O.x = 2   : (−∞, −∞)
  ```

  Note que só o caso em que o raio está dentro dos planos devolve `(−∞, ∞)`, que
  são valores neutros de `t`; não deixam o intervalo mais restritivo. O único
  caso ruim é a origem exatamente sobre o plano (`0/0 = NaN`), e o `min`/`max`
  do `f32` ignora `NaN`, então aquele eixo só deixa de restringir.
- **A direção não precisa ser unitária.** O `t` sai na mesma parametrização do
  raio, qualquer que seja o comprimento de `D`. É o que deixa a BLAS receber o
  raio de `undo_ray`, que sai sem normalizar.

## Construção pela mediana

A construção é *top-down*: todos os itens começam numa caixa só (a raiz), que é
dividida em dois grupos, e cada grupo é dividido de novo, até sobrar pouco em
cada um.

```text
divide(itens):
    caixa = união das caixas dos itens
    poucos itens?  -> folha
    eixo  = o mais comprido da caixa dos centros
    corta na mediana dos centros naquele eixo
    divide(metade de baixo), divide(metade de cima)
```

Duas escolhas estão ali:

- **O eixo sai da caixa dos centros, não da caixa dos itens.** O que se quer
  separar é onde os itens *estão*, e um único triângulo comprido esticaria a
  caixa dos itens num eixo em que o resto nem se espalha.
- **O corte é na mediana: metade dos itens para cada lado.** A árvore sai
  balanceada, com profundidade perto de `log₂ n`, e nenhum filho fica vazio,
  nem quando todos os centros coincidem. E não é preciso ordenar: a seleção
  (`select_nth_unstable`, um *quickselect*) põe a mediana no lugar, os menores à
  esquerda e os maiores à direita, em tempo linear. A construção inteira fica em
  `O(n log n)`.

O que a mediana balanceia é a **contagem** de itens, não o espaço. Uma estátua
detalhada sobre um chão de dois triângulos grandes corta a estátua ao meio, e o
chão cai inteiro num dos lados, esticando a caixa daquele filho por cima de meia
estátua. A árvore continua correta; só perde a capacidade de descartar.

E os filhos podem se sobrepor. A BVH divide **objetos**, não o espaço: cada item
está em exatamente uma folha, mas as caixas de dois irmãos podem ocupar a mesma
região. É o contrário de uma grade ou de uma *kd-tree*, que dividem o espaço e
podem referenciar o mesmo objeto em várias células. O preço é que achar um
acerto num filho não dispensa o outro de cara — e é disso que tratam as duas
seções da travessia.

### Quando parar de dividir

Na mediana, a folha fica com até dois itens (`ITENS_POR_FOLHA`). É um compromisso: descer
um nível custa dois testes de caixa, e deixar um item a mais na folha custa um
teste de primitivo a mais. Folha de um item só dá uma árvore mais funda; folha
grande testa primitivo à toa.

O número é o mesmo nos dois níveis, e é uma simplificação: um item da TLAS é um
objeto inteiro (às vezes uma BLAS inteira), bem mais caro que o triângulo de
uma BLAS. O SAH troca o número fixo por uma estimativa de custo em cada nó, e
o custo de um objeto pelo da BLAS dele.

## Construção pelo SAH

A mediana escolhe o corte olhando só a contagem. O *Surface Area Heuristic*
(MacDonald e Booth, 1990) escolhe o corte que deixa a travessia mais barata, e
para isso precisa de um modelo do quanto um raio custa. A recursão, o formato da
árvore e a travessia são os mesmos da mediana; muda só a pergunta que `divide`
faz em cada nó. Qual das duas vale é `AccelerationOptions.split`.

### O custo esperado

Um raio que chega num nó `P` já cruzou a caixa dele. A chance de ele cruzar
também a caixa de um filho `F`, que está dentro de `P`, é a razão entre as
áreas de superfície:

```text
chance(cruza F | cruza P) = área(F) / área(P)
```

Vem de um resultado de geometria (teorema de Cauchy): a área da sombra de um
convexo, na média sobre todas as direções, é um quarto da área da superfície
dele. Vale para retas que atravessam a cena em direções uniformes. Os raios de
verdade não são assim: saem de dentro da cena, param no primeiro acerto e vêm
quase todos da câmera. É uma heurística, e o nome não esconde isso.

Com a chance, o custo das duas escolhas num nó:

```text
folha:  Σ custo(item)
corte:  2·C_caixa + área(E)/área(P) · Σ custo(E) + área(D)/área(P) · Σ custo(D)
```

O `2·C_caixa` é o que a travessia paga para descer: ela testa as caixas dos dois
filhos, acerte ou não. Depois disso, cada lado só custa se o raio entrar nele.

Cada nó fica com o corte mais barato e, se nem ele sai mais barato que a folha,
**vira folha**. É o critério de parada que faltava na mediana: em vez de um
número fixo de itens, a folha fecha quando dividir deixa de compensar. Triângulos
amontoados, com caixas quase iguais, fecham numa folha só, porque os dois filhos
teriam a área do pai e o corte só somaria o `2·C_caixa`. E o chão de dois
triângulos sai da estátua logo no primeiro corte: a área dele é grande, mas pesa
sobre só dois itens, e a estátua fica com uma caixa justa.

As constantes estão na unidade de um teste de primitivo: `CUSTO_PRIMITIVO = 1` e
`CUSTO_CAIXA = 0.5`. O teste dos slabs faz seis contas por eixo; o
Möller-Trumbore, dois produtos vetoriais e três escalares antes de decidir. Com
`CUSTO_CAIXA = 1` as folhas saem maiores, e a cena do `dois_modos` testa 707 mil
primitivos em vez de 252 mil.

Na implementação, os custos são comparados já multiplicados por `área(P)`, que é
a mesma para todos os cortes do nó: não há divisão, e um nó degenerado, de área
zero, não vira `NaN`. Duas guardas completam a regra: acima de oito itens
(`MAX_ITENS_POR_FOLHA`) o nó é dividido mesmo que a folha pareça mais barata, e
se todos os centros coincidem, nenhum plano os separa, e cada metade dos itens
vai para um lado.

### Os baldes

Os cortes candidatos são planos perpendiculares a um eixo. Um plano entre cada
par de itens vizinhos exigiria ordenar os itens nos três eixos em cada nó. A
versão com baldes (Wald, 2007) troca isso por uma aproximação linear:

```text
para cada eixo:
    divide a caixa dos centros em 12 baldes iguais
    passa pelos itens: cada um junta caixa, custo e contagem no balde do centro
    de cima pra baixo: acima[p] = baldes de p em diante
    de baixo pra cima: abaixo = baldes antes de p, e avalia o plano p
fica o (eixo, plano) mais barato
particiona os itens no lugar: quem está num balde abaixo do plano vai pra frente
```

São 11 planos por eixo, e cada nó custa `O(n)`, como a seleção da mediana, só
que com três passadas em vez de uma. Planos com um lado vazio são pulados: não
separam nada, e a área da caixa vazia é infinita (`Aabb::EMPTY` tem `min = +∞` e
`max = −∞`), o que daria `0 · ∞ = NaN` no custo.

### O custo de um objeto

Na TLAS o item é um objeto inteiro. Uma esfera ou um cubo custam um teste de
primitivo; uma malha custa uma travessia da BLAS dela. A BLAS é construída antes
da TLAS, então esse custo sai da própria árvore, com a mesma chance por área:

```text
custo(malha) = C_caixa                                      a raiz
             + Σ internos  área(nó)/área(raiz) · 2·C_caixa
             + Σ folhas    área(nó)/área(raiz) · itens · C_primitivo
```

É o custo esperado de um raio que cruzou a caixa do objeto
(`Bvh::custo_esperado`). A razão entre áreas é calculada em espaço local e não
muda com escala uniforme nem com translação; rotação e escala não uniforme a
deixam aproximada. Com isso a TLAS não junta uma malha pesada numa folha com
outros objetos, a menos que as caixas digam que compensa.

### Mediana contra SAH

A imagem é a mesma nas duas (o `.png` sai idêntico byte a byte). O que muda é o
trabalho:

| cena                                  | divisão | primitivos testados | caixas testadas | construção | profundidade da BLAS |
| ------------------------------------- | ------- | ------------------- | --------------- | ---------- | -------------------- |
| `dois_modos`, 320×240                 | mediana | 512 mil             | 1,61 milhão     | —          | 1                    |
| `dois_modos`, 320×240                 | SAH     | 252 mil             | 1,73 milhão     | —          | 1                    |
| malha de 612 mil triângulos, 320×240  | mediana | 1,8 por raio        | 31,3 por raio   | 250 ms     | 20                   |
| malha de 612 mil triângulos, 320×240  | SAH     | 1,3 por raio        | 26,9 por raio   | 940 ms     | 26                   |

A cena do `dois_modos` é o caso da estátua sobre o chão: um chão grande, um cubo
e esferas. O SAH corta pela metade os testes de primitivo e o render cai de
~112 ms para ~86 ms, a troco de um pouco mais de caixas (a TLAS desce de 3 para
5 níveis, separando o que a mediana deixava junto).

Na malha escaneada, de triângulos pequenos e parecidos, a mediana já é boa, e o
SAH ainda tira 28% dos testes de primitivo e 14% das caixas. Mas a construção
custa quase quatro vezes mais, e ela é paga a cada `render()`. A 320×240 isso
come o ganho; o ganho cresce com o número de raios, e a construção não. É por
isso que as duas continuam disponíveis.

## A travessia

```text
percorre(raio):
    limite = t_max
    pilha  = [raiz]                se o raio cruza a caixa da raiz

    enquanto houver nó na pilha:
        tira um nó
        o raio entra nele depois do limite?  -> pula
        folha:    testa cada item; acertou antes do limite?  limite = t do acerto
        interno:  testa a caixa dos dois filhos, empilha os que o raio cruza
```

O que faz a coisa funcionar é o **limite encolhendo**. Cada acerto achado
diminui o intervalo que interessa, e o teste das caixas usa esse intervalo:
toda caixa que começa depois do acerto é descartada sem descer. O primeiro
acerto costuma estar perto, e daí em diante a maior parte da árvore fica de
fora.

A pilha é explícita em vez de recursão. As duas fazem a mesma coisa; com a pilha
fica visível o que está pendente, e a ordem de visita é só a ordem em que se
empilha. O teste `entrada ≥ limite` na hora de tirar da pilha existe porque o nó
entrou nela com um limite maior — outro acerto pode ter aparecido no meio do
caminho.

### O filho mais perto primeiro

Dos dois filhos, o mais longe é empilhado primeiro, para o mais perto sair
primeiro. O acerto mais próximo tende a estar no filho mais perto; achá-lo cedo
encolhe o limite, e quando o filho de longe sai da pilha a entrada dele muitas
vezes já passou do acerto:

```text
   raio ───────►  [  A  ]       [  B  ]
                     ↑ acerto em t = 4
                                   B começa em t = 6: nem desce
```

Na ordem contrária o raio desceria `B` inteiro, acharia um acerto em `t = 8`, e
depois desceria `A` do mesmo jeito. A ordem não muda o resultado, só o trabalho.

## A sombra só precisaria de um

O raio de sombra pergunta outra coisa: não *o que* está entre o ponto e a luz,
mas *se* há alguma coisa. O primeiro acerto válido já responde, e uma busca
dedicada pararia ali — o *any-hit* das APIs de ray tracing em hardware — sem
encolher limite e sem procurar o mais perto.

Pesaria, porque raio de sombra é a maioria dos raios de um render de Whitted: um
por luz em cada acerto, inclusive nos acertos da recursão.

**Este renderer não faz isso**: `ocluido` usa a mesma busca pelo acerto mais
próximo que todo o resto, e só pergunta se ela achou alguma coisa. Ter um modo
de busca a mais significava um parâmetro atravessando a travessia inteira e o
laço da força bruta, para uma economia que nesta cena fica em torno de 1% dos
testes de primitivo — o caminho único se lê melhor, e o que o leitor perde é
nada. Fica como o exercício óbvio de quem quiser medir.

A parada só seria permitida porque a sombra de Whitted é binária
(`raytracer.md#o-raio-de-sombra`): qualquer objeto bloqueia a luz inteira. Com
sombra colorida por objeto transparente seria preciso achar todos os acertos do
caminho, e ela deixaria de valer de todo jeito.

## Dois níveis

Uma BVH única, sobre todos os triângulos da cena em espaço de mundo, obrigaria a
transformar cada vértice para o mundo — exatamente o que o ray tracer evita ao
levar o raio para o espaço local (`transform.md#o-raio-vai-para-o-espaco-local`)
—, e não teria onde pôr a esfera, que sob escala não uniforme é um elipsoide que
nenhum `center` e `radius` descrevem no mundo. Por isso são duas árvores com a
mesma construção:

- **BLAS** (*bottom-level*): uma por `Mesh`, sobre as faces dela, em espaço
  **local** — com os vértices como estão na cena, sem transform nenhum.
- **TLAS** (*top-level*): sobre os `Object` da cena, em espaço de **mundo**. Cada
  item dela é um objeto inteiro.

```text
                  TLAS (mundo)
                 ╱            ╲
            [caixa]          [caixa]
            ╱     ╲               ╲
        objeto    objeto          objeto
        (malha)   (esfera)        (malha)
           │         │               │
        undo_ray  undo_ray        undo_ray
           │         │               │
         BLAS     intersect        BLAS
        (local)    direto         (local)
```

Na folha da TLAS o raio passa pelo `undo_ray`, como na força bruta, e daí a
malha desce a própria BLAS e os outros primitivos (esfera, cubo, triângulo
solto) chamam `Primitive::intersect` direto.

O ponto que amarra os dois níveis é o `t`. Como `undo_ray` não normaliza a
direção, o `t` de um acerto é o mesmo no espaço local e no de mundo — e então o
**mesmo limite** atravessa as duas árvores sem conversão: o acerto achado na
BLAS de um objeto poda caixas da TLAS, e o limite da TLAS poda caixas dentro da
BLAS. Uma decisão tomada em M1.5 por outro motivo é o que deixa a árvore de dois
níveis ser simples.

Os nomes vêm das APIs de ray tracing em hardware (DirectX Raytracing, Vulkan),
que dividem a aceleração exatamente assim — e reaproveitam a mesma BLAS para
todo objeto que usa a mesma malha (*instancing*). Aqui cada objeto constrói a
sua.

### A caixa do objeto no mundo

A TLAS precisa da caixa de cada objeto em espaço de mundo, e o que o primitivo
sabe é a caixa local. Levá-la pelo transform tem um problema: uma caixa girada
não é mais alinhada aos eixos. A saída é transformar os oito cantos e pegar a
AABB em volta deles:

```text
     local                      mundo (girado 45°)

   ┌───────┐                   ┌───────────┐
   │       │                   │     ╱╲    │
   │   ●   │        ──►        │   ╱    ╲  │
   │       │                   │   ╲    ╱  │
   └───────┘                   │     ╲╱    │
                               └───────────┘
                          a caixa alinhada em volta dos cantos
```

O objeto está dentro da caixa local; o transform é afim, então leva a caixa num
paralelepípedo que continua contendo o objeto; e a AABB dos oito cantos contém o
paralelepípedo, que é o fecho convexo deles. A caixa nunca deixa o objeto de
fora.

Ela só fica folgada: um cubo girado 45° em torno de `y` ganha uma caixa √2 mais
larga, e a esfera, cuja caixa local já é um cubo em volta dela, fica mais
folgada ainda. Caixa folgada é custo — o raio entra na caixa e erra o objeto —,
nunca erro.

### As duas profundidades

O `RenderStats` traz `tlas_depth` e `blas_depth` (a mais funda das BLAS)
separados, e não um número só. Não é economia de conta: fundir os dois dá um
número que não existe.

O caminho mais longo da árvore inteira é o maior, entre as folhas da TLAS, do
caminho até a folha **mais** a BLAS que pende dela. Somar os dois máximos
independentes só acerta se a folha mais funda da TLAS for justamente a que
guarda a malha de BLAS mais funda. Com cinco objetos, a mediana parte 5 em 2 e
3: a metade de 2 fecha em folha a dois nós da raiz, a de 3 abre outro nível e
fecha a três. Ponha a malha pesada (BLAS de profundidade 20) na metade rasa e
esferas na outra — o caminho real é `max(2 + 20, 3 + 0) = 22`, e a soma dos
máximos diz 23.

O erro nunca é para baixo e é limitado pela profundidade da TLAS, mas é o tipo
de imprecisão que atrapalha justamente quem confere a conta à mão numa cena de
três objetos. Separados, os dois números são exatos, a soma que o leitor fizer é
a cota, e ainda dizem **qual** dos dois níveis está fundo — que é a pergunta útil
quando o render está lento.

## O que os passes de custo mostram

Dois passes que só o ray tracer produz, e que existem para mostrar onde o
trabalho vai:

- **`RenderPass::RayCount`**: raios por pixel, contando o primário, as sombras e
  a recursão inteira. O fundo vale 1 (o primário, que não acerta nada); as
  esferas de vidro e de metal acendem, porque cada acerto nelas gera raios
  secundários, e cada um desses tem as próprias sombras. E a face virada contra
  todas as luzes também vale 1, igual ao fundo: com `n · l ≤ 0` o `shade` nem
  pergunta a visibilidade (`raytracer.md#o-termo-direto-de-whitted`), e o passe
  deixa esse atalho à vista.
- **`RenderPass::BvhHeatmap`**: caixas testadas por pixel, somando todos os raios
  daquele pixel. Acende nas silhuetas (o raio passa raspando por muitas caixas
  sem acertar nada), na geometria densa e onde há recursão. O fundo perto de um
  objeto custa mais que o fundo longe dele: o raio entra na caixa da raiz e
  desce alguns níveis antes de descobrir que errou. Só existe com a BVH ligada.

Os dois saem dos mesmos contadores do `RenderStats`, como a diferença entre o
valor antes e depois de cada pixel. Então o passe e o total sempre concordam: a
soma do `RayCount` é o `stats.rays`, e a do `BvhHeatmap` é o
`stats.bvh_nodes_visited`.
