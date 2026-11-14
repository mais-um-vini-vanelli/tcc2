# Transform

Leva o objeto do espaço local (onde ele foi modelado) pro espaço de mundo (a cena
atual). O rasterizador e o ray tracer resolvem esse problema em sentidos
opostos: o rasterizador transforma a geometria, enquanto o ray tracer transforma o
raio da câmera.

## O transform

A geometria de um primitivo é escrita em **espaço local**: o cubo tem centro na
origem e aresta `2 * half_edge`, a esfera tem centro e raio, o triângulo tem
três pontos. Nada disso sabe onde o objeto está na cena.

Quem sabe é o `Transform` do `Object`, com três campos:

```text
scale        quanto o objeto estica em cada eixo
rotation     três ângulos de rotação nos eixos X, Y, Z
translation  pra onde o objeto vai
```

Esse tipo de transform é chamado de TRS (as iniciais de cada etapa, lidas da direita pra
esquerda).

## A ordem das três etapas

O transform acontece nessa ordem: escala -> rotação -> translação. E a ordem das três operações
é importante:

**Transladar antes de rotacionar.** A rotação gira em torno da origem; se o objeto já foi movido 
pra longe dela, a rotação faz o objeto orbitar ao invés de girar.

<TransformFig1 />

**Escalar depois de rotacionar.** A escala age nos eixos X, Y, Z, então esticar em `x` rotacionado estica 
em uma direção que não é a "largura" do objeto. O objeto fica distorcido (`shear`): um cubo vira um paralelepípedo 
por exemplo.

<EscalaRotacaoShear />

## Girar em torno de um eixo

Pra girar ao redor do eixo X, giramos o ponto pelo plano YZ que passa por ele.

Todo ponto `(y, z)` pode ser escrito em termos de uma distância até a origem (o
raio `r`) e um ângulo `θ` a partir do eixo `y`:

```text
y = r·cos(θ)
z = r·sin(θ)
```

Se eu giro esse ponto por um ângulo adicional `a`, o novo ponto `(y', z')` tem o
mesmo raio `r` (girar não muda a distância até a origem), mas o ângulo agora é
`θ + a`:

```text
y' = r·cos(θ + a)
z' = r·sin(θ + a)
```

Considere as identidades de soma de ângulos:

```text
cos(θ + a) = cos(θ) * cos(a) - sin(θ) * sin(a)
sin(θ + a) = sin(θ) * cos(a) + cos(θ) * sin(a)
```

Substituindo:

```text
y' = r·[cos(θ)cos(a) - sin(θ)sin(a)] = r·cos(θ)cos(a) - r·sin(θ)sin(a)
z' = r·[sin(θ)cos(a) + cos(θ)sin(a)] = r·sin(θ)cos(a) + r·cos(θ)sin(a)
```

Trocando `y = r·cos(θ)` e `z = r·sin(θ)` do passo 1:

```text
y' = y * cos(a) - z * sin(a)
z' = z * cos(a) + y * sin(a)
```

A mesma lógica vale para a rotação ao redor dos outros eixos.

### Euler XYZ

Três ângulos, aplicados em sequência: primeiro em torno de X, depois de Y,
depois de Z. A ordem importa: girar 90° em X e depois 90° em Y não leva
ao mesmo lugar que girar 90° em Y e depois 90° em X. Por isso "ângulos de Euler"
sempre vem com uma ordem colada no nome, e a daqui é a mesma do Blender.

## A normal usa o transform inverso

O ponto e a normal **não passam pelo mesmo transform**. Achatando um paralelepípedo pela metade em 
`y`, e olhando pra uma das faces:

<NormalEscalada />

A superfície e a normal achataram e ficaram mais deitadas. Mas a normal é perpendicular à 
superfície, e deitar a superfície deveria ter levantado a normal pra manter a perpendicularidade: os 
dois deviam ter girado em sentidos opostos.

### A conta, com números

Uma superfície a 45° no plano XY, escala `s = (1, 0.5, 1)`.

O que a superfície tem de concreto no ponto é o vetor **tangente**: a direção em
que ela anda. Aqui `t = (1, 1, 0)`, e a normal é `n = (-1, 1, 0)`; são perpendiculares
`n · t = -1 + 1 = 0`.

O vetor tangente é geometria de verdade, então ele anda junto com o objeto:

```text
t' = S t = (1, 0.5, 0)
```

Escalando a normal do mesmo jeito:

```text
n_errada = S n = (-1, 0.5, 0)
n_errada · t' = -1 + 0.25 = -0.75  # ≠ 0, não é mais perpendicular
```

Escalando pela escala inversa `(1, 2, 1)`:

```text
n' = S⁻¹ n = (-1, 2, 0)
n' · t' = -1 + 1 = 0                   perpendicular de novo
```

### A conta em geral

O mesmo argumento sem números. Procuramos a matriz `M` que leva a normal, tal
que `n' = M n` continue perpendicular ao tangente transformado `t' = S t`:

```text
n' · t' = 0
(M n) · (S t) = 0
(M n)ᵀ (S t) = 0   # o produto escalar de dois vetores coluna a · b = aᵀ b (a vira linha e o produto vira matricial)
nᵀ Mᵀ S t = 0      # trocando (A B)ᵀ = Bᵀ Aᵀ
```

A última linha diz: `Mᵀ S` tem que transformar todo vetor perpendicular a `n`
em algo que continua perpendicular a `n`. O jeito de garantir isso
para todo `n` e todo `t` é com `Mᵀ S = I`, ou seja:

```text
M = (S⁻¹)ᵀ
```

Como a escala é diagonal, transpor não muda nada e sobra `M = S⁻¹`: a normal é
escalada pelo **recíproco** da escala do ponto.

```text
ponto    escala por (sx, sy, sz)
normal   escala por (1/sx, 1/sy, 1/sz)
```

## O raio vai para o espaço local

O rasterizador percorre vértices, então transformar a geometria é natural: cada
vértice passa por `apply_point`, cada normal por `apply_normal`, e daí em diante
o pipeline nem sabe que houve transform.

O ray tracer percorre os raios, e a geometria pode nem ter vértices (como acontece na esfera, identificada pela 
equação dela). Tesselar todos os objetos pra poder aplicar o transform nos vértices tira uma das vantagens 
do ray tracer, que é intersectar as superfícies exatas. Então ele faz o contrário: **leva o raio pro espaço local 
do objeto** com `undo_ray` e intersecta lá, com o objeto sem transformação.

### A direção não é normalizada, e é de propósito

`undo_ray` **não** normaliza a direção (diferente de `Ray::new`): ele monta o
raio local com a direção possivelmente não unitária.

O motivo é que o `t` da interseção é medido em unidades da direção do raio.
Um transform é afim: ele leva a reta do raio na reta do raio local e preserva
a razão entre distâncias ao longo dela. Então manter a direção local "esticada" 
pelo fator da escala faz o mesmo `t` valer no espaço local e de mundo:

```text
p(t)       = o + t d              # no mundo
p_local(t) = T⁻¹(o) + t · T⁻¹(d)  # no espaço local, mesmo t
```

<RaioEspacoLocal />

## A cadeia da árvore

Até aqui a gente estava olhando pro transform de **um** objeto, mas a cena é uma árvore de
objetos (o scene graph): cada nó tem o seu transform, relativo ao pai, e
o que leva a geometria de um nó até o mundo é a sequência de transforms até a raiz.

```text
raiz
 └── grupo        T_g   (gira 24° em Y)
      └── cubo    T_c   (escala 1, 1.3, 1)
           └── esfera   T_e

um ponto local da esfera vai pro mundo assim:

p_mundo = T_g( T_c( T_e(p) ) )
```

`Scene::flatten` percorre a árvore uma vez e devolve uma **instância** pra cada nó com
geometria. A instância carrega a cadeia `[T_e, T_c, T_g]`, guardada **do nó para a raiz**.

### Por que a cadeia não vira um transform só

Uma alternativa pra essa lista de transform é juntar os transform até o filho em
um único, mas isso não funciona aqui: `Transform` atualmente só pode ser TRS, 
escala → rotação → translação nessa ordem, e algumas combinações não podem
ser guardadas dessa forma. Exemplo:

```text
raiz
 └── grupo            (escala)
      └── cubo        (gira)
           └── esfera

p -> gira (cubo) -> escala (grupo) -> p_mundo
```

Um TRS só sabe escalar antes de girar, e aqui é preciso girar a esfera e depois escalar: é 
um transform com *shear* (eixos XYZ distorcidos), que nenhum `(translation, rotation, scale)` descreve.

Uma alternativa seria representar essa mesma composição com uma única matriz 4x4, 
que consegue representar um conjunto de transformações em uma ordem arbitrária.
