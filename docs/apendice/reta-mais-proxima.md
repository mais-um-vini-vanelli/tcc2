# O ponto mais próximo entre duas retas

Em que ponto de uma reta ela passa mais perto de outra. É a conta que o gizmo do
editor usa pra saber quanto o mouse arrastou ao longo de um eixo.

As duas retas, cada uma com um ponto e uma direção unitária:

```text
eixo:  origem_eixo + t * dir_eixo
raio:  origem_raio + s * dir_raio
```

O gap entre um ponto de cada uma:

```text
g(t, s) = (origem_eixo + t * dir_eixo) − (origem_raio + s * dir_raio)
        = (origem_eixo − origem_raio) + t * dir_eixo − s * dir_raio
        = d + t * dir_eixo − s * dir_raio
```

Chamando `d = origem_eixo − origem_raio`.

Queremos o `t` do ponto em que o raio do mouse passa mais perto do eixo, ou seja
onde o vão é o mais curto. No vão mais curto, `g` é perpendicular às duas retas:
se ele não fosse perpendicular ao eixo, dava pra andar ao longo do eixo e chegar
mais perto.

```text
g · dir_eixo = d · dir_eixo + t * (dir_eixo · dir_eixo) − s * (dir_raio · dir_eixo) = 0
g · dir_raio = d · dir_raio + t * (dir_eixo · dir_raio) − s * (dir_raio · dir_raio) = 0
```

Com `dir_eixo · dir_eixo = dir_raio · dir_raio = 1` (as duas direções são
unitárias) e `a = dir_eixo · dir_raio`:

```text
g · dir_eixo = d · dir_eixo + t     − s * a = 0  ->  t = −(d · dir_eixo) + s * a
g · dir_raio = d · dir_raio + t * a − s     = 0  ->  s =  (d · dir_raio) + t * a
```

Substituindo o `s` na equação do `t`:

```text
t               = −(d · dir_eixo) + (d · dir_raio + t * a) * a
t               = −(d · dir_eixo) + (d · dir_raio) * a + t * a * a
t − t * a * a   = −(d · dir_eixo) + (d · dir_raio) * a
t * (1 − a * a) = (d · dir_raio) * a − d · dir_eixo
t               = [ (d · dir_raio) * a − d · dir_eixo ] / (1 − a * a)
```
