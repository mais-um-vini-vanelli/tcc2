# Shading

## A lei do cosseno de Lambert

Um feixe de luz se espalha por uma área cada vez maior quanto mais inclinadao ele chega na
superfície:

```text
ângulo θ=0                    ângulo θ grande
  | | | |                         \ \ \ \
  | | | |                         _\_\_\_\_
  ━━━━━━━━━                     ━━━━━━━━━━━━━━━━━━
feixe em 4 unidades         mesmo feixe em 8 unidades
de superfície                 de superfície
```

Então a quantidade de energia por unidade de área é inversamente
proporcional à inclinação.

Olhando mais a fundo:

<LeiCossenoFeixe />

- `w = |AC|` é a distância entre as duas bordas do feixe, medida perpendicularmente a elas
- `s = |AB|` é o comprimento do feixe na superfície
- O ângulo `ACB = 90°`, então `ABC` é um triângulo retângulo
- O ângulo em B é `90° − θ`, com `θ = ângulo entre l e n`

No triângulo retângulo, seno do ângulo * hipotenusa = cateto oposto, então:

```text
w = s * sen(90° − θ) = s * cos(θ) -> s = w / cos θ
```

No triângulo retângulo, os dois ângulos não retos somam 90° e o cateto adjacente de um é o cateto oposto
do outro, então `sen(90° − θ) = cos(θ)`. Trocando:

```text
w = s * sen(90° − θ) = s * cos(θ) -> s = w / cos θ
```

Ou seja, pra todo `w`, a área que o feixe cobre na superfície é `w * (1/cos(θ))`.

```text
Energia por unidade de área = energia * w/s
                            = energia * w * 1/s
                            = energia * w * 1/(w * (1/cos(θ)))  # Trocando s
                            = energia * w * (1/w) * cos(θ)
                            = energia * (w/w) * cos(θ)
                            = energia * cos(θ)
```

## A atenuação com a distância

A energia de uma luz pontual se espalha na casca de uma esfera de raio `d`. A área dessa
casca é `4π·d²`, então aumentar `d` aumenta a área e diminui a quantidade de
energia que chega a cada ponto da esfera. Então a quantidade de watts por ponto
da esfera (feixe de luz) é `watts / 4π·d²`.

`intensity` já foi informada em `watts / 4π` — o quanto a luz ilumina a um metro
de distância, quando `d = 1` e `watts / 4π·d² = watts / 4π`. Então:

```text
watts por feixe = watts / 4π·d² = intensity / d²
```

## O cone do spot

Um spot é uma luz pontual com um cone por cima: tem a mesma atenuação por `1/d²`, 
e um fator a mais que mede o quão alinhada a superfície está com o eixo frontal do spot.

::: warning 🚧 TODO Médio
Visualização disso.
:::

Para descobrir se um ponto está dentro do cone do spot, usamos o ângulo entre o raio 
que sai da luz até o ponto e o eixo do spot.

Daria de calcular esse ângulo com um `acos` por ponto, mas podemos comparar cossenos
diretamente pra economizar processamento; com o detalhe que o cosseno diminui quando o
ângulo cresce, então `cos_interno > cos_externo` e as commparações ficam invertidas.

O fator é 0 fora do cone externo, 1 dentro do interno, e uma rampa entre os dois:

```text
t = clamp((cos θ − cos_externo) / (cos_interno − cos_externo), 0, 1)
```

Exemplo com `inner_angle = 20°` e `outer_angle = 30°`:

```text
cos_interno = cos 20° = 0.9397
cos_externo = cos 30° = 0.8660
cos_interno − cos_externo = 0.0737
```

| ponto em     | θ   | cos θ  | conta                              | t         |
|--------------|-----|--------|------------------------------------|-----------|
| centro       | 0°  | 1.0000 | (1.0000 − 0.8660) / 0.0737 =  1.82 | 1 (clamp) |
| cone interno | 20° | 0.9397 | (0.9397 − 0.8660) / 0.0737 =  1.00 | 1         |
| no meio      | 25° | 0.9063 | (0.9063 − 0.8660) / 0.0737 =  0.55 | 0.55      |
| cone externo | 30° | 0.8660 | (0.8660 − 0.8660) / 0.0737 =  0.00 | 0         |
| fora         | 40° | 0.7660 | (0.7660 − 0.8660) / 0.0737 = −1.36 | 0 (clamp) |

O `t²` suaviza a borda:

| t    | t²   |
|------|------|
| 0.0  | 0.0  |
| 0.25 | 0.06 |
| 0.5  | 0.25 |
| 0.75 | 0.56 |
| 1.0  | 1.0  |

## O 1/π de Lambert

O brilho difuso de uma superfície de lambert se espalha igualmente por todas as direções,
mas isso não quer dizer que ela devolve a mesma quantidade de luz de todos os ângulos de visão:
um quadrado fixo da superfície tem tamanhos diferentes quando se olha de cima e de lado.

::: warning 🚧 TODO
Plot de uma esfera, um pedaço no plano e um slider pra controlar o ângulo.
:::

A lei do cosseno diz quanto chega . A BRDF `f` diz quanto da luz que chega (a irradiância `E`) sai 
por cada direção (a radiância `L`). Em uma superfície de Lambert esse brilho é o mesmo em todas as direções:

```text
L_saida = f · E
```

Mesmo brilho em todas as direções não é mesma potência em todas as direções. A potência que calculamos
antes dependia do ângulo de visão, mas L_saida é a **potência por área aparente**. Quando olhamos a superfície de 
lado a potência que chega na nossa direção diminui, mas a área também diminui; os dois se anulam e o brilho difuso da 
superfície é constante.

Somando o que sai por todas as direções do hemisfério dá a energia total que deixa a superfície:

```text
M = ∫ L_saida · cos θ dω = L_saida · ∫ cos θ dω = f · E · ∫ cos θ dω
```

A integral do cosseno no hemisfério, com `dω = sen θ dθ dφ`:

```text
= ∫₀^2π ( ∫₀^π/2 cos θ · sen θ dθ ) dφ
= ∫₀^2π ( sen(π/2)² / 2 - sen(0)² / 2 ) dφ  # resolvemos a integral de dentro: a derivada de sen²θ / 2 é cos θ · sen θ,
                                              então trocamos e subtraímos os limites
= ∫₀^2π ( 1/2 ) dφ                          # sen²(π/2)/2 − sen²(0)/2 = 1²/2 − 0²/2 = 1/2
= 1/2 · ∫₀^2π dφ                            # constante sai da integral
= 1/2 · 2π                                  # ∫₀^2π dφ = 2π
= π
```

Então a quantidade de energia que se espalha pelo hemisfério é `f · E · π`.

Agora falta combinar isso com o albedo: uma superfície de albedo `a` devolve a fração `a` do que recebeu, então
a energia que sai é `a · E`. As duas contas falam da mesma energia, então podemos comparar:

```text
f · E · π = a · E -> f = a / π
```

E assim chegamos em `f` (a BRDF) que diz 'dada a luz que chegou num ponto, qual é o brilho que sai em uma direção?'.

## kd e ks a partir de metallic

Na literatura `kd` e `ks` são dois parâmetros do material, mas aqui eles saem do
`transparency` e `metallic`.

Metais não devolvem luz difusa, só especular (reflexos) e o especular é da cor do metal (ouro brilha amarelo).
Materiais dielétricos como plástico devolvem luz difusa e especular, e o especular é branco.
  Materiais dielétricos como água podem ter transparência, que diz 'da luz que entrou no material, quanta atravessou ao invés de
  voltar como brilho difuso?'

`transparency` e `metallic` tiram luz do difuso: metais não têm brilho difuso, e dielétricos com transparency só devolvem como
difuso a luz que não atravessou o material.

`ks` de Blinn-Phong e `f0` de Schlick respondem à mesma pergunta física: que
fração da luz volta na interface sem entrar no material.

## A normalização do especular

O especular de Blinn (1977) é empírico: `ks · (n·h)^shininess` foi escolhido
porque produz um brilho parecido com o que aparece em materiais de verdade. O `ks` define o brilho 
especular máximo, quando `n·h = 1` (o reflexo da luz bate perfeitamente com a câmera). 

A shininess diz o quão polida é a superfície; uma superfície polida reflete os raios em um cone pequeno (brilho especular
forte e em uma área pequena), enquanto uma superfície rugosa espalha os raios em um cone grande (brilho especular fraco e em uma área grande):
a quantidade de luz especular não muda, mas o formato do brilho sim.

O problema é que com a fórmula `ks · (n·h)^shininess`, aumentar a shininess (polir a superfície) diminui cada vez mais 
a quantidade de luz que ela reflete. Dá de provar isso integrando o termo `(n·h)^shininess` em todas as direções que a luz é refletida:

```text
∫ (n·h)^shininess · cos θ dω  ≈  8π / (shininess + 8)
```

Jogando alguns valores de `shininess` nesse `8π / (shininess + 8)`:

| shininess | energia do lobo        |
|-----------|------------------------|
| 1         | 2.79                   |
| 10        | 1.40                   |
| 100       | 0.23                   |
| 1000      | 0.025                  |

Dá de ver que com shininess = 1 o brilho especular tem 2.79x mais energia do que chegou, e com shininess = 1000 ele só tem 0.025x a energia original.

Queremos que o fresnel `f0` bata com a energia distribuída pelo hemisfério, então:

```text
f0 = ks · ∫ (n·h)^shininess · cos θ dω 
f0 = ks · (8π / (shininess + 8))
ks = f0 / (8π / (shininess + 8))
ks = f0 · (shininess + 8) / (8π)
```

### De onde vem o 8π / (shininess + 8)

## De roughness para shininess

`α` é um escalar que diz o quão inclinadas as microfacetas de uma superfície são. 

Calculamos ele com: `√( (1/A) ∫∫ [ (∂z/∂x)² + (∂z/∂y)² ] dx dy )`, ou seja:
  - percorrer a superfície deitada no plano XY
  - em cada ponto, calcula a inclinação com (z/x)² + (z/y)² (as tangentes nas direções x e y)
  - acumular as inclinações e tirar a média
  - tirar a raiz quadrada disso, pra desfazer os quadrados da tangentes

Esse número é a tangente média da superfície, então ele diz a inclinação média das microfacetas:

| α         | inclinação média (arctan(α)) |
|-----------|------------------------------|
| 1         | 45°                          |
| 0.25      | 14°                          |
| 0.1       | 0.6°                         |

O usuário informa o `α` indiretamente pelo `roughness` do material: `α = roughness²`. Essa conversão
é o que a BRDF do glTF [especifica](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#microfacet-surfaces),
e serve pra que mudar a roughness perto de 0 ou perto de 1 cause mudanças visuais parecidas.

Agora precisamos converter o `α` para a `shininess` que o Phong usa.

Phong pode ser escrito como uma NDF; uma função `f(m)` que diz quantas das microfacetas apontam na direção `m`,
com 1 = todas as microfacetas e 0 = nenhuma:

```text
D(m) = χ⁺(m·n) · (shininess+2)/(2π) · (cos θ_m)^shininess

# χ⁺(m·n) é a função indicadora positiva; 1 se a microfaceta aponta pra fora da superfície, 0 se aponta pra dentro
(não participa da soma).

# (p+2)/(2π) é um termo de normalização, pra forçar ∫ D(m)(m·n) dω = 1; a área projetadas das microfacetas apontando pra 
todas as direções precisa bater com a área da superfície original.

# θ_m é o ângulo entre m e a normal (m·n)
```

E aqui temos a NDF de Beckmann, que usa o `α`:

```text
D(m) = χ⁺(m·n) · exp(−tan²θ_m / α²) / (π · α² · cos⁴θ_m)
```

Avaliamos as duas com θ_m = 0 e comparamos:

```text
Phong:     D(0) = (shininess+2)/(2π) # cos 0 = 1
Beckmann:  D(0) = 1/(π α²)           # cos 0 = 1, tan 0 = 0

(shininess + 2)/(2π) = 1/(π α²)
 shininess + 2       = 2/α²
 shininess           = 2/α² − 2
```

E assim chegamos na conversão: `shininess = 2/α² − 2`.

## O termo emitido

Além de espalhar luz que recebeu, uma superfície também pode emitir luz própria;
isso é representado pelo campo `emissive` do material.

O `emissive` não tem teto em 1. `albedo` é uma **fração** do que chega, e
acima de 1 devolveria mais energia do que recebeu; `emissive` é radiância,
e o valor de uma lâmpada é muito maior que o de um papel branco.

A ideias de conservação de energia, ângulo de incidência de visão etc. não se
aplicam a esse componente, já que a luz vem da própria superfície.

Um objeto emissivo **não ilumina os vizinhos**; o jeito de fazer uma lâmpada é combinar um objeto 
emissivo pra aparecer na imagem, e uma `Light` no mesmo lugar pra iluminar. 
O raytracer não considera objetos emissivos como fonte de luz; pra implementar
isso corretamente seria preciso luzes de área com integração de Monte Carlo.
