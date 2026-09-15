# Produtos/projetos correlatos — Software Renderer Didático

Pesquisa na web por soluções prontas que resolvam o mesmo problema do projeto de TCC2
(biblioteca/software renderer didático em Rust, com API única alternável entre
rasterização e ray tracing). Feita em 2026-08-24.

## Conclusão geral

Não foi encontrado nenhum projeto pronto — em Rust ou em qualquer outra linguagem —
que combine rasterização e ray tracing em uma única API didática alternável, como
proposto no TCC2. O padrão encontrado é:

- ou é um rasterizador puro,
- ou é um ray/path tracer puro,
- ou é uma ferramenta de visualização sem API de programação,
- ou é conteúdo de ensino (lições/tutoriais) com programas de exemplo separados
  para cada técnica, sem biblioteca unificadora.

Isso reforça o diferencial do projeto proposto: unificar os dois pipelines sob uma
única API simples, algo não encontrado em nenhum concorrente direto.

## Renderizadores software didáticos (fazem só um lado)

- **[TinyRenderer (ssloy)](https://github.com/ssloy/tinyrenderer)** — curso clássico
  em C++, só rasterização (~500 linhas). É um curso passo-a-passo, não uma
  biblioteca com API própria.
- **[Nori (EPFL / Wenzel Jakob)](https://github.com/wjakob/nori)** — framework usado
  em cursos de graduação (EPFL, ETHZ, Cornell), só ray/path tracing, em C++. Os
  próprios autores citam PBRT e Mitsuba como grandes demais para fins didáticos,
  motivando a criação do Nori.
- **[SoftGLRender](https://github.com/keith2018/SoftGLRender)** — rasterizador
  software em C++ que compara resultado com OpenGL/Vulkan; só rasterização.
- **[Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html)**
  (Shirley et al.) — livro/série popular no ensino, mas só path tracer em C++, sem
  rasterização; não é biblioteca com API pública, é código para copiar/adaptar.

## Ferramentas educacionais interativas (visualização, não biblioteca)

- **[Rayground](https://diglib.eg.org/items/9b9e8067-c850-493e-ab28-2fce1612cd80)** —
  ferramenta web WebGL para ensinar ray tracing via shaders; foco em visualização
  em sala, não em API de programação.
- **[Virtual Ray Tracer 2.0](https://www.sciencedirect.com/science/article/pii/S0097849323000067)** —
  app gamificado para visualizar o processo de ray tracing em tempo real; também
  não é uma lib para os alunos programarem contra.

## Ray tracers em Rust (nenhum com modo rasterização)

Todos implementam só path/ray tracing (a maioria seguindo RTIOW), nenhum expõe uma
API que alterne para rasterização:

- [dps/rust-raytracer](https://github.com/dps/rust-raytracer)
- [ranveeraggarwal/rust-raytracer](https://github.com/ranveeraggarwal/rust-raytracer)
- [matklad/rustraytracer](https://github.com/matklad/rustraytracer)
- [reecelikesramen/rust-raytracer](https://github.com/reecelikesramen/rust-raytracer)
- [tray_rust](https://github.com/Twinklebear/tray_rust)

## Scratchapixel

**[Scratchapixel](https://www.scratchapixel.com/)** — site/curso gratuito de
Computação Gráfica que cobre tanto
[rasterização](https://www.scratchapixel.com/lessons/3d-basic-rendering/rasterization-practical-implementation//overview-rasterization-algorithm.html)
quanto [ray tracing](https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-overview//ray-tracing-rendering-technique-overview.html),
com implementações passo a passo em C++, cobrindo praticamente os mesmos tópicos do
TCC2: edge functions, coordenadas baricêntricas, câmera, interseção raio-triângulo,
aceleração espacial, shading físico etc.

É o correlato mais próximo do projeto, mas com diferenças importantes:

1. **Não é uma biblioteca/API** — é uma coleção de lições/artigos, cada uma com um
   programa C++ standalone e didático próprio. Não existe um pacote único
   importável com algo como `render(scene, mode: Raster | RayTrace)`.
2. **Rasterização e ray tracing são trilhas separadas** — não há alternância entre
   os dois modos sobre a mesma cena/API; são exemplos de código independentes,
   cada lição com seu próprio mini-programa.
3. Sem Rust, sem BVH como estrutura central, sem objetivo de "produto"
   reutilizável — é puramente conteúdo de ensino (mais parecido com um livro-texto
   interativo do que com um software/biblioteca).

Reforça o mesmo gap identificado nos outros achados: cobre os dois paradigmas em
conteúdo educacional, mas ninguém empacota isso como uma biblioteca com API única
e alternável — que é o núcleo da proposta do TCC2.
