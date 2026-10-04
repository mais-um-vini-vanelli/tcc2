# Reflexão e refração

## O raio refletido

Pra calcular a direção do raio de reflexão, precisamos:

- Decompor `d` na parte perpendicular à superfície (ao longo de `n`) com `(d · n) * n`
- O resultado aponta pra dentro da superfície, então multiplicar por -1 pra apontar pra fora (na direção de de `n`)
- Somar o dobro desse vetor normal ao `d` original: somar `-(d · n) * n` uma vez zera a componente perpendicular; 
  somar duas vezes a inverte. A componente paralela não é alterada.

```text
      n
      ^       
      |      
      |             
  d   |   r         
   \  |  /
    \ | /           r = d - 2(d · n) * n
     \|/
  ----X--------
```

::: warning 🚧 TODO
Trocar por plot, mostrando d · n também; mostrar -(d · n) * n somando duas vezes.
:::

## O peso do raio de espelho

Quanto do que o espelho vê entra na cor do ponto. É o Fresnel, calculado a partir de 
`metallic` e `albedo`, e essa foi uma decisão arbitrária da nossa lib: um `reflectivity` 
separado no material poderia discordar do especular da BRDF, que responde a mesma pergunta.

`f0` é a refletância de frente — a fração devolvida quando a luz chega
perpendicular:

```text
dielétrico (metallic = 0)   f0 = ((1 - n) / (1 + n))^2   branco, e o n é o do material
metal      (metallic = 1)   f0 = albedo                  ouro reflete amarelo, cobre laranja
```

Os materiais dielétricos têm componente especular branco - uma bola vermelha de plástico 
tem brilho especular branco - porque é a porção da luz que não entrou na superfície pra interagir com o pigmento.

Os materiais dielétricos têm especular alto e da cor do metal, e não têm lóbulo difuso - toda cor que vem dele especular/reflexo.

Schlick (1994) dá a variação com o ângulo em uma linha, no lugar da equação
completa:

```text
F(cos) = f0 + (1 - f0) * (1 - cos)^5
```

Com a luz de frente (`cos = 1`) sobra `f0`; raspando a superfície (`cos → 0`) o
termo vai a 1, e **todo** material vira espelho. É o efeito que se vê na água de um 
lago: o mesmo material que reflete 4% de frente reflete quase tudo de lado.

::: warning 🚧 TODO
Imagem de lago.
:::

A mesma dupla `f0`/Schlick é usada pelo especular da BRDF em `shading.rs` — o
peso do raio e a cor do brilho respondem à mesma pergunta física.

## O limite do espelho de Whitted

Um raio refletido só faz sentido se a superfície reflete numa direção só. Numa
superfície áspera as microfacetas apontam para todo lado, e o reflexo é a média
de um cone inteiro de direções — o lóbulo que a BRDF de microfacetas descreve.
Amostrar esse cone exige dezenas de raios por acerto, e é isso que separa
Whitted de um *path tracer*.

A regra adotada: o raio de espelho é disparado enquanto `roughness < 0.1`, e
acima disso fica só o termo direto. A alternativa — mandar um raio só no centro
do lóbulo — daria um reflexo nítido em um material que a própria BRDF descreve
como borrado, o que é pior que não refletir.

## O raio refratado

A direção de quem atravessa. A lei de Snell relaciona os ângulos com a normal
nos dois meios pelos índices de refração:

```text
n1 * sen θ1 = n2 * sen θ2   n1, n2 = índices dos dois meios
                            θ1, θ2 = ângulos com a normal
                            cos θ1 = -d · n
```

```text
            n ^
              |
        -d    |
          \   |
           \θ₁|
            \ |
    ----------+----------> û    (a superfície)
              | \
              |θ₂\
              |   \
              |    t
```

::: warning 🚧 TODO
Trocar por chart.
:::

Isolando `sen θ₂` e trocando `η = n1/n2`:

```text
sen θ₂ = n1 * n1 * sen θ₁
sen θ₂ = η * sen θ₁
```

Podemos decompor `t` em componente paralelo à normal + componente tangente
à superfície:

```text
t = sen θ₂ * û + cos θ₂ * -n
```

Então:

```text
t = sen θ₂ * û + cos θ₂ * -n
t = (η * sen θ₁) * û - cos θ₂ * n  # trocando sen θ₂ = η * sen θ₁
t = η * (sen θ₁ * û) - cos θ₂ * n
```

Podemos conseguir a componente tangente de `d` (`sen θ₁ * û`) com
`d - (componente normal) = d - (cos θ₁ * -n)`; ou seja, removendo o
deslocamento na direção da normal e deixando apenas o deslocamento na direção da
tangente:

```text
t = η * (d + cos θ₁ * n) - cos θ₂ * n
t = η * d + η * cos θ₁ * n - cos θ₂ * n
t = η * d + (η * cos θ₁ - cos θ₂) * n
```

Falta descobrir cos θ₂. Volando pra equação de Snell:

```text
sen θ₂ = η * sen θ₁
sen²θ₂ = η² * sen²θ₁       # elevando tudo ao quadrado
sen²θ₂ = η² * (1 - cos²θ₁) # com pitágoras: sen²θ₁ + cos²θ₁ = 1 -> sen²θ₁ = 1 - cos²θ₁
```

Se `sen²θ₂ > 1` é o caso de reflexão interna total (abaixo). 
Com `sen²θ₂` calculado, podemos descobrir `cos θ₂` com pitágoras:

```text
|t|² = (sen θ₂)² + (cos θ₂)²
cos θ₂ = √(1 - (sen θ₂)²)
```

### Entrando e saindo

O mesmo código serve para os dois sentidos - raio entrando e saindo do objeto - e o que decide é o sinal 
de `-d · n`. 

A normal da geometria sempre aponta para fora do objeto, então:

```text
-d · n > 0   o raio chega de fora:  η = 1/índice,
-d · n < 0   o raio sai de dentro:  η = índice,  
```

### Reflexão interna total

Saindo de um meio denso para um menos denso (`η > 1`), `sen²θ₂` pode passar de
1 — e não existe ângulo cujo seno seja maior que 1. Fisicamente: não há caminho
de saída, e **toda** a luz é refletida de volta para dentro. No código, trocamos o raio de refração 
por um de reflexão.

## O peso do raio refratado

Simétrico ao peso do espelho, e sai do **mesmo** Fresnel: o que não voltou pelo
reflexo é exatamente o que entrou na superfície.

```text
espelho    F
refração   (1 - F) * transparency
difuso     (1 - F) * (1 - transparency) * (1 - metallic) * albedo * (n * l)
```

`transparency` responde: do que **entrou** no material, quanto atravessa até o outro lado em vez 
de voltar espalhado? Vidro ~1, tinta 0.

A sequência de forks que um feixe pode fazer quando bate na superfície:

- `F` — refletiu na interface, nunca entrou → **raio de reflexão / especular**
- `1 - F` — entrou no material
  - `metallic` — absorvido lá dentro, não sai mais → **nada**
  - `transparency` — atravessou para o outro lado → **raio de refração**
  - o restante — espalhou e voltou para fora → **difuso**

### O que continua aproximado

- **Reflexão interna total.** Schlick não vai a 1 quando não há caminho de
  saída, então lá o peso fica abaixo do físico. O modelo troca a *direção* do
  raio (transmitido vira refletido) e não o peso.
- **Radiância comprime ao mudar de meio**, por um fator (n₂/n₁)². Energia se
  conserva, radiância não é invariante. O fator cancela entre entrar e sair do
  mesmo objeto, e por isso é omitido.
