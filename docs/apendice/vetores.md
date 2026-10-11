# Produto escalar e vetorial

As duas operações entre vetores que aparecem em quase todas as outras seções.

## Produto escalar

```text
u · v = u.x * v.x + u.y * v.y              em 2D
u · v = u.x * v.x + u.y * v.y + u.z * v.z  em 3D
```

Significado:

- Mede o quanto dois vetores estão alinhados. Ele retorna:
  - valores positivos se o ângulo entre eles é < 90°
  - valor = 0 se o ângulo entre eles é = 90°
  - valores negativos se o ângulo entre eles é > 90° (quando eles apontam em
    direções opostas)
- Se um dos vetores é unitário, o resultado é a projeção de um dos vetores no
  outro.

O produto escalar de um vetor com ele mesmo é o comprimento do vetor: `|v| = √(v · v)`.

## Produto vetorial ou cruzado

```text
u × v = ( u.y * v.z − u.z * v.y,
          u.z * v.x − u.x * v.z,
          u.x * v.y − u.y * v.x )
```

Significado:

- O resultado dessa operação é um vetor perpendicular a `u` e `v`.
- O comprimento do vetor resultante é igual à área do paralelogramo formado entre
  os dois vetores no plano.
- O sentido do vetor resultante depende da ordem de `u` e `v`. Se você posicionar
  os dedos da mão direita no sentido de `u` e fechá-los em direção a `v`, o seu
  polegar apontará o sentido do resultado.

Essa operação não existe propriamente em 2D, mas pode ser simulada extendendo os
dois vetores pra 3D com `z = 0`; ou seja tratando eles como vetores 3D 'deitados'
no plano XY. Sobra só a componente `z`:

```text
u × v = u.x * v.y − u.y * v.x
```

Significado:

- O resultado dessa operação é a área do paralelogramo formado entre os dois
  vetores no plano.
- Se o resultado é positivo, o vetor 3D resultante apontaria pra 'fora da tela';
  ou seja os vetores `u × v` estão em sentido anti-horário.
