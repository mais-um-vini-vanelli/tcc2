---
layout: home
hero:
  name: 'TCC<span class="caret">2</span>'
  tagline: Rasterização e ray tracing em Rust, com a teoria de cada estágio.
      
features:
  - title: Rasterização
    details: Projetar a geometria para a tela, recortar e cobrir triângulos.
    link: /rasterizacao/
  - title: Ray tracing
    details: Partir da tela para a cena, e seguir o raio até onde ele levar.
    link: /raytracing/
---

## Os dois modos, e o contraste entre eles

Os dois renderizadores resolvem o mesmo problema — *que cor tem cada pixel* — e
dividem a mesma descrição de cena, o mesmo `transform`, o mesmo modelo de
*shading*. Mas os algoritmos divergem em alguns pontos, entre eles:

- O rasterizador projeta a geometria *para* a tela, percorre objetos e descobre que pixels
eles cobrem; o ray tracer parte da tela *para* a cena, percorre pixels e descobre que objetos eles encontram. É o
assunto de [Câmera e projeção](/fundamentos/camera).

- O rasterizador leva a geometria até o mundo usando o transform; o ray tracer
traz o raio até a geometria, pela inversa do transform. É o que permite usar fórmulas arbitrárias pra especificar geometria 
durante o ray tracing, como a da esfera, independente da deformação informada no transform. É o assunto de 
[Transformações](/fundamentos/transform).

- Por simular o caminho dos raios de luz, o ray tracer consegue simular os fenômenos de sombra, reflexo e refração por padrão, enquanto o
rasterizador precisa de técnicas próprias pra isso — mapa de sombra, mapa de ambiente, renders extras do ponto de vista do espelho.

## Como ler essa documentação

Começar por [Rasterização](/rasterizacao/) ou [Ray tracing](/raytracing/), consultando as seções relevantes de [Fundamentos](/fundamentos/camera) quando necessário.

Os pontos do código que dependem de cada seção apontam pra cá com um comentário
`// Teoria: <página>#<seção>`.
