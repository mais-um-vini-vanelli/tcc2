# Shading — teoria

## A lei do cosseno de Lambert

A luz se espalha por uma área cada vez maior quanto mais inclinada ela chega na
superfície; então a quantidade de watts por unidade de área é inversamente
proporcional à inclinação:

```text
angulo θ=0                    angulo θ grande
  | | | |                         \ \ \ \
  | | | |                         _\_\_\_\_
  ━━━━━━━━━                     ━━━━━━━━━━━━━━━━━━
4 feixes em 4 unidades        4 feixes em 8 unidades
de superfície                 de superfície
```

Pegando um feixe de luz:

```text
        \       \
         \       \
          \       C <- ângulo = 90º
           \   w / \  n
            \   /   \ |
             \ /     \|
───────────────A───────B──────
              └── s ──┘
```

- `w = |AC|` é a distância entre as duas bordas medida perpendicularmente a elas
- `s = |AB|` é o comprimento do feixe na superfície
- Na ascii art não aparece mas o ângulo `ACB = 90º`: `ABC` é um triângulo retângulo
- O ângulo em `B = 90º - (ângulo entre l e n) = θ`

No triângulo retângulo, cateto oposto = hipotenusa × seno do ângulo, então:

```text
w = s · sen(90° − θ) = s · cos(θ) -> s = w / cos θ
```

Pra todo `w`, a área que o feixe cobre na superfície é `w * (1/cos(θ))`.

```text
Watts por unidade de área = watts * w/s
                          = watts * w * 1/s
                          = watts * w * 1/(w * (1/cos(θ)))
                          = watts * w * (1/w) * cos(θ)
                          = watts * (w/w) * cos(θ)
                          = watts * cos(θ)
```

## kd e ks a partir de metallic

`kd` e `ks` são, na literatura, dois parâmetros do material. Aqui eles saem do
`metallic`: metal não devolve luz difusa, e o brilho dele é da cor do próprio
metal (ouro brilha amarelo) enquanto o de um dielétrico é branco (plástico
vermelho brilha branco).

Os dois fatores tiram luz do difuso pelo mesmo motivo: ela não voltou espalhada.
Em metal porque nada entra na superfície, em vidro porque o que entrou atravessou
e saiu do outro lado — e é o raio de refração que carrega essa parte.

`ks` de Blinn-Phong e `f0` de Schlick respondem à mesma pergunta física: que
fração da luz volta na interface sem entrar no material.

## De roughness para shininess

Transformar o parâmetro `roughness` de PBR pra `shininess` de Blinn-Phong:

- Primeiro `α = r²` (Disney, SIGGRAPH 2012), não é exata, mas essa fórmula foi
  encontrada empiricamente e dá bons resultados
- Depois `n = 2/α² − 2` (Walter et al. 2007)

[TODO: voltar aqui depois de adicionar Cook-Torrance pra provar isso]
