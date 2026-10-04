# Geração de raios

Como nasce o raio primário de um pixel `(x, y)`, nos dois modos de câmera. É o caminho
inverso da [Projeção](/rasterizacao/projecao) do rasterizador.

### Do pixel para NDC

O pixel é um índice inteiro, mas o raio tem que passar pelo *centro* dele, então somamos meio píxel. 
Ex: o pixel 1,1 tem centro em (1.5, 1.5).

Depois as coordenadas viram NDC (`[-1, 1]`). É o caminho inverso do "NDC para pixel" do rasterizador.

Exemplo com 320x240, no eixo x:

```text
operação	| range
x             0 … 320
/ largura     0 … 1
* 2           0 … 2
- 1          -1 … 1
```

No eixo y a coordenada cresce pra baixo no buffer e pra cima em NDC, então tiramos o complemento com `1 - y` no final.

```text
y             0 … 240
/ altura      0 … 1
* 2           0 … 2
1 - y        -1 … 1  (invertido)

y = 2 (base da imagem) -> 1-2 = -1
y = 1 (centro)         -> 1-1 =  0
y = 0 (topo da imagem) -> 1-0 =  1
```

### Perspectiva: a tela virtual

Imagine uma tela virtual a 1 unidade pra frente da câmera. Todos os raios saem
do mesmo ponto (a posição da câmera) e passam pelo ponto dessa tela que
corresponde ao pixel.

O tamanho dessa tela vem do fov. Dividindo o fov por 2 acha-se o ângulo entre o
topo e o eixo de visão (ou entre a base e o eixo, mesmo valor); a tangente desse
ângulo diz *quanto a altura aumenta a cada unidade que se avança em z*, já que
`tan() = oposto/adjacente`. Como a tela está a 1 unidade, a meia-altura dela é a
própria tangente.

```text
                        ^  topo da tela
                      / |
                    /   |
      fov/2       /     |  meia_altura = tan(fov/2) * 1
câmera  ) - - - - - - - +  centro da tela
          \       \     |
            \       \   |
              \       \ |
                        v  base da tela
      |<---- 1.0 ------>|
```

::: warning 🚧 TODO
Desenho 3D da tela virtual, mostrando o ângulo, z= 1 e com slider de altura.
:::

Pensando em 2D, as coordenadas (`u`, `v`) do plano variam entre -1 e 1 conforme a função é chamada pra
cada pixel da imagem. Multiplicando por `meia_altura`, o pixel com `v = 1` cai
em `meia_altura` no plano virtual, o com `v = -1` cai em `-meia_altura`, e os
intermediários interpolam linearmente. Multiplicar esse valor pelo vetor `cima`
da câmera dá o deslocamento no eixo vertical em 3D.

No eixo horizontal entra o aspect, porque o fov guardado é o vertical:

```text
aspect = width / height  ->  width = height * aspect

tan(fov_h / 2) = width / z                     
               = (height * aspect) / z     # porque width = height * aspect
               = (height / z) * aspect
               = tan(fov_v / 2) * aspect   # porque height/z = tan(fov_v / 2)
```

Então o deslocamento horizontal é `direita * (u * meia_altura * aspect)`.

::: warning 🚧 TODO
Desenho 2D da tela virtual com (u, v) à esquerda, desenho 3D com a tela no espaço à direita.
:::

Somando os três deslocamentos — 1 unidade pra frente, o vertical e o horizontal
— sai um único vetor apontando da câmera pro pixel: a direção do raio. A origem
é a posição da câmera, igual pra todos os pixels.

### Ortográfica: a origem é que varia

Na projeção ortográfica é o contrário: a **direção** é a mesma em todos os
pixels (o vetor `frente` da câmera) e a **origem** é que muda, alinhada com o
pixel. É a diferença entre um cone de raios saindo de um ponto e um feixe de
raios paralelos.

`u` e `v` vão de -1 a 1 (tamanho 2, uma unidade pra cada lado), mas o volume de
visão é `width` x `height` — metade pra cada lado em cada eixo. Então
multiplicar
`u` por `width * 0.5` e `v` por `height * 0.5` converte a coordenada normalizada
no deslocamento em unidades de mundo.

Exemplo com `width = 320`:

```text
 u  | u * (width * 0.5)
-1    -160 (borda esquerda)
 0       0 (centro)
+1    +160 (borda direita)
```

Esse valor é a coordenada 2D do pixel em espaço de câmera, então ainda precisa
ser multiplicado pelos vetores `direita` e `cima`, que carregam a orientação em
3D:

```text
origem = position + direita * (u * width * 0.5) + cima * (v * height * 0.5)
```

