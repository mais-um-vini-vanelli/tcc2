# Whitted

## O termo direto de Whitted

Whitted (1979) escreve a luz que sai de um ponto `p` na direção `v` como três
partes somadas:

```text
L(p, v) = Σ_luzes  V(p, luz) * reflectancia(n, v, l) * radiancia(luz, p) 
        +  F(v, n) * L(p, r_reflexão)                                    
        +  kt      * L(p, r_refração)                                    
```

A primeira parte é a **iluminação direta**: 

```text
Σ_luzes  V(p, luz) * reflectancia(n, v, l) * radiancia(luz, p)
```

Percorremos as luzes da cena e calculamos, pra cada uma, quanta luz chega no ponto
(radiancia) e quanto dessa luz incidente volta na direção da câmera (reflectancia).

Uma luz da cena só contribui pra cor da superfície se ela não estiver bloqueada por outro
objeto - a luz deve estar 'visível' a partir do ponto - por isso geramos os raios de sombra:
raios partindo do ponto na superfície até cada luz da cena. Se o raio acerta algum objeto no
caminho até a luz, a luz está coberta e o termo de visibilidade `V(p, luz) = 0`; se o raio chega até a luz sem
bloqueios `V(p, luz) = 1`. Multiplicamos a iluminação direta por esse termo de visibilidade para descartar a 
contribuição das luzes bloqueadas.

A iluminação direta é calculada nas pipelines de rasterização e de raytracing.

As outras duas linhas calculam a luz que volta da superfície na direção da câmera por 
reflexão e refração, e não têm equivalente no rasterizador.

## A recursão de Whitted

Se a superfície tem reflexão (espelhos, metais etc.) ou refração (água, vidro etc.), o 
ray tracer gera os raios secundários de reflexão e refração, e a luz que chega por eles
é somada à cor da superfície. 
Note que os raios secundários podem acertar outras superfícies
refletoras/refratoras, então o processo recorre até encontrar uma superfície difusa ou
estourar um limite de recursões definido.

```text
trace(raio, profundidade):
    acerto = o primeiro objeto no caminho
    cor    = termo direto no acerto       

    se profundidade == max_depth: acabou

    cor += F  * trace(raio refletido, profundidade + 1)
    cor += kt * trace(raio refratado, profundidade + 1)
```

Essa recursão é o que faz o raytracing ser mais caro do que a rasterização: cada pixel gera um raio primário,
cada raio primário gera raios de sombra e (possivelmente) raios secundário, cada raio secundário pode gerar novos 
raios de sombra e raios secundários e cada raio precisa testar se intercepta a geometria da cena inteira.

Pra reduzir esse problema são usadas as [BVHs](/raytracing/bvh), que permitem reduzir (muito) a quantidade de testes de interseção que cada
raio precisa fazer.

::: info Especular da iluminação direta x raio de reflexão 

A iluminação direta já tem um brilho especular, e o ray tracer também gera o raio de
reflexão. 

Parece a mesma luz somada duas vezes, mas não é: `Light::Point` e
`Light::Directional` são pontos e direções matemáticas, de área zero. Nenhum
raio refletido jamais acerta uma delas. O especular direto é o brilho **da
luz**; o raio de espelho traz o reflexo **da geometria**. São conjuntos
disjuntos, então a luz só entra no cálculo uma vez.

Isso deixa de valer no dia em que existir luz com área — aí a mesma luz
apareceria nas duas parcelas, e uma delas teria que sair.

:::

## O raio de sombra

O `V(p, luz)` da fórmula responde: **existe alguma coisa entre este ponto e esta luz?**

```text
             luz
              o
             /
            /  <- o raio de sombra: origem no ponto, direção para a luz
      #########
          /
         /          o cubo está no caminho: o ponto está na sombra
        /  
    ---p---------
```

A direção e o alcance dependem do tipo da luz:

```text
Point        direção = (posição - p) normalizada,  alcance = |posição - p|
Directional  direção = -direction,                 alcance = infinito
Environment  não tem direção: nada a testar, sempre visível
```

Na luz pontual, um objeto *atrás* da luz não faz sombra nenhuma, por isso
só consideramos acertos com `t` < alcance. Na direcional não existe
"atrás da luz": ela vem do infinito, e qualquer coisa no caminho bloqueia.

Duas observações sobre o que este teste **não** precisa:

- **Não precisa do acerto mais próximo.** Basta saber que existe um, e pode parar 
  na primeira interseção encontrada (*any-hit*). Nosso código usa a busca pela interseção
  mais próxima por simplicidade.
- **Não usa material.** Sombra em Whitted é binária: ou a luz chega
  inteira, ou não chega. Vidro projeta a mesma sombra preta que concreto — a
  sombra colorida por objeto transparente exige carregar o fator de transmissão pelo
  caminho, o que é outro modelo.

## O acne e o epsilon

O raio de sombra nasce exatamente em cima da superfície que o gerou. Em
aritmética exata ele sairia dali sem tocá-la de novo; em `f32`, o ponto de
partida pode ficar dentro do objeto por causa de arredondamento e o raio de sombra
esbarra no próprio objeto. É a *shadow acne*: pixels pretos espalhados pela parte iluminada.

```text
sem deslocamento                  com deslocamento
                                        p'  (p + n * ε)
   ---p---------                     ---p---------
      ^                                 ^
      acerta a si mesmo em t ≈ 0        o t ≈ 0 já ficou para trás
```

A correção é empurrar a origem do raio secundário um `ε` (um valor pequeno) ao longo da normal,
para o lado em que ele vai seguir — `+n` se o raio sai pela frente da
superfície, `-n` se ele entra no objeto (o caso do raio de refração). O `t`
mínimo da busca também fica em `ε` como uma guarda adicional e se resolve o problema.
