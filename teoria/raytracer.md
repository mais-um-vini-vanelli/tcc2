# Ray tracing — teoria

As explicações longas do ray tracer, na ordem em que o pipeline as usa. Os
pontos do código que dependem de cada uma apontam pra cá com um comentário
`// Teoria: raytracer.md#secao`.

## A pipeline

O que o ray tracer faz, na ordem, e as seções que explicam cada estágio:

```text
-> um raio primário por pixel      (Da tela para a cena)
-> o raio vai pro espaço local     (transform.md: Desfazendo,
   de cada objeto, pela inversa     O raio vai para o espaço local)
   do transform
-> teste contra as primitivas      (O raio e a esfera,
   que a BVH escolhe                O raio e o triângulo,
                                    bvh.md)
-> fica o acerto mais próximo,
   entre near e far
-> a cor direta do acerto          (O termo direto de Whitted)
   |
   +-> um raio de sombra por luz   (O raio de sombra)
-> mais o que chega pelo espelho   (O raio refletido,
   |                                O peso do raio de espelho)
   +-> tudo de novo, a partir      (A recursão de Whitted)
       do ponto de acerto
-> mais o que atravessa o objeto   (O raio refratado)
   |
   +-> tudo de novo
```

Com a BVH desligada, todo raio testa todo objeto da cena. Ligada, ela não muda
nada acima — a aceleração decide *quais* primitivos testar, não como. A
explicação dela está em [bvh.md](bvh.md).

O contraste com o rasterizador está logo no primeiro estágio, e é o motivo de os
dois modos existirem na mesma API: **o rasterizador projeta a geometria *para* a
tela; o ray tracer parte da tela *para* a cena**. Um percorre objetos e descobre
que pixels eles cobrem; o outro percorre pixels e descobre que objetos eles
encontram.

O mesmo contraste reaparece no segundo estágio, e é o assunto de
[transform.md](transform.md): o rasterizador leva a geometria até o mundo, o ray
tracer traz o raio até a geometria. É por isso que a esfera daqui vira elipsoide
sob escala não uniforme sem ser tesselada.

E reaparece uma terceira vez, agora com peso: da metade da lista para baixo
**não existe equivalente no rasterizador**. Sombra, reflexo e refração saem de
graça de "seguir um raio e perguntar o que ele encontra", e o rasterizador, que
nunca segue um raio, precisa de um algoritmo diferente para cada um deles (mapa
de sombra, mapa de ambiente, um segundo render do ponto de vista do espelho).

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

## O raio e o triângulo

A outra metade da geometria. A esfera tem equação própria; o triângulo é
resolvido por Möller e Trumbore (1997), que é o método que praticamente todo ray
tracer usa — e que devolve, no mesmo cálculo, o `t` **e** as coordenadas
baricêntricas do acerto.

### O sistema

Ancore-se em `p0` e olhe para as duas arestas que saem dele como se fossem eixos:

```text
        p2
        ●
       ╱ ╲
 E2   ╱   ╲
     ╱     ╲
    ●───────●
   p0   E1   p1
```

`E1` e `E2` não são perpendiculares nem unitários, mas isso não importa:
qualquer ponto do plano do triângulo é `p0 + u·E1 + v·E2` para exatamente um par
`(u, v)`. É um sistema de coordenadas oblíquo, com origem em `p0`.

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

Um ponto está dentro do triângulo se:

```text
u ≥ 0
v ≥ 0
u + v ≤ 1    (não passou da aresta p1p2)
```

Um raio é `O + t·D`, então pra achar o ponto de interseção com o triângulo
usamos:

```text
O + t·D = p0 + u·E1 + v·E2
```

Isolando as incógnitas à esquerda e definindo `T = O − p0`:

```text
−t·D + u·E1 + v·E2 = T
```

Isso é uma equação de vetores, então podemos expandir pra equações de escalares:

```text
−t·Dx + u·E1x + v·E2x = Tx
−t·Dy + u·E1y + v·E2y = Ty
−t·Dz + u·E1z + v·E2z = Tz
```

Vamos resolver isso por Cramer, então precisa reorganizar como multiplicação de
matrizes:

```text
┌                 ┐ ┌   ┐   ┌    ┐
│ −Dx   E1x   E2x │ │ t │   │ Tx │
│ −Dy   E1y   E2y │ │ u │ = │ Ty │
│ −Dz   E1z   E2z │ │ v │   │ Tz │
└                 ┘ └   ┘   └    ┘
```

### Resolvendo por Cramer

Organizamos as operações de Cramer:

```text
det(M) = det(−D, E1, E2)
t = det( T, E1, E2) / det(M)
u = det(−D,  T, E2) / det(M)
v = det(−D, E1,  T) / det(M)
```

E agora reorganizamos pra precisar calcular menos coisa. Identidades:

```text
1) det(a, b, c) = a · (b × c)
2) det(a, b, c) = det(b, c, a) = det(c, a, b)
```

Com isso, vamos definir `P = D × E2` e `Q = T × E1` e reorganizar os cálculos de
determinante:

```text
det(M) = det(−D, E1, E2)
       = −det(D, E1, E2) <-  det(-1·a, b, c) = -1 · det(a, b, c)
       = −det(E1, E2, D) <-  Prop 2
       = −E1 · (E2 × D)  <-  Prop 1
       =  E1 · (D × E2)  <-  E2×D = −(D×E2)
       =  E1 · P         <-  Substituindo P

t:   det(T, E1, E2)
   = det(E2, T, E1) <-  Prop 2
   = E2 · (T × E1)  <-  Prop 1
   = E2 · Q         <-  Substituindo Q

u:   det(−D, T, E2)
  = −det(T, E2, D)
  = −T · (E2 × D)
  =  T · (D × E2)
  =  T · P

v:   det(−D, E1, T)
   = −det(D, E1, T)
   = −D · (E1 × T)
   =  D · (T × E1)
   =  D · Q
```

### O que cada teste rejeita

`det(M)` tem um significado especial: é o volume com sinal de `E1 x D x E2`.
Volume 0 significa que os 3 estão no mesmo plano. Como `E1` e `E2` são as duas
direções que geram o plano do triângulo, `D` ser coplanar com eles quer dizer que
`D` não tem nenhuma componente saindo do plano:

```text
det ≠ 0                          det = 0

    D
     ╲                           ────D────→
  ════╳══════                     ═══════════

 o raio sai do plano           o raio fica no plano (passa paralelo ao triângulo)
```

`det(M) = 0` significa que um desses aconteceu:

- O raio é paralelo ao plano e está fora dele (não encosta no triângulo)
- O raio está deitado dentro do plano (triângulo tem espessura 0, então não encosta também)
- O triângulo é degenerado, com `E1` e `E2` paralelos e área zero (não encosta também)

Então não houve interseção e o triângulo pode ser descartado.

```text
det ≈ 0        o raio corre paralelo ao plano do triângulo
u < 0, u > 1   o acerto no plano caiu fora, do lado da aresta p0p2
v < 0          idem, do lado da aresta p0p1
u + v > 1      caiu fora pela hipotenusa p1p2
t ≤ 0          o triângulo está atrás da origem do raio
```

Cada rejeição é uma saída antecipada: quem falha em `u` nem calcula `Q`.

### Os dois sinais do determinante passam

O sinal de `det(M)` diz de que lado o raio chega — é a mesma informação que o
sinal da área em tela dá ao rasterizador (`rasterizer.md#backface-culling`). Lá
ela serve para descartar a face de trás; aqui **não** se descarta nada.

O motivo é o raio de refração: ele entra no objeto e precisa achar a face de
dentro para sair. Com backface culling o vidro não teria saída, e o raio seguiria
para sempre dentro dele. Um ray tracer que só faz sombra e reflexo pode culled o
verso e ganhar velocidade; um que refrata, não.

### E as baricêntricas saem de graça

`u` e `v` são exatamente os pesos de `p1` e `p2`, e o de `p0` é `1 − u − v`. É
com eles que a normal e o uv do acerto são interpolados — e, diferente do
rasterizador, **sem correção de perspectiva nenhuma**: aqui os pesos foram
medidos na geometria em 3D, não numa projeção em tela. O rasterizador só chega
neste mesmo número depois de interpolar `1/z` e desfazer a divisão
(`rasterizer.md#pesos-sob-perspectiva`); o ray tracer nunca dividiu por `z`,
então não tem o que desfazer.

## O uv do acerto

Achado o `t`, a normal no ponto é `(P − centro) / raio` — na esfera, a direção
do centro para a superfície *é* a normal. Falta a coordenada de textura, e ela
sai da mesma parametrização que a tesselação (`Primitive::tessellate`) usa para
gerar os vértices, só que percorrida ao contrário: lá o par (θ, φ) gera a
posição, aqui a posição devolve o par. É o mesmo contraste do resto do capítulo
— aproximar por triângulos ou resolver direto —, agora na coordenada de
textura.

```text
θ = u · π    0 no polo norte, π no polo sul
φ = v · τ    dá a volta no equador

normal = (sen θ · cos φ,  cos θ,  sen θ · sen φ)
uv     = (v, u)
```

O y da normal já é `cos θ` sozinho, então θ é imediato: `θ = acos(y)`, dividido
por π para cair em 0..1.

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

```text
x = sen θ · cos φ = r · cos φ
z = sen θ · sen φ = r · sen φ

φ = atan2(sen φ, cos φ) = atan2(r · sen φ, r · cos φ)
```

O `r` pode ser ignorado na chamada de `atan2`: multiplicar os dois catetos de um
triângulo retângulo pelo mesmo fator não muda os ângulos internos.

Sobra um detalhe de intervalo. `atan2` devolve em `−π..π`, que depois da divisão
por τ vira `−0.5..0.5` — o meridiano de trás da esfera (`x = 0`, `z = −1`, ou
seja φ = 3π/2) aparece como `−0.25`. Somar 1 aos negativos recoloca tudo em
`0..1`, que é o que a amostragem de textura espera.

## O termo direto de Whitted

Whitted (1979) escreve a luz que sai de um ponto `p` na direção `v` como três
parcelas somadas:

```text
L(p, v) = Σ_luzes  V(p, luz) · reflectancia(n, v, l) · radiancia(luz, p)   <- direto
        +  F(v, n) · L(p, r_reflexão)                                      <- recursão
        +  kt      · L(p, r_refração)                                      <- recursão
```

A primeira linha é a **iluminação direta**: percorre as luzes da cena e pergunta,
para cada uma, quanto dela chega ao ponto e quanto disso volta na direção de
quem olha. É literalmente o que `shading::shade` faz, e é o único termo que o
rasterizador também consegue calcular — por isso ele mora em `shading.rs`, o
módulo compartilhado, e não aqui.

As outras duas linhas não têm equivalente no rasterizador, e são o assunto da
seção seguinte.

Três detalhes de quem chama:

- **`v` sai do raio, não da câmera.** No acerto primário `v` aponta para a
  câmera, e daria na mesma pedir isso à `Camera`. Mas o ponto que aparece
  *dentro de um espelho* é olhado pelo espelho, e o `v` dele é o inverso do raio
  refletido. Como o raio sempre carrega essa informação — inclusive a diferença
  entre perspectiva e ortográfica, que já foi resolvida quando ele nasceu —,
  `shade` recebe `v` pronto e não precisa saber de câmera nenhuma.
- **`V` é um parâmetro, não uma conta.** É o fator de visibilidade: 1 se a luz
  chega inteira, 0 se algo está na frente. O ray tracer resolve com um raio de
  sombra; o rasterizador, quando tiver, vai resolver com um mapa de sombra
  consultado por textura. Mesmo termo da fórmula, dois algoritmos que não se
  parecem em nada — e é por isso que ele entra em `shade` como função passada
  pelo chamador, em vez de cada modo reescrever o laço das luzes só para
  multiplicar o seu.
- **Luz por trás nem chega a perguntar.** Se `n · l ≤ 0` a luz está do outro
  lado da superfície e não ilumina de jeito nenhum. O teste vem antes do `V`
  justamente para não gastar um raio de sombra respondendo uma pergunta que a
  geometria já respondeu.

## A recursão de Whitted

O que faz do ray tracer um ray tracer, e não um rasterizador que percorre pixels
em vez de triângulos.

Achado o ponto de acerto, ele é tratado como uma câmera nova: dele saem mais
raios, e o que esses raios encontram volta somado à cor do ponto. Reflexo é
"para onde eu olharia se este ponto fosse um espelho"; refração é "por onde a
luz sairia se eu atravessasse este ponto". Cada um é uma chamada recursiva da
mesma função que resolveu o raio primário.

```text
trace(raio, profundidade):
    acerto = o primeiro objeto no caminho     -> nada? devolve o fundo
    cor    = termo direto no acerto           (O termo direto de Whitted)

    se profundidade == max_depth: acabou

    cor += F  · trace(raio refletido, profundidade + 1)
    cor += kt · trace(raio refratado, profundidade + 1)
```

`max_depth` é o que impede a recursão de não acabar: dois espelhos de frente um
para o outro geram uma cadeia infinita de reflexos, e a esfera de vidro gera uma
de refrações. O parâmetro é um toggle de caixa branca — subir de 0 até 5 mostra
um reflexo aparecendo dentro do outro, e mostra também onde ele para de valer o
custo. Com `max_depth = 0` a imagem é iluminação direta pura, que é o que o
rasterizador desenha (mais a sombra, que o raio de sombra ainda dá).

O custo é a parte a olhar: cada nível pode dobrar o número de raios, e cada raio
testa a cena inteira. É o argumento do BVH de M3 escrito em uma linha.

### Somar o especular e o reflexo não é contar duas vezes

O termo direto já tem um brilho especular, e agora entra também um raio de
espelho. Parece a mesma luz somada duas vezes, e não é: `Light::Point` e
`Light::Directional` são pontos e direções matemáticas, de área zero. Nenhum raio
refletido jamais acerta uma delas. O especular direto é o brilho **da luz**; o
raio de espelho traz o reflexo **dos objetos**. São conjuntos disjuntos, e é
assim que Whitted escreve.

Isso deixa de valer no dia em que existir luz com área — aí a mesma luz apareceria
nas duas parcelas, e uma delas teria que sair.

## O raio de sombra

O `V` da fórmula, do lado do ray tracer, é a pergunta mais direta que se pode
fazer: **existe alguma coisa entre este ponto e esta luz?**

```text
             luz
              o
             /
            /  <- o raio de sombra: origem no ponto, direção para a luz
   ####### /
        \ /
         X          o cubo está no caminho: o ponto está na sombra
        / \
    ---p---------
```

A direção e o alcance saem do tipo da luz:

```text
Point        direção = (posição − p) normalizada,  alcance = |posição − p|
Directional  direção = −direction,                 alcance = infinito
Ambient      não tem direção: nada a testar, sempre visível
```

O alcance importa. Um objeto *atrás* da luz pontual não faz sombra nenhuma, e
sem limitar o `t` ao comprimento até a luz ele faria. Na direcional não existe
"atrás da luz": ela vem do infinito, e qualquer coisa no caminho bloqueia.

Duas observações sobre o que este teste **não** precisa:

- **Não precisa do acerto mais próximo.** Basta saber que existe um, e uma
  busca dedicada poderia parar na primeira interseção encontrada (*any-hit*).
  O código reusa a busca pelo mais próximo mesmo assim, de propósito — ver
  `bvh.md#a-sombra-so-precisaria-de-um`.
- **Não precisa de material.** Sombra em Whitted é binária: ou a luz chega
  inteira, ou não chega. Vidro projeta a mesma sombra preta que concreto — a
  sombra colorida por objeto transparente exige rastrear a transmitância pelo
  caminho, o que é outro modelo. O tipo de retorno é `f32` e não `bool` porque
  penumbra (várias amostras sobre uma luz com área) e o filtro do mapa de sombra
  do lado do rasterizador devolvem fração, não sim/não.

## O acne e o epsilon

O raio de sombra nasce exatamente em cima da superfície que o gerou. Em
aritmética exata ele sairia dali sem tocá-la de novo; em `f32`, o ponto de
partida cai meio ULP para dentro do objeto metade das vezes, e o objeto se
descobre bloqueando a própria luz. O resultado é o *shadow acne*: um chuvisco de
pixels pretos espalhado pela parte iluminada.

```text
sem deslocamento                  com deslocamento
                                        p'  (p + n·ε)
   ---p---------                     ---p---------
      ^                                 ^
      acerta a si mesmo em t ≈ 0        o t ≈ 0 já ficou para trás
```

A correção é empurrar a origem do raio secundário um `ε` ao longo da normal,
para o lado em que ele vai seguir — `+n` se o raio sai pela frente da
superfície, `−n` se ele entra no objeto (é o caso do raio de refração). O `t`
mínimo da busca também fica em `ε`, e as duas guardas juntas resolvem o problema
inteiro.

O valor é um compromisso conhecido: pequeno demais e o acne volta, grande demais
e a sombra se descola do objeto que a projeta (*peter-panning*). O certo escala
com o tamanho da cena, e escolher um número fixo é uma simplificação —
declarada.

## O raio refletido

A direção do espelho. Decompondo `d` na parte perpendicular à superfície (ao
longo de `n`) e na parte paralela a ela, refletir é inverter só a
perpendicular:

```text
      n
      ^      r
      |     /
      |    /        d·n é negativo (o raio chega contra a normal)
  d   |   /         (d·n)·n é a componente de d ao longo de n
   \  |  /
    \ | /           r = d − 2(d·n)·n
     \|/
  ----X--------
```

Tirar `(d·n)·n` uma vez zera a componente perpendicular; tirar duas vezes a
inverte. A componente paralela nunca é tocada, que é a definição de reflexão
especular: o ângulo de entrada é igual ao de saída, medidos da normal.

## O peso do raio de espelho

Quanto do que o espelho vê entra na cor do ponto. **Não** é um campo do
material: é o Fresnel, calculado a partir de `metallic` e `albedo`, e essa foi
uma decisão explícita de design (um `reflectivity` solto no material poderia
discordar do especular da BRDF, que já responde a mesma pergunta).

`f0` é a refletância de frente — a fração devolvida quando a luz chega
perpendicular:

```text
dielétrico (metallic = 0)   f0 = 0.04       4%, e branco: vidro, plástico, água
metal      (metallic = 1)   f0 = albedo     ouro reflete amarelo, cobre laranja
```

Os 4% não são chute: saem da equação de Fresnel para um índice de refração de
~1.5, que é onde quase todo dielétrico comum cai. E é o número que explica por
que a esfera de metal fica **preta** sem raio de espelho: metal não tem lóbulo
difuso (`kd = 1 − metallic = 0`), então tudo que ele mostra é reflexo. Sem
recursão, não há o que mostrar.

Schlick (1994) dá a variação com o ângulo em uma linha, no lugar da equação
completa:

```text
F(cos) = f0 + (1 − f0) · (1 − cos)^5
```

Com a luz de frente (`cos = 1`) sobra `f0`; raspando a superfície (`cos → 0`) o
termo vai a 1, e **todo** material vira espelho. É o efeito que se vê num piso
encerado olhando de longe, ou na água de um lago na altura dos olhos: o mesmo
material que reflete 4% de cima reflete quase tudo de lado.

A mesma dupla `f0`/Schlick é usada pelo especular da BRDF em `shading.rs` — o
peso do raio e a cor do brilho respondem à mesma pergunta física, e por isso
moram numa função só.

E o `F` que sai daqui não pesa só este raio: o que ele **não** reflete é o que
entra na superfície, e é dali que sai o peso do raio de refração — ver "O peso
do raio refratado".

## O limite do espelho de Whitted

Um raio refletido só faz sentido se a superfície reflete numa direção só. Numa
superfície áspera as microfacetas apontam para todo lado, e o reflexo é a média
de um cone inteiro de direções — o lóbulo que a BRDF de microfacetas descreve.
Amostrar esse cone exige dezenas de raios por acerto, e é isso que separa Whitted
de um *path tracer*.

A regra adotada: o raio de espelho é disparado enquanto `roughness < 0.1`, e
acima disso fica só o termo direto. A alternativa — mandar um raio só no centro
do lóbulo — daria um reflexo nítido em um material que a própria BRDF descreve
como borrado, o que é pior que não refletir.

A divergência é conteúdo, não defeito: o metal áspero é o caso em que **o modelo
de iluminação e o modelo de traçado discordam**, e ver os dois lado a lado é a
forma mais curta de explicar por que os renderizadores modernos trocaram Whitted
por amostragem.

## O raio refratado

A direção de quem atravessa. A lei de Snell relaciona os ângulos com a normal
nos dois meios pelos índices de refração:

```text
n1 · sen θ1 = n2 · sen θ2          n1, n2 = índices dos dois meios
                                   θ1, θ2 = ângulos com a normal
```

```text
            n ^
              |
        -d    |
          \   |
           \θ₁|
            \ |
    ----------+----------> û    (a superfície)
              | \
              |θ₂\
              |   \
              |    t
```

Com `η = n1/n2` e `cos θ1 = −d·n`:

```text
sin θ₂ = η · sin θ₁
```

Eleva tudo ao quadrado:

```text
sin²θ₂ = η² · sin²θ₁
```

Com pitágoras `sin²θ₁ = 1 − cos²θ₁`, então trocamos:

```text
sin²θ₂ = η² · (1 − cos²θ₁)
```

Se `sin²θ₂ > 1` é o caso de reflexão interna total (abaixo). Pra descobrir
`cos θ₂` usamos pitágoras:

```text
|t|² = (sin θ₂)² + (cos θ₂)²
cos θ₂ = √(1 - (sin θ₂)²)
```

Podemos decompor `d` e `t` em componente paralelo à normal + componente tangente
à superfície:

```text
d = sin θ₁ · û  +  cos θ₁ · -n
t = sin θ₂ · û  +  cos θ₂ · -n
```

Então:

```text
t = sin θ₂ · û  +  cos θ₂ · -n
t = (η ·  sin θ₁) · û  −  cos θ₂ · n
t =  η · (sin θ₁  · û) −  cos θ₂ · n
```

Podemos conseguir a componente tangente de `d` com
`d_tan = d - (componente paralela) = d - (cos θ₁ · -n)`; ou seja, removendo o
deslocamento na direção da normal e deixando apenas o deslocamento na direção da
tangente:

```text
t =  η · (d + cos θ₁·n) −  cos θ₂ · n
t =  η·d + η·cos θ₁·n   −  cos θ₂ · n
t =  η·d + (η·cos θ₁ − cos θ₂)·n
```

### Entrando e saindo

O mesmo código serve para os dois sentidos, e o que decide é o sinal de `−d·n`.
A normal da geometria sempre aponta para fora do objeto:

```text
−d·n > 0   o raio chega de fora:  η = 1/índice,   normal como está
−d·n < 0   o raio sai de dentro:  η = índice,     normal invertida
```

Esquecer essa troca é o erro clássico: a esfera de vidro fica com uma casca
transparente e um interior que não se comporta como vidro, porque metade das
refrações usou o índice errado.

O mesmo sinal cobra o seu no **peso**: no acerto pela face de dentro `n · v` é
negativo, e Schlick, que clampa o cosseno em zero, devolveria F = 1 — espelho
perfeito — fechando a refração justamente na saída do vidro. Por isso o peso usa
`|n · v|`: o ângulo que interessa é com a normal do lado de onde o raio veio, e
quem responde por não haver caminho de saída é a reflexão interna total, não o
clamp. Uma esfera de vidro que fica preta em vez de transparente costuma ser
este sinal, e não a direção do raio.

### Reflexão interna total

Saindo de um meio denso para um menos denso (`η > 1`), `sen²θ2` pode passar de
1 — e não existe ângulo cujo seno seja maior que 1. Fisicamente: não há caminho
de saída, e **toda** a luz é refletida de volta para dentro. É o que faz a fibra
óptica funcionar, e é o brilho prateado da borda de uma bolha de ar dentro
d'água.

No código isso é o `None` da função de refração, e quem chama troca o raio
transmitido por um refletido. Não é um caso de erro: é parte do modelo, e a
esfera de vidro fica errada sem ele.

## O peso do raio refratado

Simétrico ao peso do espelho, e sai do **mesmo** Fresnel: o que não voltou pelo
reflexo é exatamente o que entrou na superfície.

```text
espelho    F
refração   (1 − F) · transparency
difuso     (1 − metallic) · (1 − transparency) · albedo · (n · l)
```

`transparency` deixou de significar "que fração da luz atravessa" — essa
pergunta é de geometria, depende do ângulo, e quem responde é o Fresnel. Ele
responde a outra, que a geometria não sabe: do que **entrou** no material,
quanto atravessa até o outro lado em vez de voltar espalhado. Vidro ~1, tinta 0.
Daí ele aparecer duas vezes, aqui e cortando o `kd` do difuso em `shading.rs`:
os dois termos dividem a mesma sobra, e um só pode crescer às custas do outro.

### Por que os pesos valem no sentido de volta

A objeção é natural: o ray tracer anda ao contrário, e os dois raios secundários
vêm de lugares diferentes. Por que os pesos deles teriam alguma relação entre
si?

Porque a divisão de Fresnel não se aplica ao par — ela se aplica a cada feixe
que chega, um por vez:

```text
feixe que chega       fração que segue pro olho      o resto
────────────────────────────────────────────────────────────────────────
do espelho            F, refletido                   1 − F entra no vidro
de dentro do vidro    1 − F, transmitido             F volta pra dentro
```

A segunda linha é a que fecha o argumento: a fração que sai do vidro para o ar é
o mesmo `1 − F` porque as equações de Fresnel são simétricas no par de ângulos
conjugados — o coeficiente é o mesmo medido de qualquer um dos dois lados da
interface. É a reciprocidade de Helmholtz, e é ela que autoriza traçar a luz ao
contrário.

Somar as duas contribuições é **superposição**, não fusão de energia: dois
feixes se somam num ponto sem interagir. A restrição física é por feixe —
nenhum entrega mais de 100% de si mesmo somando todas as direções de saída.

E os coeficientes somarem 1 não é lei geral: é que as duas únicas direções que a
interface acopla com `v` são justamente os dois lados da mesma divisão. Numa
superfície difusa o acoplamento é o hemisfério inteiro, e o critério vira uma
integral — a albedo direcional, que precisa ser ≤ 1. Whitted colapsa esse
hemisfério em duas direções delta, e é por isso que aqui dá pra conferir somando
na mão.

### O teste da esfera uniforme

Ponha a cena inteira dentro de uma casca de radiância uniforme `L` (o *white
furnace*). Cada ponto de vidro devolve `(peso_reflexo + peso_refração) · L`:

```text
acoplado      F + (1 − F)·1  =  1        devolve L: o vidro some no fundo ✓
solto (0.9)   0.04 + 0.9     =  0.94·L   de frente, perde 6%
solto (0.9)   0.6  + 0.9     =  1.50·L   raspando, devolve mais luz do que existe
```

A última linha é o defeito que a mudança corrigiu, e ele compõe: cada nível de
recursão multiplica por 1.5 de novo, e o que segura não é o modelo, é
`max_depth`. Na imagem o sintoma aparece onde o Fresnel é maior — a silhueta do
vidro estourando em branco.

### O que continua aproximado

- **Reflexão interna total.** Schlick não vai a 1 quando não há caminho de
  saída, então lá o peso fica abaixo do físico. O modelo troca a *direção* do
  raio (transmitido vira refletido) e não o peso.
- **Radiância comprime ao mudar de meio**, por um fator (n₂/n₁)². Energia se
  conserva, radiância não é invariante. O fator cancela entre entrar e sair do
  mesmo objeto, e por isso é omitido.
- **O termo direto ainda entra sem o `(1 − F)` na frente.** A luz que a BRDF
  devolve deveria sair do mesmo orçamento que o reflexo, e hoje não sai: o
  `kd` desconta `transparency` e `metallic`, mas não o Fresnel. Fica para
  quando a BRDF microfacetada entrar.
