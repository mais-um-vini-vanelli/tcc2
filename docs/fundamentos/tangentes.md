# Tangentes e normal mapping

A tangente por vértice é pra onde a superfície anda quando o `u` da uv cresce.
Junto com a normal ela monta a base em que um normal map guarda os seus vetores.

## A base TBN

```text
T  no eixo do u
B  no eixo do v
N  pra fora da superfície
```

`B` não é guardado porque sai de `N × T`, com o handedness definindo a direção.
Então o que o vértice guarda é só a tangente — unitária e perpendicular à normal
do vértice — e o handedness: é o sinal da bitangente, +1 ou −1.

## A tangente por vértice

Numa superfície de parametrização conhecida, `du` e `dv` são as direções em que a
uv cresce ali. A tangente é o `du` projetado no plano da superfície
(Gram-Schmidt), e o sinal sai de comparar o `N × T` que a base vai usar com o
`dv` que a uv realmente tem:

```text
T = normalizado( du − N * (N · du) ) # tiramos de du a componente que está ao longo de `N` e renormalizamos
handedness = sinal( (N × T) · dv )
```

### A tangente da esfera

A fórmula da normal da esfera é `N(θ,φ) = (sen θ cos φ, cos θ, sen θ sen φ)`
([Tesselação](/rasterizacao/tesselacao#a-esfera-uv)).

O `u` cresce junto com o `φ`, então a tangente é a direção em que a normal anda
quando o `φ` cresce — o vetor que vai da normal até a normal de um `φ` um
pouco maior. Esse vetor é a derivada de `N` com relação a `φ`.

Comparando a fórmula da normal com a fórmula da derivada, dá de conseguir a
derivada com `(−N.z, 0, N.x)`:

```text
N      = ( sen θ cos φ,  cos θ,  sen θ sen φ)
∂N/∂φ  = (−sen θ sen φ,      0,  sen θ cos φ)

du = ∂N/∂φ = (−N.z, 0, N.x)
```

Nos polos o paralelo tem raio zero e a direção do `u` não existe; aí cai na
tangente qualquer.

### Quando não há uv

Sem uv de onde tirar a tangente de verdade, serve uma tangente qualquer
perpendicular à normal: o relevo sai girado, mas a base fecha.

Ela sai de cruzar a normal com um dos eixos — o menos alinhado a ela, o que evita
o produto vetorial quase nulo de dois vetores paralelos:

```text
eixo = (1, 0, 0)  se |N.x| < 0.9
       (0, 1, 0)  caso contrário

T = normalizado( eixo × N )
```

## Do mapa para o mundo

A normal que o shading usa no ponto é a da geometria, distorcida pelo mapa de
normais quando existe um. A normal e a tangente chegam em espaço de mundo e já
interpoladas, e a base TBN é montada ali.

Primeiro a tangente é reortogonalizada contra a normal interpolada: tiramos de
`T` a componente que está ao longo de `N` e renormalizamos. Se as duas são
paralelas, a componente é tudo e não sobra tangente nenhuma; aí entra a tangente
qualquer perpendicular a `N`.

A textura guarda valores em [0, 1], mas uma normal tem componentes em [−1, 1],
então precisamos converter: `n = normalizado( texel * 2 − 1 )`.

E aí a normal do normal map vai do espaço tangente (TBN) para o espaço de mundo,
multiplicando os componentes da normal pela base:

```text
B = (N × T) * handedness

n_mundo = normalizado( T * n.x  +  B * n.y  +  N * n.z )
```

## A bitangente sob espelhamento

O transform pode mexer no *lado* da bitangente: uma escala com um número ímpar de
eixos negativos espelha o espaço, e o handedness inverte junto.