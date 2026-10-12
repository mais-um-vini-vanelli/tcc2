# Frustum e recorte

Os seis planos do volume de visão e o que sobra da geometria depois de recortada
contra eles. Recorte por Sutherland e Hodgman (1974).

## O plano

O frustum é feito de seis planos, e a pergunta que a gente faz pra cada um é
sempre a mesma — de que lado desse plano o ponto caiu?

Um plano em 3D fica definido por um vetor normal `n` e um ponto `p0` que
pertence a ele. Um ponto `p` qualquer só está no plano se o vetor que vai de
`p0` até `p` for perpendicular à normal:

```text
n · (p - p0) = 0
n · p - n · p0 = 0
```

Definindo `deslocamento = -(n · p0)`, sobra a forma que a struct guarda:

```text
n · p + deslocamento = 0
```

O valor dessa expressão diz de que lado o ponto está. Voltando à primeira forma
e chamando `v = p - p0`:

```text
n · v
```

Isso é a projeção de `v` em cima da normal do plano. Com a normal normalizada, o
resultado é a distância de `p` até o plano, com sinal:

```text
p acima (lado da normal)     n · v > 0
p no plano                   n · v = 0   (v e n perpendiculares)
p abaixo (lado oposto)       n · v < 0
```

<PlanoDistancia />

## O frustum

De onde saem as seis equações de plano do frustum.

### Near e far

Pro near plane queremos a normal apontando pra frente (`+z`), ou seja
`n = (0, 0, 1.0)`. Além disso, qualquer ponto do near plane tem a forma
`(0, 0, near)`.

O deslocamento do plano é:

```text
deslocamento = -(n · p0) = -((0, 0, 1.0) · (0, 0, near)) = -near
```

O deslocamento é o resultado de `Plane::distance` com `p = 0`:

```text
distance(0) = n · (0, 0, 0) + deslocamento = deslocamento
```

Ele diz o quão pra trás ou pra frente a origem está do plano.

### Laterais sob ortográfica

Na ortográfica a caixa não abre com a distância: as laterais são paralelas ao
eixo de visão, em `x = ±width/2` e `y = ±height/2`.

### Laterais sob perspectiva

A caixa de visão é uma pirâmide partindo da origem, ou seja `deslocamento = 0`.

Olhando só pro plano de cima da pirâmide:

```text
tan(fov/2) = oposto / adjacente, com adjacente = z
```

<FovMeiaAltura />

A cada unidade de `z` que avança, o plano superior da pirâmide (o maior `y`
visível) aumenta em `tan(fov/2)` — a quantidade que o código guarda como
`meia_altura`. Isso quer dizer que pra um ponto `p` estar dentro da caixa ele
precisa ter:

```text
p.y <= p.z * meia_altura
```

Revisando e abrindo uma equação de plano:

```text
n · p + deslocamento >= 0
n.x * p.x + n.y * p.y + n.z * p.z + deslocamento >= 0
```

Colocando o nosso plano na mesma forma:

```text
p.y <= p.z * meia_altura
-p.y + p.z * meia_altura >= 0
0 * p.x + (-1 * p.y) + meia_altura * p.z + 0 >= 0
```

Comparando com a fórmula aberta da equação de plano, a gente pode ver que a
normal precisa ser `(0, -1, meia_altura)` pra cumprir a inequação.

Então o plano de cima da caixa é `n = (0, -1, meia_altura)`, `deslocamento = 0`.
E podemos seguir a mesma lógica pras outras faces, trocando `tan(fov/2)` por
`aspect * tan(fov/2)` pras faces horizontais.

## Recorte no plano

Onde a aresta cruza o plano: a fórmula do `t`.

Se a gente conhece as duas extremidades de uma aresta, a gente pode andar por
ela com:

```text
ponto = ponto_atual + (ponto_seguinte - ponto_atual) * t
```

Ou seja, fazendo uma interpolação linear das duas posições com `t` de 0 a 1.

```text
t = 0    -> devolve ponto_atual
t = 1    -> devolve ponto_seguinte
t = 0.5  -> devolve o ponto médio entre ponto_atual e ponto_seguinte
```

O ponto `p` onde a aresta cruza o plano tem:

```text
1) distance(p) = n · p + deslocamento = 0
2) p = atual + (proximo - atual) * t
```

Substituindo a fórmula de `p` em 1) temos:

```text
distance(p) = n · [atual + (proximo - atual) * t] + deslocamento
```

Agora vamos simplificar essa fórmula.

Propriedades do produto escalar:

```text
P1 distributiva              -> u · (v + w) = u · v + u · w
P2 escalar (k) vai para fora -> u · (k * v) = k * (u · v)
```

Aplicando P1:

```text
n · [atual + (proximo - atual) * t] + deslocamento
n · atual + n · [(proximo - atual) * t] + deslocamento
```

Aplicando P2:

```text
n · atual + t * [n · (proximo - atual)] + deslocamento
```

Aplicando P1 de novo no `[n · (proximo - atual)]`:

```text
n · atual + t * (n · proximo - n · atual) + deslocamento
```

Note que `n · atual + deslocamento` é a distância do vértice atual até o plano.
Chamando ela de `d_atual`:

```text
d_atual + t * (n · proximo - n · atual)
```

Agora simplificando o `(n · proximo - n · atual)`. Note que:

```text
d_atual   = n · atual   + deslocamento  ->  n · atual   = d_atual - deslocamento
d_proximo = n · proximo + deslocamento  ->  n · proximo = d_proximo - deslocamento
```

Trocando:

```text
d_atual + t * [(d_proximo - deslocamento) - (d_atual - deslocamento)]
d_atual + t * (d_proximo - deslocamento - d_atual + deslocamento)
```

`deslocamento` e `-deslocamento` se cancelam:

```text
d_atual + t * (d_proximo - d_atual)
```

Perfeito, temos a fórmula da distância de um ponto da aresta em função de `t`.
Agora é igualar a 0 e resolver pra achar o ponto em que a aresta cruza o plano:

```text
d_atual + t * (d_proximo - d_atual) = 0
t * (d_proximo - d_atual) = -d_atual
t = -d_atual / (d_proximo - d_atual)
```
