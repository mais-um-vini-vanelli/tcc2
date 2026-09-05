# Ray tracing — teoria

As explicações longas do ray tracer, na ordem em que o pipeline as usa. Os
pontos do código que dependem de cada uma apontam pra cá com um comentário
`// Teoria: raytracer.md#secao`.

## A pipeline

O que o ray tracer faz hoje (M0.5), na ordem, e as seções que explicam cada
estágio:

```text
-> um raio primário por pixel      (Da tela para a cena)
-> o raio vai pro espaço local     (transform.md: Desfazendo,
   de cada objeto, pela inversa     O raio vai para o espaço local)
   do transform
-> teste contra cada primitiva     (O raio e a esfera)
   da cena, por força bruta
-> fica o acerto mais próximo,
   entre near e far
-> a cor é o albedo do material
```

`Triangle`, `Mesh` e `Cube` ainda são ignorados, e não há sombra, reflexo nem
BVH — isso é M3 e M4.

O contraste com o rasterizador está logo no primeiro estágio, e é o motivo de os
dois modos existirem na mesma API: **o rasterizador projeta a geometria *para* a
tela; o ray tracer parte da tela *para* a cena**. Um percorre objetos e descobre
que pixels eles cobrem; o outro percorre pixels e descobre que objetos eles
encontram.

O mesmo contraste reaparece no segundo estágio, e é o assunto de
[transform.md](transform.md): o rasterizador leva a geometria até o mundo, o ray
tracer traz o raio até a geometria. É por isso que a esfera daqui vira elipsoide
sob escala não uniforme sem ser tesselada.

## Da tela para a cena

Como nasce o raio primário de um pixel `(x, y)`.

### Do pixel para NDC

O pixel é um índice inteiro, mas o raio tem que passar pelo *centro* dele — daí
o meio pixel somado antes de qualquer coisa (o pixel 1,1 tem centro em 1.5,
1.5).

Depois as coordenadas viram NDC, `[-1, 1]` na largura e na altura. É o caminho
inverso do "NDC para pixel" do rasterizador.

Exemplo com 320x240, no eixo x:

```text
operação	| faixa depois dela
x             0 … 320
/ largura     0 … 1
* 2           0 … 2
- 1          -1 … 1
```

No eixo y tem a inversão de sentido: y cresce pra baixo no buffer e pra cima em
NDC, então em vez de subtrair 1 no fim a gente tira o valor de 1.

```text
y             0 … 240
/ altura      0 … 1
* 2           0 … 2
1 - y        -1 … 1  (invertido)

y = 2 (base da imagem) -> 1-2 = -1
y = 1 (centro)         -> 1-1 =  0
y = 0 (topo da imagem) -> 1-0 =  1
```

Daqui pra frente `u` e `v` são essas duas coordenadas normalizadas, e o que muda
entre os dois modos de câmera é o que se faz com elas.

### Perspectiva: a tela virtual

Imagine uma tela virtual a 1 unidade pra frente da câmera. Todos os raios saem
do mesmo ponto (a posição da câmera) e passam pelo ponto dessa tela que
corresponde ao pixel.

O tamanho dessa tela vem do fov. Dividindo o fov por 2 acha-se o ângulo entre o
topo e o eixo de visão (ou entre a base e o eixo, mesmo valor); a tangente desse
ângulo diz *quanto a altura aumenta a cada unidade que se avança em z*, já que
`tan() = oposto/adjacente`. Como a tela está a 1 unidade, a meia-altura dela é a
própria tangente.

```text
                        ^  topo da tela
                      / |
                    /   |
      fov/2       /     |  meia_altura = tan(fov/2) * 1
câmera  ) - - - - - - - +  centro da tela
          \       \     |
            \       \   |
              \       \ |
                        v  base da tela
      |<---- 1.0 ------>|
```

Pensando em 2d: `u` e `v` variam entre -1 e 1 conforme a função é chamada pra
cada pixel da imagem. Multiplicando por `meia_altura`, o pixel com `v = 1` cai
em `meia_altura` no plano virtual, o com `v = -1` cai em `-meia_altura`, e os
intermediários interpolam linearmente. Multiplicar esse valor pelo vetor `cima`
da câmera dá o deslocamento no eixo vertical.

No eixo horizontal entra o aspect, porque o fov guardado é o vertical:

```text
aspect = W / H  ->  W = H × aspect

tan(fov_h / 2) = W / z                     (definição do fov horizontal)
               = (H × aspect) / z          (porque W = H × aspect)
               = (H / z) × aspect          (só reagrupando a multiplicação)
               = tan(fov_v / 2) × aspect   (porque H/z é o tan vertical)
```

Então o deslocamento horizontal é `direita * (u * meia_altura * aspect)`.

Somando os três deslocamentos — 1 unidade pra frente, o vertical e o horizontal
— sai um único vetor apontando da câmera pro pixel: a direção do raio. A origem
é a posição da câmera, igual pra todos os pixels.

### Ortográfica: a origem é que varia

Na projeção ortográfica é o contrário: a **direção** é a mesma em todos os
pixels (o vetor `frente` da câmera) e a **origem** é que muda, alinhada com o
pixel. É a diferença entre um cone de raios saindo de um ponto e um feixe de
raios paralelos.

`u` e `v` vão de -1 a 1 (tamanho 2, uma unidade pra cada lado), mas o volume de
visão é `width` x `height` — metade pra cada lado em cada eixo. Então multiplicar
`u` por `width * 0.5` e `v` por `height * 0.5` converte a coordenada normalizada
no deslocamento em unidades de mundo.

Exemplo com `width = 320`:

```text
 u  | u * width * 0.5
-1    -160 (borda esquerda)
 0       0 (centro)
+1    +160 (borda direita)
```

Esse valor é a coordenada 2d do pixel em espaço de câmera, então ainda precisa
ser multiplicado pelos vetores `direita` e `cima`, que carregam a orientação:

```text
origem = position + direita * (u * width * 0.5) + cima * (v * height * 0.5)
```

## O raio e a esfera

A interseção raio-esfera, que é a conta que o rasterizador não tem como fazer —
ele precisa aproximar a mesma esfera por triângulos (ver
`rasterizer.md#a-pipeline`).

### Montando a equação

A esfera é o conjunto de pontos P com `|P − centro| = raio`, ou
`|P − centro|² = raio²`: distância até o centro igual ao raio. O ponto a uma
distância `t` ao longo do raio é `P' = origem + direção * t` (`Ray::at()`).

Substituindo P por P' na equação da esfera:

```text
  |(origem + t*direção) − centro|² = raio²
> |(origem − centro) + t*direção|² = raio²
```

Duas identidades do produto escalar antes de continuar.

A fórmula geométrica é `u · v = |u| × |v| × cos(θ)`, com θ o ângulo entre u e v.
Quando os dois vetores são o mesmo, o ângulo é 0° e `cos(0°) = 1`, então
`v · v = |v| × |v| = |v|²`. Ou seja: **o produto escalar de um vetor com ele
mesmo é o comprimento ao quadrado** — que é justamente o que aparece na equação.

Chamando `oc = origem − centro`:

```text
  |oc + t*direção|² = raio²
> (oc + t*direção) · (oc + t*direção) = raio²
```

E mais três propriedades:

```text
P1 distributiva              -> u · (v + w)  = u·v + u·w
P2 comutativa                -> u · v        = v · u
P3 escalar (k) sai para fora -> u · (k·v)    = k (u·v)
```

Abrindo o lado esquerdo com `A = oc` e `B = t·D`:

```text
(A + B)·(A + B) = A·A + A·B + B·A + B·B

1)  A·A  =  oc · oc
2)  A·B  =  oc · (t·D)
3)  B·A  =  (t·D) · oc
4)  B·B  =  (t·D) · (t·D)
```

2 e 3 são iguais por P2, e por P3 viram `t (oc · D)` — daí o fator 2. Juntando
tudo:

```text
  (oc + t*direção) · (oc + t*direção) = raio²
> oc · oc + 2t(oc · direção) + t² (direção · direção) = raio²
  1)        2) + 3)            4)
> oc · oc + 2t(oc · direção) + t² (direção · direção) - raio² = 0
> t² (direção · direção) + 2t(oc · direção) + (oc · oc - raio²) = 0
```

### Resolvendo

Organizando na forma `at² + bt + c = 0`, pra resolver com Bhaskara:

```text
a = (direção · direção) = 1   (a direção é normalizada, e v·v = |v|²)
b = 2(oc · direção)
c = oc · oc - raio²
```

```text
t = (−b ± √(b²−4ac)) / 2a
```

O discriminante já responde a pergunta geométrica antes das raízes:

```text
b² − 4ac  < 0    o raio passa longe: não existe P na esfera
          = 0    o raio tangencia a esfera (uma raiz)
          > 0    o raio atravessa: entra e sai (duas raízes)
```

Com duas raízes, `t0` (com o `−√`) é sempre a menor, porque a raiz quadrada é
positiva: `t0` é onde o raio entra na esfera e `t1` onde sai. O que interessa é a
superfície mais próxima, então fica a menor das duas — mas só entre as
positivas, senão a câmera "veria" uma esfera que está atrás dela. Por isso o
código pega a primeira de `[t0, t1]` que passa do zero: se `t0` é negativo e `t1`
não, a câmera está dentro da esfera.
