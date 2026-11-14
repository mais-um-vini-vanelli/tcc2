# Tesselação

O rasterizador trabalha só com triângulos: todo objeto da cena passa por
`Primitive::tessellate` na entrada, que converte o primitivo em uma malha de
triângulos.

É o equivalente de `intersect` do lado do ray tracer: um resolve a superfície por
aproximação em triângulos, o outro pela equação exata.

## O cubo

### As seis faces

Cada face tem a normal, e pra onde apontam a direita e o "pra baixo" de quem
olha a face de fora. A textura segue essas direções, as mesmas que o `intersect`
usa:

```text
normal         direita        baixo
( 1,  0,  0)   ( 0, 0, -1)    (0, -1,  0)
( 0,  1,  0)   ( 1, 0,  0)    (0,  0,  1)
( 0,  0,  1)   ( 1, 0,  0)    (0, -1,  0)
(-1,  0,  0)   ( 0, 0,  1)    (0, -1,  0)
( 0, -1,  0)   ( 1, 0,  0)    (0,  0, -1)
( 0,  0, -1)   (-1, 0,  0)    (0, -1,  0)
```

Os cantos saem de somar e subtrair essas duas direções da normal, e são
numerados vistos de fora: 0 em cima à esquerda, 1 em cima à direita, 2 embaixo à
direita, 3 embaixo à esquerda.

```text
canto 0 = normal − direita − baixo     uv (0, 0)
canto 1 = normal + direita − baixo     uv (1, 0)
canto 2 = normal + direita + baixo     uv (1, 1)
canto 3 = normal − direita + baixo     uv (0, 1)
```

### O winding

`0→1→2` gira no sentido horário visto de fora; a face da frente é a
anti-horária, então os dois triângulos de cada face são `0→3→2` e `0→2→1`
([Backface culling](/rasterizacao/triangulo#backface-culling)).

## A esfera uv

Tesselação em grade de paralelos e meridianos ("uv sphere"), com o dobro de
meridianos em relação aos paralelos.

`θ` desce do polo norte (0) ao sul (π ou 90º); `φ` dá a volta no equador. Na esfera
unitária centrada na origem, a posição do ponto e a normal ali são o mesmo vetor:

```text
u = i / paralelos      θ = u · π
v = j / meridianos     φ = v · τ

r      = sen θ
normal = (r · cos φ,  cos θ,  r · sen φ)
uv     = (v, u)
```

<EsferaPhi />

### Os vértices repetidos

Nos polos (`θ = 0`, `θ = π`) e na emenda dos meridianos (`φ = τ`) a gente tem
vários vértices na mesma posição. Isso é preciso pra permitir uvs diferentes pra
esses vértices e não ter problema na hora de aplicar texturas.

### Os triângulos degenerados nos polos

Cada célula da grade (paralelo `i` × meridiano `j`) tem quatro pontos, e vira
dois triângulos anti-horários vistos de fora, como no cubo:

```text
a = i · por_linha + j
b = a + 1
c = a + por_linha
d = c + 1
```

Na fileira `i = 0`, `a` e `b` estão os dois na linha 0, que é `θ = 0`: o polo
norte. Todos os vértices dessa linha ocupam a mesma posição no espaço. Então:

```text
   a = b   ← o mesmo ponto
    /\
   /  \
  c————d
```

Mesma coisa pra `i = paralelos - 1`, mas com `c = d`. Esses dois triângulos
degenerados são descartados, e cada fileira de polo gera só um triângulo por
célula em vez de dois.
