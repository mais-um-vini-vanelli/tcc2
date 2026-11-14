# Projeção

Camera space -> NDC -> pixel: as duas últimas trocas de coordenadas, nos dois
modos de câmera.

## Camera space para NDC

### Perspectiva

A área visível é um cone saindo da câmera, e abrindo mais conforme se afasta no
eixo `z`.

Perto da câmera cabe pouco mundo na tela; longe cabe muito. Objetos próximos
ficam maiores, objetos distantes ficam menores (as coordenadas `x`, `y` dos
pontos ficam mais próximas de 0).

Divide o `fov` por 2 pra descobrir o ângulo entre o topo e a origem (ou a base e
a origem, mesmo valor). E depois pega a tangente desse ângulo, que diz *quanto a
altura aumenta a cada unidade que a gente avança em `z`*, já que
`tan() = oposto / adjacente`.

<FovMeiaAltura />

Agora a conversão dos pontos em camera space pra NDC. Para o eixo `y`:

- o ponto está em `(x, y, z)`, a `y` unidades de altura;
- no eixo `y`, o topo da tela fica a `z * meia_altura` unidades;
- então a coordenada `y` projetada vai ficar em
  `y / topo da tela = y / (z * meia_altura)`.

Para o eixo `x`, não podemos usar só `tan(fov/2)`, porque o `fov` é o ângulo
vertical e a tela geralmente tem uma largura diferente da sua altura.

O aspect ratio é a proporção largura/altura, então multiplicamos `meia_altura`
(taxa de aumento da altura por unidade de `z`) por `aspect` pra descobrir a taxa
de aumento da largura por unidade de `z`. Se a largura da tela é 2 vezes a
altura, faz sentido o tamanho da caixa de visão crescer 2 vezes mais rápido que
a altura.

Prova que `meia_largura = meia_altura * aspect`. Num triângulo retângulo,
`tan(ângulo) = oposto / adjacente`:

```text
             │ ← H  (meia-altura visível: o "oposto")
câmera ●─────┘
        ← z →   (a distância: o "adjacente")
```

```text
tan(fov_v / 2) = height / z
tan(fov_h / 2) = width / z
```

```text
aspect = width / height  ->  width = aspect * height

tan(fov_h / 2) = width / z
               = (height * aspect) / z    # trocando width = height * aspect
               = (height / z) * aspect
               = tan(fov_v / 2) * aspect  # porque height / z = tan(fov_v / 2)
```

<ProjecaoNdc modo="perspectiva" />

### Ortográfica

A área visível é uma caixa de `width * height`. Usando o eixo `x` de exemplo, o
centro da área fica em `x = 0`, e os limites em `-width/2` e `+width/2`.

A gente quer um fator `f` que converte:

- um ponto com `x = -width/2` -> `x = -1` em NDC;
- um ponto com `x = +width/2` -> `x = +1` em NDC.

Dá de transformar em sistema de equação e resolver:

```text
A) -width/2 * f = -1
B) +width/2 * f =  1
```

Isolando `f` na equação B, temos `f = 1/(width/2)`. Trocando `f` na equação A,
temos `-width/2 * 1/(width/2) = -1`, que é uma verdade. Então dá de usar
`1/(width/2)` pra converter coordenadas do eixo `x` de camera space em NDC.

A mesma ideia vale pro eixo `y`.

Na projeção ortográfica o tamanho dos objetos não muda com a distância da
câmera, então `z` não é usado na projeção.

<ProjecaoNdc modo="ortografica" />

## NDC para pixel

NDC -> pixel: a última etapa da projeção.

Agora sim, só converter as coordenadas `x, y = [-1, 1]` em uma matriz de pixels
`x = [0, largura]` e `y = [0, altura]`.

Exemplo com a resolução 320x240:

```text
operação | range
    x      -1 …   1
+ 1.0       0 …   2    # tira o negativo, origem vai pra borda esquerda
* 0.5       0 …   1    # vira fração da tela
* 320       0 … 320    # vira index do pixel
```

`y` cresce pra cima em NDC e pra baixo no buffer, então a gente primeiro inverte
a direção com `1 - y`:

```text
y = +1 (topo)    ->  1 - 1    = 0  (base)
y =  0 (centro)  ->  1 - 0    = 1  (centro)
y = -1 (base)    ->  1 - (-1) = 2  (topo)
```

E depois as mesmas etapas usadas pra converter `x`.
