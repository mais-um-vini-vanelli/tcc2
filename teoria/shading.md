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

## O 1/π de Lambert

A lei do cosseno diz quanto chega (a irradiância `E`, em W/m²). A BRDF `f` diz
quanto disso sai por cada direção: uma superfície de Lambert espalha por igual,
então a radiância que sai é a mesma pra todo lado:

```text
L_saida = f · E
```

Somando o que sai por todas as direções do hemisfério, com o mesmo `cos` da lei
de Lambert (uma direção rasante enxerga a área de lado, e leva menos), dá a
energia total que deixa a superfície:

```text
M = ∫ L_saida · cos θ dω = f · E · ∫ cos θ dω
```

A integral do cosseno no hemisfério, com `dω = sen θ dθ dφ`:

```text
∫₀^2π ∫₀^π/2 cos θ · sen θ dθ dφ = 2π · [sen² θ / 2]₀^π/2 = 2π · 1/2 = π
```

O hemisfério tem ângulo sólido 2π, e o peso do cosseno corta isso pela metade.

Uma superfície de albedo `a` devolve a fração `a` do que recebe, então `M = a · E`:

```text
f · E · π = a · E   ->   f = a / π
```

Sem o 1/π, `M = π · a · E`: qualquer albedo acima de 1/π ≈ 0.32 devolveria
mais energia do que recebeu.

A luz ambiente não leva o 1/π porque ele se cancela. Com a mesma radiância `L`
chegando de todas as direções, `E = ∫ L cos θ dω = π · L`, e o que sai é
`(a/π) · π · L = a · L`, que é a conta que o `shade` faz.

Nas luzes pontuais e direcionais o 1/π fica. É por isso que a mesma cena
precisa de `intensity` π vezes maior do que antes da correção, e é o que deixa
os valores do `KHR_lights_punctual` entrarem direto: o glTF usa a mesma BRDF.

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

Os dois ainda levam um fator que não é da fórmula clássica: o 1/π de Lambert no
`kd`, e a normalização do especular no `ks`. São escolhas nossas na hora de
montar os parâmetros; a fórmula de Blinn-Phong continua a da literatura.

## A normalização do especular

O especular de Blinn (1977) é empírico: `ks · (n·h)^n` foi escolhido porque
produz um brilho com cara de brilho. O `ks` fixa o **pico** do lobo, mas não o
**total** de luz que ele devolve, que é a integral do lobo nas direções de
saída. Com a luz chegando pela normal:

```text
∫ (n·h)^n · cos θ dω  ≈  8π / (n + 8)
```

| n     | energia do lobo (× ks) |
|-------|------------------------|
| 1     | 2.86                   |
| 10    | 1.50                   |
| 100   | 0.24                   |
| 1000  | 0.025                  |
| 10000 | 0.0025                 |

O lobo estreita e a energia cai junto: polir a superfície faria ela refletir
menos. Fisicamente a energia é a mesma, concentrada num cone menor, então o
pico é que tem que subir.

Multiplicar `ks` pelo inverso da integral fixa o total em `ks`:

```text
ks = f0 · (n + 8) / (8π)
```

O fator vem de *Real-Time Rendering* (3ª ed., 2008); a versão pro Phong,
`(n + 2) / (2π)`, é de Lewis (1994). É uma aproximação: a tabela acima é a
integral numérica, e a diferença para `8π / (n + 8)` fica abaixo de 8%.

Com roughness baixo o `n` vai para milhões, e o brilho de uma luz pontual num
vidro vira um ponto pequeno e muito intenso, como o reflexo de uma lâmpada.

## De roughness para shininess

Transformar o parâmetro `roughness` de PBR pra `shininess` de Blinn-Phong:

- Primeiro `α = r²` (Disney, SIGGRAPH 2012), não é exata, mas essa fórmula foi
  encontrada empiricamente e dá bons resultados
- Depois `n = 2/α² − 2` (Walter et al. 2007)

[TODO: voltar aqui depois de adicionar Cook-Torrance pra provar isso]

## O termo emitido

A equação de renderização (Kajiya, 1986) tem dois termos: o que a superfície
emite e o que ela devolve do que chega nela.

```text
L(x, v) = Le(x, v)  +  ∫ f(x, l, v) · L(x, l) · cos θ  dω
          └── emitido ──┘   └────────── refletido ──────────┘
```

O `shade` é a integral aproximada por uma soma sobre as luzes da cena; o `Le`
é o `Material.emissive`, e entra somado e cru:

```text
cor = emission + Σ_luzes V · f · radiância
```

Cru porque ele não é luz que chegou. A BRDF, o cosseno de Lambert, o 1/d² e a
visibilidade são todos conta sobre a luz que veio de fora, e nada disso vale
pra energia que sai da própria superfície: uma lâmpada acesa não fica mais
escura vista de lado nem quando alguém passa na frente da janela.

Daí três consequências:

- O `emissive` não tem teto em 1. `albedo` é uma **fração** do que chega, e
  acima de 1 devolveria mais energia do que recebeu; `emissive` é radiância,
  e o valor de uma lâmpada é muito maior que o de um papel branco. Quem traz
  isso de volta pro intervalo da imagem é o `exposure` e o tonemap do
  `OutputOptions`.
- Um objeto emissivo **não ilumina os vizinhos**. Whitted só integra sobre as
  luzes da lista, e a superfície não está nela. No ray tracer ele aparece nos
  reflexos e através do vidro, porque a recursão chama o `shade` no ponto
  atingido — mas o chão embaixo dele continua escuro. Iluminar de verdade é
  luz de área: amostrar pontos na superfície emissiva e mandar um raio de
  sombra pra cada, o que já é integração de Monte Carlo.
- Enquanto isso, o jeito de fazer uma lâmpada é o par: um objeto emissivo pra
  aparecer na imagem, e uma `Light` no mesmo lugar pra iluminar. O objeto
  bloquearia o raio de sombra da sua própria luz, e é pra isso que existe o
  `Object.casts_shadows`.

No glTF é o `emissiveFactor` com o `emissiveTexture` — multiplicados, porque
`Value` é constante **ou** mapa. A extensão `KHR_materials_emissive_strength`
existe justamente porque o fator do formato base vai só até 1, e uma lâmpada
HDR precisa passar disso; aqui basta pôr o valor grande direto no `emissive`.
