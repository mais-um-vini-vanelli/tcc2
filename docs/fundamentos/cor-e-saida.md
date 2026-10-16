# Cor e saída

## Exposição

A exposição é um fator que multiplica todos os pixels antes do tonemap: `c = pixel * 2^exposure`. Cada
`+1` em `exposure` dobra a luz que chega ao filme e cada `-1` corta pela metade.

## Tone mapping

O tonemap é parecido com a conversão de linear para sRGB, mas resolve outro
problema:

- O tonemap pega valores de radiância da cena (cores `f32` no intervalo [0, ∞]) e
  converte pra cores `f32` no intervalo [0, 1].
- A conversão pra sRGB pega as cores `f32` no intervalo [0, 1] e converte pra
  `u8` [0, 255], com a maior parte dos inteiros alocados pras cores escuras
  (valores menores).

Sem tonemap nenhum, o que passa de 1 é cortado canal por canal na gravação. Isso
estoura os claros e desvia a cor.

### Reinhard

Reinhard é aplicar `c / (1 + c)` em cada canal. É uma opção simples de tonemap que cumpre 
o que a gente espera de uma curva de tons:

- Preto continua preto: `f(0) = 0`.
- Nunca passa de 1, por maior que seja a entrada.
- É sempre crescente: o mais claro na cena continua mais claro na imagem (o clamp
  ingênuo falha aqui, porque 2 e 40 viram o mesmo valor).
- Os escuros ficam como estão, já que eles não eram o problema. Com `c` pequeno,
  `1 + c ≈ 1`, então `f(c) ≈ c`.

## A codificação sRGB

O olho humano separa melhor os tons escuros que os claros. O pulo de
`Rgb(0.1, 0.1, 0.1)` pra `Rgb(0.2, 0.2, 0.2)` é bem visível; o mesmo pulo de
`Rgb(0.8, 0.8, 0.8)` pra `Rgb(0.9, 0.9, 0.9)` quase não dá de ver, mesmo os dois 
sendo os mesmos +0.1 de luz.

Gravar a imagem é trocar cada componente `f32` por um `u8`: 256 níveis no lugar
de um intervalo contínuo. Com a conta ingênua `(c * 255) as u8` esses níveis
ficam igualmente espaçados, e aí metade deles cai acima de 0.5, onde o
olho quase não distingue nada, enquanto o intervalo `[0, 0.1]` — que o olho
distingue bem — fica só com 26.

A solução é espaçar os 256 níveis de forma desigual, aplicando uma curva de gama
antes de escalar pra byte:

```text
γ = 2.2

linear -> sRGB:  (c ^ (1/γ)) * 255
sRGB -> linear:  (c / 255) ^ γ
```

<GamaCurva />

Com a cor já em [0, 1], a gravação aplica a gama e escala pra byte. O caminho de
volta — uma textura lida de um arquivo — desfaz a gama para voltar ao linear:

