# BVH

## Por que acelerar

No raytracing cada pixel gera um raio primário, cada raio primário gera raios de sombra e (possivelmente) 
raios secundário, cada raio secundário pode gerar novos raios de sombra e raios secundários e cada raio 
precisa testar se intercepta alguma geometria da cena inteira (e qual foi a interseção mais próxima).

```text
320 × 240 pixels  ×  ~3 raios por pixel (primário + sombras)  ×  100 mil triângulos
  ≈ 2 × 10¹⁰ testes de Möller-Trumbore
```

Um método para acelerar busca é montar a BVH: os primitivos que estão perto uns dos outros são agrupados, 
e cada grupo é vira uma caixa. Se o raio não cruza a caixa, nada lá dentro pode ser
acertado, e um único teste descarta o grupo inteiro. A mesma ideia das *bounding boxes* em 2D.

Pra otimizar ainda mais, as caixas são organizadas em uma hierarquia — a *bounding volume hierarchy* —
uma árvore com caixas cada vez menores até chegar nas folhas. A mesma ideia das árvores de busca.

```text
render():
  -> uma BLAS por mesh: organiza as faces e trabalha em espaço local
  -> a TLAS da cena: organiza os objetos e trabalha em espaço de mundo

para cada raio:
  -> desce a TLAS, só recorrendo quando o raio intersecta a caixa
  -> na folha, o raio vai pro espaço local do objeto
  -> desce a BLAS
  -> se intercepta uma folha, faz o teste de interseção com a geometria
```

::: warning 🚧 TODO Alto, estático
Desenho de uma BVH.
:::

## As bounding boxes

As caixas são *axis-aligned bounding boxes* (AABB): os 6 números descrevendo o menor e o maior valor em 
cada eixo.

Ela não é a que melhor se ajusta ao objeto - uma caixa orientada (OBB) abraça
melhor um objeto com curvatura - mas custa pouca memória, o teste contra o
raio é barato, e é simples de juntar com outras caixas: o mínimo dos mínimos e o
máximo dos máximos.

Cada primitivo sabe a própria caixa, em espaço local (`Primitive::bounds`):

```text
Triangle   mínimo e máximo dos três vértices
Mesh       mínimo e máximo de todos os vértices
Sphere     centro ± raio, nos três eixos
Cube       centro ± meia aresta
```

A caixa vazia é a que tem `min = +∞` e `max = -∞`: juntá-la a qualquer outra
devolve a outra, então ela é o ponto de partida de toda união.

## O teste dos slabs

Kay e Kajiya (1986). Uma AABB é a interseção de três *fatias* (*slabs*): em
cada eixo, a região entre os dois planos paralelos que a limitam. Um plano
`x = k` é cruzado pelo raio `O + t * D` quando `O.x + t * D.x = k`, ou seja
`t = (k - O.x) / D.x`. Nos dois planos do eixo `x`:

```text
t0 = (min.x - O.x) / D.x
t1 = (max.x - O.x) / D.x
```

e está dentro da fatia `x` entre esses dois `t`. Se `D.x` é negativo o raio
cruza o `max` antes do `min`, então a entrada é o menor dos dois e a saída o
maior. Estar dentro da caixa é estar dentro das três fatias (6 planos) **ao mesmo tempo**,
ou seja, na interseção dos três intervalos:

```text
t_entrada = max(entrada_x, entrada_y, entrada_z)
t_saida   = min(saida_x,   saida_y,   saida_z)

acertou a caixa se T_entrada ≤ t_saida
```

Em 2D, com o raio subindo para a direita:

<BvhFig2 />

A entrada na caixa é a **última** entrada (na fatia `y`), e a saída é a
**primeira** saída (da fatia `x`).
Se o raio sair de uma fatia antes de entrar na outra, a entrada fica depois 
da saída, e ele não cruza a caixa.

Três detalhes que o código herda de graça:

- **O intervalo começa em `t_min..t_max`, não em `-∞..∞`.** Assim a mesma
  comparação descarta a caixa que está atrás da origem do raio e a que está
  depois do acerto mais próximo achado até ali.
- **A divisão por zero é proposital.** Se a direção tem componente 0 em um
  eixo, é porque o raio passa paralelo a ele; então ele só está dentro dos dois
  planos se a origem está dentro dos dois planos. A divisão é de `f32` e segue a
  IEEE 754: divisão por 0 não causa panic e devolve `±∞` dependendo do sinal do
  numerador. Visualize os planos `x_min = 0` e `x_max = 1`, e um raio com
  `D.x = 0`:

  ```text
  dentro,  O.x = 0.5 : (-∞, ∞)
  fora(-), O.x = -1  : (∞, ∞)
  fora(+), O.x = 2   : (-∞, -∞)
  ```

Note que só o caso em que o raio está dentro dos planos devolve `(-∞, ∞)`, que
são valores neutros de `t`; não deixam o intervalo mais restritivo.

## Construção pela mediana

A construção é *top-down*: todos os itens começam numa caixa só (a raiz), que é
dividida em dois grupos, e cada grupo é dividido de novo, até sobrar pouca geometria em
cada um.

```text
divide(itens):
    caixa = união das caixas dos itens

    se poucos itens -> vira folha
    senão:
      eixo = o mais comprido da caixa dos centros
      corta na mediana dos centros naquele eixo
      divide(metade de baixo), divide(metade de cima)
```

O eixo é escolhido a partir dos centros e o corte é na mediana. Esse método 
balanceia a **contagem** de itens, não o espaço. 

::: warning 🚧 TODO Alto, estático
Imagem de caso ruim.
:::

E os filhos podem se sobrepor. A BVH divide **objetos**, não o espaço: cada item
está em exatamente uma folha, mas as caixas de dois irmãos podem ocupar a mesma
região.

### Quando parar de dividir

Na mediana, a folha fica com até dois itens (`ITENS_POR_FOLHA`). É uma
escolha: descer um nível custa dois testes de caixa, e deixar um item a mais
na folha custa um teste de primitivo a mais.

O número é o mesmo nos dois níveis, e é uma simplificação: um item da TLAS é um
objeto inteiro (às vezes uma BLAS inteira), bem mais caro que o triângulo de
uma BLAS.

## Construção pelo SAH

A mediana escolhe o corte olhando só a contagem. O *Surface Area Heuristic*
(MacDonald e Booth, 1990) escolhe o corte que deixa a travessia mais barata, e
para isso precisa de uma estimativa do quanto um raio custa. A recursão, o formato da
árvore e a travessia são os mesmos da mediana; muda só a pergunta que `divide`
faz em cada nó.

### O custo esperado

Um raio que chega num nó `P` já cruzou a caixa dele. A chance de ele cruzar
também a caixa de um filho `F`, que está dentro de `P`, é a razão entre as
áreas de superfície:

```text
chance(cruza F | cruza P) = área(F) / área(P)
```

Vem de um resultado de geometria (teorema de Cauchy): a área da sombra de um
convexo, na média sobre todas as direções, é um quarto da área da superfície
dele.

Então o custo das duas escolhas em um nó é:

```text
se é folha: Σ custo(item)
se é corte: 2 * C_caixa + área(E)/área(P) * Σ custo(E) + área(D)/área(P) * Σ custo(D)

2 * C_caixa -> custo de testar as duas AABBs dos filhos
área(E)/área(P) * Σ custo(E) -> chance de um raio cruzar o filho esquerdo * o custo de testar
                                os itens dele
área(D)/área(P) * Σ custo(D) -> a mesma coisa pro filho direito                            
```

Cada nó fica com o corte mais barato e, se nem ele sai mais barato que a folha,
**vira folha**. É o critério de parada que faltava na mediana: em vez de um
número fixo de itens, a folha fecha quando dividir deixa de compensar.

Triângulos amontoados, com caixas quase iguais, fecham numa folha só, porque os
dois filhos teriam a área do pai e o corte só somaria o `2 * C_caixa`.

### Os baldes

Os cortes candidatos são planos perpendiculares a um eixo. Um plano entre cada
par de itens vizinhos exigiria ordenar os itens nos três eixos em cada nó. A
versão com baldes (Wald, 2007) troca isso por uma aproximação:

```text
para cada eixo:
    divide os centros em 12 baldes iguais
    para cada item: 
      descobre o balde dele e soma caixa, custo e contagem
    percorre os baldes de trás para a frente: 
      acima[p] = baldes de p em diante
    percorre os baldes: 
      abaixo = baldes antes de p
      calcula o custo do corte com abaixo[p] e acima[p]
      salva o melhor corte
fica o (eixo, plano) mais barato
```

### O custo de um objeto

Na TLAS o item é um objeto inteiro. Uma esfera ou um cubo custam um teste de
primitivo; uma malha custa uma travessia da BLAS dela. A BLAS é construída antes
da TLAS, então esse custo sai da própria árvore, com a mesma chance por área:

```text
custo(malha) = C_caixa  # raiz
             + Σ internos  área(nó)/área(raiz) · 2 * C_caixa
             + Σ folhas    área(nó)/área(raiz) * itens * C_primitivo
```

Com isso a TLAS não junta uma malha pesada numa folha com
outros objetos, a menos que as caixas digam que compensa.

## A travessia

```text
percorre(raio):
    limite = t_max
    pilha  = [raiz]                

    enquanto houver nó na pilha:
        tira um nó
        
        se o raio não intersecta ou intersecta depois do limite -> pula

        folha:    testa cada item; se acertou antes do limite -> limite = t do acerto
        interno:  testa a caixa dos dois filhos, empilha os que o raio cruza
```

Um ponto importante é o **limite encolhendo**: cada acerto diminui o intervalo
válido de `t`, então as caixas que começam depois do acerto mais próximo nem precisam
ser testadas.

## Dois níveis

Uma BVH única com todos os objetos da cena em espaço de mundo obrigaria a
transformar cada vértice para o mundo. Isso é exatamente o que o ray tracer evita ao
levar o raio para o espaço local e é por isso que se usam os dois tipos de BVH juntos:

- **BLAS** (*bottom-level*): uma por `Mesh`, sobre as faces dela, em espaço
  **local**.
- **TLAS** (*top-level*): sobre os `Object` da cena, em espaço de **mundo**.

```text
                  TLAS (mundo)
                 ╱            ╲
            [caixa]          [caixa]
            ╱     ╲               ╲
        objeto    objeto          objeto
        (malha)   (esfera)        (malha)
           │         │               │
        undo_ray  undo_ray        undo_ray
           │         │               │
         BLAS     intersect        BLAS
        (local)    direto         (local)
```

Na folha da TLAS o raio passa pelo `undo_ray`, como na força bruta, e daí a
malha desce a própria BLAS e os outros primitivos (esfera, cubo, triângulo
solto) chamam `Primitive::intersect` direto.

Como `undo_ray` não normaliza a direção, o `t` de um acerto é o mesmo no espaço local e 
no de mundo, então o acerto achado na BLAS de um objeto é usado pra podar caixas da TLAS
e vice-versa.

Os nomes vêm das APIs de ray tracing em hardware (DirectX Raytracing, Vulkan),
que dividem as *acceleration structures* exatamente assim.
