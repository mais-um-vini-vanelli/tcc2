# Interseções

As contas que decidem se um raio acerta uma primitiva, e o que o acerto carrega
de volta.

## O raio e a esfera

O ray tracer consegue calcular a interseção raio-esfera sem precisar aproximar o
formato real da esfera com triângulos, como o rasterizador faz.

### Montando a equação

A esfera é o conjunto de pontos P com `|P - centro| = raio`, ou
`|P - centro|² = raio²`: distância até o centro igual ao raio. O ponto a uma
distância `t` ao longo do raio é `P' = origem + direção * t` (`Ray::at()`).

::: warning 🚧 TODO Médio
Desenho 3D da esfera, com P, centro, vetor P - centro, raio.
:::

Substituindo P por P' na equação da esfera:

```text
  |(origem + t * direção) - centro|² = raio²
> |(origem - centro) + t * direção|² = raio²
```

A fórmula geométrica do produto escalar é `u · v = |u| * |v| * cos(θ)`, com θ o 
ângulo entre u e v. Quando os dois vetores são o mesmo, o ângulo é 0° e `cos(0°) = 1`, então
`v · v = |v| * |v| * 1 = |v|²`. Ou seja: **o produto escalar de um vetor com ele
mesmo é o comprimento ao quadrado**.

```text
  |oc + t * direção|² = raio²                      # trocando origem - centro = oc
> (oc + t * direção) · (oc + t * direção) = raio²  # usando v · v = |v|²
```

Três propriedades do produto escalar:

```text
P1 distributiva              -> u · (v + w)  = u · v + u · w
P2 comutativa                -> u · v        = v · u
P3 escalar (k) sai para fora -> u · (k * v)    = k * (u · v)
```

Abrindo o lado esquerdo com `A = oc` e `B = t * D`:

```text
(A + B)·(A + B) = A · A + A · B + B · A + B · B

1)  A · A  =  oc · oc
2)  A · B  =  oc · (t * D)
3)  B · A  =  (t * D) · oc
4)  B · B  =  (t * D) · (t * D)
```

2 e 3 são iguais (P2), e podem virar `t (oc · D)` usando P3. Juntando tudo:

```text
(oc + t * direção) · (oc + t * direção)                              = raio²
oc · oc + t(oc · direção) + t(oc · direção) + t² (direção · direção) = raio²
oc · oc + 2t(oc · direção) + t² (direção · direção)                  = raio²
oc · oc + 2t(oc · direção) + t² (direção · direção) - raio²          = 0
t² (direção · direção) + 2t(oc · direção) + (oc · oc - raio²)        = 0
```

### Resolvendo por Bhaskara

Organizando na forma `at² + bt + c = 0`:

```text
a = (direção · direção) = 1  # a direção é normalizada, e v · v = |v|²
b = 2(oc · direção)
c = oc · oc - raio²
```

```text
t = (-b ± √(b²-4ac)) / 2a
```

O discriminante tem um significado geométrico:

```text
b² - 4ac  < 0    o raio passa longe: não existe um ponto em que o raio intercepta a esfera
          = 0    o raio toca um único ponto da esfera (uma raiz)
          > 0    o raio atravessa a esfera: entra e sai (duas raízes)
```

Com duas raízes, `t0` (com o `-√`) é sempre a menor, porque a raiz quadrada é
positiva: `t0` é onde o raio entra na esfera e `t1` onde sai. O que interessa é
a superfície mais próxima, então a primeira interseção é a menor das duas.
Ignoramos `t` negativo, porque um acerto atrás da origem do raio está atrás da câmera.
Se `t0` é negativo e `t1` não, a câmera está dentro da esfera.


### O uv do acerto

Achado o `t`, a normal no ponto é `(P - centro) / raio` — na esfera, a direção
do centro para a superfície *é* a normal. Falta as coordenadas de textura, e elas
saem da mesma parametrização que a tesselação (`Primitive::tessellate`) usa para
gerar os vértices, só que percorrida ao contrário: lá o par (θ, φ) gera a
posição, aqui a posição devolve o par.

```text
Ângulos da esfera em função de u e v:

θ = u · π    0 no polo norte, π no polo sul
φ = v · τ    dá a volta no equador

Normal e uv de um ponto em função dos ângulos da esfera:

normal = (sen θ · cos φ,  cos θ,  sen θ · sen φ)
uv     = (θ/π, φ/τ)
```

O y da normal é `cos θ`, então `θ = acos(y)` e `u = θ/π`.

Para φ, olhando a esfera de cima (plano xz):

```text
              +z
               │
               │     ● (x, z)
               │    ╱
               │   ╱  r = sen θ
               │  ╱
               │ ╱ φ
    ───────────┼───────────── +x
               │
```

::: warning 🚧 TODO Médio
Trocar por um chart.
:::

```text
x = sen θ · cos φ = r · cos φ
z = sen θ · sen φ = r · sen φ

φ = atan2(r · sen φ, r · cos φ) = atan2(sen φ, cos φ)
```

O `r` pode ser ignorado na chamada de `atan2`: multiplicar os dois catetos de um
triângulo retângulo pelo mesmo fator não muda os ângulos internos.

Sobra um detalhe de intervalo. `atan2` devolve em `-π..π`, então os ângulos acima de 180º voltam
como um valor negativo de `0..-π` ao invés de seguir até 2π como se espera das coordenadas uv.
Ex: `(x = 0, z = -1)` volta como `-π/2` ao invés de `3π/2`. Então somamos 1 aos negativos para
corrigir pro intervalo esperado.

## O raio e o triângulo

A outra metade da geometria. A esfera tem equação própria; o triângulo é
resolvido por Möller e Trumbore (1997). Ele devolve, no mesmo cálculo, o `t` **e** 
as coordenadas baricêntricas do acerto.

### O sistema

Ancore-se em `p0` e olhe para as duas arestas que saem dele como se fossem
eixos:

```text
        p2
        ●
       ╱ ╲
 E2   ╱   ╲
     ╱     ╲
    ●───────●
   p0   E1   p1
```

::: warning 🚧 TODO
Desenho 3D disso.
:::

`E1` e `E2` não são perpendiculares nem unitários, mas isso não importa:
qualquer ponto do plano do triângulo pode ser escrito como `p0 + u * E1 + v * E2` para 
exatamente um par `(u, v)`. É um sistema de coordenadas com origem em `p0`.

Podemos visualizar os pontos do triângulo no plano UV:

```text
 v
 │
1┤● p2
 │  ╲
 │   ╲
 │    ╲
 │     ╲
0┤●──────●────
 p0      p1   u
 0       1
```

::: warning 🚧 TODO Alto
Desenho 2D disso, com u e v extendendo até -0,5.
:::

Um ponto está dentro do triângulo se:

```text
u ≥ 0
v ≥ 0
u + v ≤ 1   não passou da aresta P1->P2
```

Um raio é `O + t * D`, então pra achar o ponto de interseção com o triângulo
usamos:

```text
O + t * D = p0 + u * E1 + v * E2
```

Isolando as incógnitas à esquerda e definindo `T = O - p0`:

```text
-t * D + u * E1 + v * E2 = T
```

Isso é uma equação de vetores, então podemos expandir pra equações de escalares:

```text
-t * Dx + u * E1x + v * E2x = Tx
-t * Dy + u * E1y + v * E2y = Ty
-t * Dz + u * E1z + v * E2z = Tz
```

Vamos resolver isso por Cramer, então precisa reorganizar como multiplicação de
matrizes:

```text
┌                 ┐ ┌   ┐   ┌    ┐
│ -Dx   E1x   E2x │ │ t │   │ Tx │
│ -Dy   E1y   E2y │ │ u │ = │ Ty │
│ -Dz   E1z   E2z │ │ v │   │ Tz │
└                 ┘ └   ┘   └    ┘
```

### Resolvendo por Cramer

Organizamos as operações de Cramer:

```text
det(M) = det(-D, E1, E2)
t = det( T, E1, E2) / det(M)
u = det(-D,  T, E2) / det(M)
v = det(-D, E1,  T) / det(M)
```

Agora vamos reorganizar as fórmulas pra reaproveitar valores intermediários.

Identidades relevantes:

```text
1) det(a, b, c) = a · (b × c)
2) det(a, b, c) = det(b, c, a) = det(c, a, b)  # podemos rotacionar os termos
```

Com isso, vamos definir `P = D × E2` e `Q = T × E1` e reorganizar os cálculos de
determinante:

```text
det(M) = det(-D, E1, E2)
       = -det(D, E1, E2) # usando det(-1·a, b, c) = -1 · det(a, b, c)
       = -det(E1, E2, D) # usando P2
       = -E1 · (E2 × D)  # usando P1
       =  E1 · (D × E2)  # usando E2×D = -(D×E2)
       =  E1 · P         # substituindo P = D × E2

t:   det(T, E1, E2)
   = det(E2, T, E1) # usando P2
   = E2 · (T × E1)  # usando P1
   = E2 · Q         # substituindo Q = T × E1

u:   det(-D, T, E2)
  = -det(T, E2, D)
  = -T · (E2 × D)
  =  T · (D × E2)
  =  T · P

v:   det(-D, E1, T)
   = -det(D, E1, T)
   = -D · (E1 × T)
   =  D · (T × E1)
   =  D · Q
```

### O que cada teste rejeita

`det(M)` tem um significado geométrico: é o volume com sinal de `E1 x D x E2`.

::: warning 🚧 TODO Médio
Desenho 3D do volume E1 x D x E2.
:::

Volume 0 significa que os 3 estão no mesmo plano. Como `E1` e `E2` são as duas
direções que geram o plano do triângulo, `D` ser coplanar com eles quer dizer
que `D` não tem nenhuma componente saindo do plano:

```text
det ≠ 0                          det = 0

    D
     ╲                           ────D────→
  ════╳══════                     ═══════════

 o raio sai do plano           o raio fica no plano (passa paralelo ao triângulo)
```

`det(M) = 0` significa que um desses aconteceu:

- O raio é paralelo ao plano e está fora dele (não encosta no triângulo)
- O raio está deitado dentro do plano (triângulo tem espessura 0, então não
  encosta também)
- O triângulo é degenerado, com `E1` e `E2` paralelos e área zero (não encosta
  também)

Então não houve interseção e o triângulo pode ser descartado.

Então as condições em que não houve interseção são:

```text
det ≈ 0        
u < 0    
v < 0    
u + v > 1
t ≤ 0      # o triângulo está atrás da origem do raio, então está atrás da câmera
```

### Os dois sinais do determinante passam

O sinal de `det(M)` diz de que lado o raio chega — é a mesma informação que o
sinal da área em tela dá ao rasterizador ([Backface
culling](/rasterizacao/triangulo#backface-culling)).
O rasterizador pode descartar a face de trás, mas o ray tracer precisa manter o acerto.

O motivo é o raio de refração: ele entra no objeto e precisa achar a face de
dentro para sair. Com backface culling o vidro não teria saída, e o raio
seguiria para sempre dentro dele.

### E as baricêntricas saem de graça

`u` e `v` são exatamente os pesos de `p1` e `p2`, e o de `p0` é `1 - u - v`. É
com eles que a normal e o uv do acerto são interpolados — e, diferente do
rasterizador, **sem correção de perspectiva nenhuma**: aqui os pesos foram
medidos na geometria em 3D, então não houve distorção de perspectiva.

## O raio e o cubo

O cubo também tem equação própria. Levando o raio pro cubo com o centro na
origem, as seis faces viram os planos `x = ±h`, `y = ±h` e `z = ±h`.

Cada face tem a normal, e pra onde apontam a direita e o "pra baixo" de quem olha
a face de fora:

```text
normal         direita        baixo
( 1,  0,  0)   ( 0, 0, -1)    (0, -1,  0)
( 0,  1,  0)   ( 1, 0,  0)    (0,  0,  1)
( 0,  0,  1)   ( 1, 0,  0)    (0, -1,  0)
(-1,  0,  0)   ( 0, 0,  1)    (0, -1,  0)
( 0, -1,  0)   ( 1, 0,  0)    (0,  0, -1)
( 0,  0, -1)   (-1, 0,  0)    (0, -1,  0)
```

### O t de cada face

O plano da face é `n · ponto = h`. O raio é `p(t) = o + t·d`. O raio cruza a face
quando `n · (o + t·d) = h -> n · o + t·(n · d) = h`, então dá de descobrir `t` com: `t = (h − n · o) / (n · d)`.

### O acerto só vale dentro do quadrado

O plano é infinito, mas a face não. `(a, b)` são as coordenadas, em meias
arestas, do ponto em que o raio cruza o plano da face, e o acerto só vale dentro
do quadrado `a, b ∈ [-1, 1]`:

```text
p = o + t·d

a = (p · direita) / h
b = (p · baixo)   / h
```

Das faces que passam os dois testes, vale a mais próxima.

### O infinito e o NaN são de propósito

Raio paralelo ao plano dá `n · d = 0`, e a divisão dá ±∞ (ou NaN, com a origem em
cima do plano). Como no [teste dos slabs](/raytracing/bvh#o-teste-dos-slabs),
isso é proposital: um `t` infinito leva o ponto pro infinito e ele cai fora do
quadrado, e NaN falha o `t > 0`.
