# Câmera

A troca de coordenadas que os dois renderizadores compartilham, em sentidos
opostos: o rasterizador leva a geometria até a tela, o ray tracer parte da tela
até a cena.

## World space para camera space

A primeira troca de sistema de coordenadas: sair do mundo e passar a descrever
tudo do ponto de vista da câmera.

No mundo, a câmera é só mais um objeto: tem uma posição qualquer e olha numa
direção qualquer. Os vetores da base dela (frente, direita, cima) não precisam
estar alinhados com os eixos `x`, `y`, `z` do mundo, o que atrapalha o resto da
pipeline; o frustum de visão, por exemplo, é um volume que sai da câmera e abre
na direção pra onde ela olha, e escrever os seis planos desse volume em
coordenadas de mundo daria seis equações diferentes por cena.

Então convertemos os pontos pra um sistema de coordenadas em que a câmera está
na origem e os eixos são os eixos dela: direita, cima e frente. Aí o frustum
vira sempre o mesmo volume, e a profundidade de um ponto vira simplesmente o seu
`z`.

::: warning 🚧 TODO
Câmera inclinada, olhando pra um ponto.
:::

A conversão de um ponto do mundo tem dois passos:

1. tirar a posição da câmera do ponto: `d = ponto - camera.position()`. O vetor
   `d` vai da câmera até o ponto; ele já é o ponto visto de um sistema em que a
   câmera está em `(0, 0, 0)`, só que ainda medido nos eixos do mundo;
2. descobrir as coordenadas de `d` nos eixos da câmera, com o produto escalar:
   `(d · direita, d · cima, d · frente)`.

O segundo passo é a mudança de base. Vale lembrar por que o produto escalar
serve aqui: `d · eixo = |d| * |eixo| * cos(θ)`, e como o eixo é normalizado
(`|eixo| = 1`), sobra `|d| * cos(θ)` — o cateto adjacente do triângulo retângulo
formado por `d` e pela sua projeção naquele eixo. Ou seja, exatamente o quanto o
ponto anda naquela direção.

::: warning 🚧 TODO
World space à esquerda, camera space à direita, 1 ponto e 1 câmera nas cenas, um
vetor apontando da câmera pro ponto, posições do ponto e da câmera editáveis.
:::

Fazendo isso pros três eixos, o resultado é o mesmo ponto descrito por outras
três coordenadas: quanto ele está à direita, quanto acima e quanto à frente da
câmera. A cena não mudou de forma — nenhuma distância entre pontos foi alterada,
só a referência.

É por isso que o recorte contra o frustum acontece nesse espaço: os planos já
estão alinhados com os eixos e as coordenadas ainda são lineares, então
interpolar um vértice novo em cima de uma aresta recortada continua dando o
ponto certo.
