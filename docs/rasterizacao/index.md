# Rasterização

## A pipeline

Os processos que o rasterizador executa pra desenhar um triângulo na tela:

- Converter os primitivos da cena em malha de triângulos ([Tesselação](/rasterizacao/tesselacao))
- Converter os vértices de local space para world space ([Transformações](/fundamentos/transform))
- Converter os vértices de world space para camera space ([Câmera](/fundamentos/camera))
- Fazer o clipping contra o frustrum ([Frustum e recorte](/rasterizacao/frustum-e-recorte))
- Fazer backface culling ([O triângulo](/rasterizacao/triangulo))
- Converter os vértices de camera space para screen space ([Projeção](/rasterizacao/projecao))
- Testar quais pixels estão dentro do triângulo com edge functions ([O triângulo](/rasterizacao/triangulo))
- Cálcular as coordenadas baricêntricas para interpolar atributos dos vértices, como normal e uv ([O triângulo](/rasterizacao/triangulo))
- Fazer o depth test contra o z-buffer

O contraste com o ray tracer está logo no primeiro estágio: aqui a geometria vai
*até* a tela.

## As seções

- [Tesselação](/rasterizacao/tesselacao) — o cubo e a esfera uv em triângulos
- [Frustum e recorte](/rasterizacao/frustum-e-recorte) — o plano, o frustum, Sutherland-Hodgman
- [Projeção](/rasterizacao/projecao) — camera space para NDC, NDC para pixel
- [O triângulo](/rasterizacao/triangulo) — edge functions, pesos, onde a cor é avaliada, backface culling
