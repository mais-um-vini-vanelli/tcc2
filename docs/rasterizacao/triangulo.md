# O triângulo

Cobertura, baricêntricas e orientação: o que o rasterizador faz depois que o
triângulo já está em pixels. Teste de cobertura por edge functions,
Pineda (1988).

## Edge functions

A edge function é o dobro da área: de onde vêm as baricêntricas.

Essa função responde à pergunta: traçando uma reta infinita que passa por A e B,
o ponto P caiu de que lado? Pra isso tiramos o produto vetorial de `u` e `v`:

```text
u × v = u.x * v.y - u.y * v.x
```

- o **sinal** da resposta diz pra qual lado `v` está de `u`;
- a **magnitude** diz o tamanho do ângulo entre `u` e `v` (com 90° entre eles, o
  resultado chega no ápice).

A edge function `borda(b, c, p)` retorna `2 * área(BCP)`. A mesma coisa pra
`borda(c, a, p)` e `borda(a, b, p)`.

### O resultado da edge function é o dobro da área do triângulo ABP

A fórmula da área de um triângulo é `base * altura / 2`.

A edge function `(A, B, P)` usa dois vetores: `u = B - A` (a base) e
`v = P - A` (uma das arestas laterais). Com `|u|` como comprimento da base, a
área do triângulo `ABP` é `|u| * altura / 2`, e a altura é a distância de P até
a reta que passa por A e B — em outras palavras, a projeção de `v` no eixo
perpendicular a `u`.

::: warning 🚧 TODO Alto
Imagem de `u`, `v` e `u'`.
:::

Então, pra descobrir a altura:

1. rotacionar `u` em 90°, virando `u'` (o eixo perpendicular a `u`);
2. normalizar `u'`, dividindo pelo seu comprimento `|u|`;
3. projetar `v` em `u'` com um produto escalar.

Então a altura do triângulo `ABP` é

```text
h = (u' / |u|) · v
```

Reorganizando a fórmula:

```text
h = (u' · v) / |u|

|u| * h = u' · v
```

Se temos um vetor `u = (x, y)`, a fórmula desse vetor rotacionado em 90° é
`u' = (-y, x)`. Isso pode ser visto em:

```text
(1, 0)  ->  ( 0, 1)    "direita" vira "cima"
(0, 1)  ->  (-1, 0)    "cima"    vira "esquerda"
```

Então `u' = (-u.y, u.x)`:

```text
|u| * h = (-u.y, u.x) · v
```

Desenvolvendo o produto escalar e reorganizando:

```text
|u| * h = -u.y * v.x + u.x * v.y
```

Pode ver que à esquerda da equação ficou `base (|u|) * altura`, e à direita
ficou a fórmula da edge function. Considerando que a área de um triângulo é
`base * altura / 2`, dá de ver que **o resultado da edge function é o dobro da
área do triângulo `ABP`**.

### Da área para o peso de interpolação

Se P está dentro do triângulo, a soma das áreas dos subtriângulos BCP, CAP e ABP
precisa ser igual à área do triângulo ABC.

::: warning 🚧 TODO Alto
Desenho de um triângulo com P dentro e os 3 subtriângulos.
:::

E quanto maior o subtriângulo formado entre uma aresta de ABC e P, mais perto P
está do vértice oposto à aresta, e mais longe ele está dos outros 2 vértices.

::: warning 🚧 TODO Alto
Imagem mostrando P em cima de um vértice, no meio de uma aresta, no meio do
triângulo.
:::

Outra forma de ver isso é comparando o subtriângulo com o triângulo completo:

- subtriângulo BCP → base = BC; altura = distância de P até a reta BC
- triângulo ABC → base = BC (a mesma); altura = distância de A até a reta BC

Os dois têm a mesma base, e quando dois triângulos compartilham a base, a razão
entre as áreas é a razão entre as alturas:

```text
       área(BCP)     base * h_P / 2     h_P     distância de P até a reta BC
l0 = ───────────── = ──────────────── = ───── = ────────────────────────────
       área(ABC)     base * h_A / 2     h_A     distância de A até a reta BC
```

Ou seja: `l0` mede o quanto P está longe da aresta oposta, em relação ao quanto
o vértice A está. E se `l0 = 1`, P e A estão à mesma distância de BC: P está em
cima de A.

Então podemos usar a proporção `área(subtriângulo) / área(triângulo)` para
descobrir o quão perto P está de cada vértice, e usar isso como coeficiente de
interpolação.

## Pesos sob perspectiva

A seção anterior mostrou como sair das edge functions pros pesos: cada peso é a
razão entre a área de um subtriângulo e a área do triângulo inteiro. Só que essa
medida foi feita na tela, depois da projeção; a gente precisa desfazer a
distorção da perspectiva pra que as razões continuem válidas.

### Ortográfica: nada a fazer

A projeção ortográfica é afim, e transformações afins preservam razões ao longo
de uma reta: o ponto médio continua o ponto médio, o ponto a 1/3 continua a 1/3.
Como consequência preservam também razões entre áreas — e peso baricêntrico é
exatamente uma razão entre áreas. Ou seja: o peso medido na tela já é o peso no
mundo, é só interpolar.

### Perspectiva: a razão não sobrevive à divisão por z

A projeção perspectiva não é afim, porque cada ponto é dividido pelo seu próprio
`z`. Ela mantém as retas, mas distorce as proporções ao longo delas.

Uma aresta de `A` até `B`, com os pares escritos como `(x, z)`. O ponto médio
`P`, antes de projetar, sai da interpolação normal:

```text
0.5 * (-1, 1) + 0.5 * (1, 5) = (0, 3)

A = (-1, 1)
P = ( 0, 3)
B = ( 1, 5)
```

Projetando os três (dividindo `x` por `z`):

```text
A' = (-1.0, 1)
P' = ( 0.0, 3)
B' = ( 0.2, 5)
```

A distância em `x` de A->P e de P->B era 1 dos dois lados antes de projetar;
depois da projeção, a metade mais longe ocupa um pedaço bem menor da tela. E é
por isso que interpolar direto na tela erra.

Tentando achar o P original a partir de `A'` e `B'` com `t = 0.5`:

```text
0.5 * (-1.0, 1) + 0.5 * (0.2, 5) = (-0.4, 3)    <- errado, P' tem x = 0.0
```

### O conserto: interpolar em 1/z

O que anda em passo constante na tela não é `z`, é `1/z`. Então os pesos são
levados pra esse espaço, somados e renormalizados:

```text
inversos = [l0/z0, l1/z1, l2/z2]
z        = 1 / (inversos[0] + inversos[1] + inversos[2])
pesos    = inversos * z
```

A soma dos inversos é o `1/z` do próprio ponto, então invertê-la já devolve a
profundidade dele. E o peso corrigido de cada vértice é a fatia dele nesse
`1/z`, renormalizada pra somar 1 de novo. Interpolar qualquer atributo com esses
pesos é andar em passo constante pelo triângulo **no mundo**, não pela tela.

### Por que `1/z` é linear na tela

## Onde a cor é avaliada

Com os pesos na mão, falta decidir em que ponto do triângulo a cor é calculada —
por face, por vértice ou por fragmento. Isso não diz nada sobre *como* ela é
calculada; isso é o modelo de reflectância ([Shading](/fundamentos/shading)).

| frequência | o que faz                                                                                                  |
|------------|------------------------------------------------------------------------------------------------------------|
| `Flat`     | Uma cor por triângulo, da normal geométrica da face. As faces da malha ficam visíveis. |
| `Gouraud`  | Uma cor por vértice, interpolada no interior. Barato, mas o brilho especular só existe se cair em cima de um vértice. |
| `Phong`    | A normal é interpolada e a cor calculada por pixel. É o padrão.                                            |

A escolha só existe no rasterizador: o ray tracer calcula a cor no ponto do
acerto.

### A normal interpolada não é unitária

A normal interpolada entre três normais unitárias não sai unitária, então precisa
normalizar depois de interpolar. Considere `n1 = (1,0)`, `n2 = (0,1)` e `t = 0.5`:

```text
n' = n1 * (1 − t) + n2 * t = (0,5, 0,5)

|n'| = √(0,5² + 0,5²) = √(0,25 + 0,25) = √0,5 ≈ 0,7071
```

## Backface culling

Em objetos sólidos, apenas os triângulos do lado da frente estão visíveis pra
câmera. Pra economizar processamento, podemos descartar os triângulos do lado de
trás, que estão cobertos.

Daria pra usar as normais informadas dos vértices pra saber qual o lado da
frente do triângulo, mas a convenção é usar a ordem dos vértices (winding
order): os vértices das faces de fora de um objeto são informados em ordem
anti-horária e as faces de dentro (por consequência) em ordem horária.

::: warning 🚧 TODO
Cubo com as faces transparentes, triângulo ABC + normal em uma das faces, slider
pra rotacionar.
:::

Se o triângulo é formado pelos pontos A, B e C, tenho dois jeitos de avançar
partindo de A:

```text
A
|\
| \
B--C
```

- sentido anti-horário → A -> B -> C
- sentido horário → A -> C -> B

Se a gente olhar pro mesmo triângulo de trás (espelhado na horizontal) os
sentidos das travessias se invertem:

```text
    A
   /|
  / |
 /  |
C---B
```

Considerando um triângulo ABC achatado no plano XY, o produto vetorial entre as
arestas AB e AC dá um vetor perpendicular às duas apontando "pra frente" (pra
fora da tela) ou "pra trás" (pra dentro da tela), dependendo da ordem das
arestas:

```text
A = (0, 1, 0)        AB = B - A = (0, -1, 0)
B = (0, 0, 0)        AC = C - A = (1, -1, 0)
C = (1, 0, 0)
```

```text
AB × AC = ((-1 * 0) - (0 * -1), (0 * 1) - (0 * 0), (0 * -1) - (-1 * 1)) = (0, 0,  1)
AC × AB = ((-1 * 0) - (0 * -1), (0 * 0) - (1 * 0), (1 * -1) - (-1 * 0)) = (0, 0, -1)
```

Ou seja: `AB × AC` dá uma normal apontando pra `+z`, e inverter a ordem das
arestas dá uma normal apontando pra `-z`.

Note que `A -> B -> C` é justamente o sentido anti-horário, então vértices
informados em sentido anti-horário dão `z` positivo — um vetor saindo da tela.

Note também que a edge function que a gente já usa é exatamente a fórmula do
componente `z` de um produto vetorial entre dois vetores 3D:

```text
u.x * v.y - u.y * v.x
```

Então a gente pode usar o sinal da área já calculada pela edge function pra
saber qual lado do triângulo projetado em 2D está virado para a câmera.
