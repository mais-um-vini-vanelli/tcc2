# Ray tracing

## A pipeline

Os processos que o ray tracer executa pra colorir cada pixel:

- Gerar o raio primário ([Geração de raios](/raytracing/geracao-de-raios))
- Levar o raio vai pro espaço local de cada objeto ([Transformações](/fundamentos/transform))
- Testar se o raio intersepta algum objeto e escolher a interseção mais próxima ([Interseções](/raytracing/intersecoes))
- Calcular a cor direta do acerto usando as luzes que acertam o objeto sem bloqueio ([Whitted](/raytracing/whitted))
- Calcular as cores que a superfície recebe por reflexão e por refração, quando o material é translúcido ou refletor ([Whitted](/raytracing/whitted), [reflexão e refração](/raytracing/reflexao-e-refracao))

As [BVHs](/raytracing/bvh) podem ser usadas para reduzir (muito) a quantidade de testes de interseção do raio com a geometria, mas elas não afetam o resultado visual da cena.

O contraste com o rasterizador está logo no primeiro estágio: aqui se parte da
tela *para* a cena.

## As seções

- [Geração de raios](/raytracing/geracao-de-raios) — do pixel ao raio primário, nos dois modos de câmera
- [Interseções](/raytracing/intersecoes) — o raio e a esfera, o raio e o triângulo, o uv do acerto
- [Whitted](/raytracing/whitted) — termo direto, recursão, raio de sombra, acne e epsilon
- [Reflexão e refração](/raytracing/reflexao-e-refracao) — os raios secundários e seus pesos
- [BVH](/raytracing/bvh) — mediana, SAH, travessia, dois níveis
