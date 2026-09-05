# Rasterização — teoria

As explicações longas do pipeline de rasterização, na ordem em que o
pipeline as usa. Os pontos do código que dependem de cada uma apontam pra cá
com um comentário `// Teoria: rasterizer.md#secao`.

## A pipeline

Os estágios que o rasterizador percorre hoje, na ordem, e as seções que
explicam cada um:

```text
-> world space
-> camera space                    (Mundo para camera space)
-> recorte contra o frustum        (O plano, O frustum, Recorte no plano)
   Sutherland e Hodgman (1974)
-> backface culling                (Backface culling)
-> screen space                    (Camera space para NDC, NDC para pixel)
-> teste de cobertura por edge     (Edge functions)
   functions dentro da bounding
   box, Pineda (1988)
-> baricêntricas com correção de   (Edge functions, Pesos sob perspectiva)
   perspectiva, interpolando
   normal e uv
-> teste de profundidade contra
   o z-buffer
```

## Mundo para camera space

A primeira troca de sistema de coordenadas: sair do mundo e passar a descrever
tudo do ponto de vista da câmera.

No mundo, a câmera é só mais um objeto: tem uma posição qualquer e olha numa
direção qualquer. Nada nela está alinhado com os eixos x, y, z do mundo. Isso
atrapalha todo o resto do pipeline — o frustum é um volume que sai da câmera e
abre na direção pra onde ela olha, e escrever os seis planos desse volume em
coordenadas de mundo daria seis equações diferentes por cena.

A saída é escrever os pontos num sistema de coordenadas em que a câmera está na
origem e os eixos são os eixos dela: direita, cima e frente. Aí o frustum vira
sempre o mesmo volume, e a profundidade de um ponto vira simplesmente o seu z.

Os três eixos vêm de `Camera::basis()` e são ortonormais: perpendiculares dois a
dois e de comprimento 1.

```text
                 cima
                  ↑
                  |
    câmera ●──────┼──────→ direita
             ╲    |
              ╲   |
               ↘ frente (pra onde a câmera olha)
```

A conversão de um ponto do mundo tem dois passos:

```text
- tirar a posição da câmera do ponto: d = ponto - camera.position()
  d é o vetor que vai da câmera até o ponto. Ele já é o ponto visto de um
  sistema em que a câmera está em (0,0,0) — só que ainda medido nos eixos do
  mundo.
- descobrir quanto de d cabe em cada eixo da câmera, com o produto escalar:
  (d · direita, d · cima, d · frente)
```

O segundo passo é a mudança de base. Vale lembrar por que o produto escalar
serve aqui: `d · eixo = |d| * |eixo| * cos(θ)`, e como o eixo é normalizado
(`|eixo| = 1`), sobra `|d| * cos(θ)` — o cateto adjacente do triângulo retângulo
formado por d e pela sua projeção naquele eixo. Ou seja, exatamente o quanto o
ponto anda naquela direção.

```text
        d
      ↗ |
    ↗   | parte de d que não cabe no eixo
  ↗ θ   |
 ●──────┴──────→ eixo (normalizado)
 └──────┘
  d · eixo
```

Fazendo isso pros três eixos, o resultado é o mesmo ponto descrito por outras
três coordenadas: quanto ele está à direita, quanto acima e quanto à frente da
câmera. A cena não mudou de forma — nenhuma distância entre pontos foi alterada,
só o referencial. Distorção nenhuma acontece aqui; a divisão por z, que é o que
de fato deforma, vem depois, em "Camera space para NDC".

É por isso que o recorte contra o frustum acontece nesse espaço: os planos já
estão alinhados com os eixos e as coordenadas ainda são lineares, então
interpolar um vértice novo em cima de uma aresta recortada continua dando o
ponto certo.

## O plano

Antes do frustum, o objeto de que ele é feito: seis planos, e a pergunta que a
gente faz pra cada um é sempre a mesma — de que lado desse plano o ponto caiu?

Um plano em 3D fica definido por um vetor normal `n` e um ponto `p0` que
pertence a ele. Um ponto `p` qualquer só está no plano se o vetor que vai de
`p0` até `p` for perpendicular à normal:

```text
> n · (p - p0) = 0
> n · p - n · p0 = 0
```

Definindo `deslocamento = -(n · p0)`, sobra a forma que a struct guarda:

```text
> n · p + deslocamento = 0
```

Guardar `(normal, deslocamento)` em vez de `(normal, p0)` é o que faz o teste
custar um produto escalar e uma soma, sem precisar carregar um ponto de
referência.

E o valor dessa expressão, quando não dá zero, não é lixo: é a resposta de que
lado o ponto está. Voltando à primeira forma e chamando `v = p - p0`:

```text
> n · v
```

Isso é a projeção de `v` em cima da normal do plano. Com a normal normalizada, o
resultado é a distância de `p` até o plano, com sinal:

```text
p acima (lado da normal)     n · v > 0
p em cima do plano           n · v = 0   (v e n perpendiculares)
p abaixo (lado oposto)       n · v < 0
```

```text
            p ●
              ↑ distância > 0
        n ↑   |
    ──────────┴──────────  plano
              |
              ↓ distância < 0
              ● p
```

É exatamente isso que `Plane::distance` devolve, e é o que o recorte usa duas
vezes: o sinal diz se o vértice fica ou sai, e a magnitude dos dois sinais
opostos de uma aresta dá o `t` do ponto de cruzamento (ver "Recorte no plano").

## O frustum

De onde saem as seis equações de plano do frustum.

### Near e far

Pro near plane queremos a normal apontando pra frente (+z) (0, 0, 1.0)
Além disso, qualquer ponto do near plane tem a forma (0, 0, near)

O deslocamento do plano é:

```text
deslocamento = -(n · p0) = - ((0, 0, 1.0) * (0, 0, near)) = -near
```

O deslocamento é o resultado de distancia() com p = 0:

```text
distancia(0) = normal · (0,0,0) + d = d
```

Ele diz o quão pra trás ou pra frente a origem está do plano. A origem sendo a câmera no nosso caso

### Laterais sob perspectiva

A caixa de visão é uma pirâmide partindo da origem, ou seja deslocamento = 0

Olhando só pro plano de cima da pirâmide:

```text
tan(fov/2) = oposto / adjacente, com adjacente = z
```

A cada unidade de z que avanaça, o plano superior da pirâmide/o maior y visível aumenta em tan(fov/2)
Isso quer dizer que pra um ponto p estar 'dentro' na caixa ele precisa ter:

```text
p.y <= p.z * meia_altura
```

Revisando e abrindo uma equação de plano:

```text
> n · p + d >= 0
> n.x * p.x + n.y * p.y + n.z * p.z + d >= 0
```

Colocando o nosso plano na mesma forma:

```text
> p.y <= p.z * meia_altura
> -p.y + p.z * meia_altura >= 0
> 0 * p.x + (-1 * p.y) + meia_altura * p.z >= 0
```

Comparando com a fórmula aberta da equação de plano, a gente pode ver que a normal precisa ser (0, -1, meia_altura) pra
cumprir a inequação

Então o plano de cima da caixa é n = (0, -1, meia_altura), d = 0
E podemos seguir a mesma lógica pras outras faces, trocando tan(fov/2) por aspect * tan(fov/2) pras faces horizontais

### Laterais sob ortográfica

Na ortográfica a caixa não abre com a distância: as laterais são
paralelas ao eixo de visão, em x = ±width/2 e y = ±height/2.

## Recorte no plano

Onde a aresta cruza o plano: a fórmula do `t`.

Se a gente conhece as duas extremidades de uma aresta, a gente pode andar por ela com:

```text
> ponto = ponto_atual + (ponto_seguinte - ponto_atual) * t
```

Ou seja, fazendo uma interpolação linear das duas posições com t de 0 a 1:

```text
> t = 0 -> devolve ponto_atual
> t = 1 -> devolve ponto_seguinte
> t = 0.5 -> devolve o ponto médio entre ponto_atual e ponto_seguinte
```

O ponto p onde a aresta cruza o plano tem:

```text
1) plano.distancia(p) = normal · p + deslocamento = 0
2) p = atual + (proximo - atual) * t
```

Substituindo a fórmula de p em 1) temos:

```text
plano.distancia(p) = normal · [atual + (proximo - atual) * t] + deslocamento
```

Agora vamos simplificar essa fórmula

Propriedades do dot product:

```text
P1 distributiva              -> u · (v + w)  = u·v + u·w
P3 escalar (k) sai para fora -> u · (k·v)    = k (u·v)
```

Aplicando P1:

```text
> normal · [atual + (proximo - atual) * t] + deslocamento
> normal · atual + normal · [(proximo - atual) * t] + deslocamento
```

Aplicando P2:

```text
> normal · atual + t * [normal · (proximo - atual)] + deslocamento
```

Aplicando P1 denovo no [normal · (proximo - atual)]:

```text
> normal · atual + t * (normal · proximo - normal · atual) + deslocamento
```

Note que normal · atual + deslocamento = plano.distancia(atual) (o d_atual que foi calculado acima), então
trocamos:

```text
> d_atual + t * (normal · proximo - normal · atual)
```

Agora simplificando o (normal · proximo - normal · atual). Note que:

```text
> d_atual = normal · atual + deslocamento -> normal · atual = d_atual - deslocamento
> d_proximo = normal · proximo + deslocamento -> normal · proximo = d_proximo - deslocamento
```

Trocando:

```text
> d_atual + t * [(d_proximo - deslocamento) - (d_atual - deslocamento)]
> d_atual + t * (d_proximo - deslocamento - d_atual + deslocamento)
```

deslocamento e -deslocamento se cancelam:

```text
> d_atual + t * (d_proximo - d_atual)
```

Perfeito, temos a fórmula da distancia de um ponto da aresta em função de t. Agora é igualar a 0 e
resolver pra achar o ponto em que a aresta cruza o plano:

```text
> d_atual + t * (d_proximo - d_atual) = 0
> t * (d_proximo - d_atual) = -d_atual
> t = -d_atual / (d_proximo - d_atual)
```

## Camera space para NDC

Camera space -> NDC: a projeção, nos dois modos de câmera.

### Perspectiva

A área visível é um cone saindo da câmera, e abrindo mais conforme se afasta no eixo z:

```text
                              ┌── topo da tela
                     ┌─────────┘
            ┌─────────┘
  câmera ●──┴───────────────────────── eixo de visão (centro da tela)
            └─────────┐
                     └─────────┐
                              └── base da tela
         z=1        z=3        z=6
```

Perto da câmera cabe pouco mundo na tela; longe cabe muito. Objetos próximos ficam maiores,
objetos distantes ficam menores (as coordenadas x,y dos pontos ficam mais próximas de 0)

Divide o fov por 2 pra descobrir o ângulo entre o topo e a origem (ou a base e a origem, mesmo
valor). E depois pega a tangente desse ângulo, que diz *quanto a altura aumenta a cada unidade que
a gente avança em z*, já que tan() = oposto/adjacente

Agora a conversão dos pontos em camera space pra ndc
Para o eixo y:

```text
  - o ponto está em (x,y,z); y unidades de altura
  - no eixo y, o topo da tela fica a z * meia_altura unidades
  - então a coordenada y projetada vai ficar em y / topo da tela = y / (z * meia_altura)
```

Para o eixo x, não podemos usar só tan(fov/2), porque o fov é o ângulo vertical e a tela geralmente tem uma
largura diferente da sua altura (a quantidade de unidades que a caixa de visão aumenta com a distância é maior
que tan(fov/2))

O aspect já diz a proporção largura/altura, então multiplicamos tan(fov/2) (taxa de aumento da altura/unidade de z) por
aspect pra descobrir taxa de aumento da largura/unidade de z.
Se a largura da tela é 2x a altura, faz sentido o tamanho da caixa de visão crescer 2x mais rápido que a altura

Prova:
Num triângulo retângulo, tan(ângulo) = oposto / adjacente

```text
             │ ← H  (meia-altura visível: o "oposto")
câmera ●─────┘
        ← z →   (a distância: o "adjacente")
```

tan(fov_v / 2) = height / z

```text
aspect = width / height -> width = aspect * height
tan(fov_h / 2) = W / z                     (definição do fov horizontal)
               = (H × aspect) / z          (porque W = H × aspect)
               = (H / z) × aspect          (só reagrupando a multiplicação)
               = tan(fov_v / 2) × aspect   (porque H/z é o tan vertical)
```

### Ortográfica

A área visível é uma caixa de width * height
Usando o eixo x de exemplo, o centro da área fica em x = 0, e os limites em -width/2 e +width/2

Como a gente quer um fator f que converte um ponto com x = -width/2 em x = -1 em ndc E um ponto com
x = +width/2 em x = 1 em ndc, dá de transformar em sistema de equação e resolver:

```text
A) -width/2 * f = -1
B) +width/2 * f =  1
```

Isolando f na equação B, tem f = 1/(width/2). Trocando f na equação A tem -width/2 * 1/(width/2) = -1, que é
uma equação verdadeira. Então dá de usar 1/(width/2) pra converter coordenadas do eixo x de camera space em ndc,
e a mesma ideia vale pro eixo y.

Na projeção ortográfica o tamanho dos objetos não muda com a distância da câmera, então z não é usado na projeção

## NDC para pixel

NDC -> pixel: a última etapa da projeção.

Agora sim, só converter as coordenadas x, y = [-1, 1] em uma matrix de pixels x = [0, resolução (largura)] e
y = [0, resolução (altura)]

Exemplo com 320x240:

```text
operação	| faixa depois dela | o que virou
    x     -1 … 1
+ 1.0	  0 … 2	              tira o negativo, origem vai pra borda esquerda
* 0.5	  0 … 1	              vira fração da tela
* 320	  0 … 320	          vira index do pixel
```

y cresce pra cima em NDC e pra baixo no buffer, então a gente primeiro inverte a direção com 1-y:

```text
y = +1 (topo)   -> 1-1 = 0 (base)
y =  0 (centro) -> 1-0 = 1 (centro)
y = -1 (base)   -> 1-(-1) = 2 (topo)
```

e depois as mesmas etapas pra converter 0… 2 -> 0...1 e 0...1 -> 0 … 240

## Edge functions

A edge function e o dobro da área: de onde vêm as baricêntricas.

Essa função responde a pergunta:
Traçando uma reta infinita que passa por a e b, o ponto p caiu de que lado?

```text
tiramos o produto vetorial u x v = u.x·v.y − u.y·v.x
o sinal da resposta diz pra qual lado v está de u
a magnitude da resposta diz o tamanho do ângulo entre u e v (com 90 graus entre eles, o resultado chega no ápice)
```

A edge function borda(b, c, p) retorna a área do triângulo formado entre (b, c, p) * 2. A mesma coisa pra borda(c, a, p) e borda(a, b, p)

### Prova

a fórmula da área de um triângulo é base x altura / 2

```text
a edge function (a, b, p) usa os dois vetores u e v, com u = (a, b) e v = (a, p)
podemos visualizar u como sendo a base de um triângulo, e v sendo uma das arestas laterais
para descobrir a base desse triângulo precisamos de |u| * altura
altura é a distância de p até a aresta u, em outras palavras: a projeção de v no eixo perpendicular a u
```

Então pra descobrir a altura podemos:

```text
1) rotacionar u em 90º
2) normalizar u_rotacionado, dividindo pelo seu comprimento |u|
3) projetar v em u_rotacionado com um produto escalar
```

Então a altura do triângulo a, b, p é

```text
> h = (u_rotacionado / |u|) · v
```

Podemos reorganizar a fórmula:

```text
> h = (u_rotacionado · v) / |u|
> h * |u| = u_rotacionado · v
```

Se temos um vetor u = (x, y), a fórmula desse vetor rotacionado em 90º é u'= (-y, x)
Isso pode ser visto em:

```text
(1, 0)  →  (0, 1) "direita" vira "cima"
(0, 1)  →  (−1, 0) "cima" vira "esquerda"
```

Então u_rotacionado = (-u.y, u.x):

```text
> h * |u| = (-u.y, u.x) · v
```

Desenvolvendo o produto escalar e reorganizando:

```text
> |u| * h = -u.y * v.x + u.x * v.y
```

Agora olha que legal: à esquerda da equação ficou base (|u|) * altura, e à direita ficou a fórmula da edge function
Considerando que a área de um triângulo é base * altura / 2, dá de ver que o resultado da edge function é o dobro da área do triângulo a, b, p

### Da área para o peso de interpolação

Se p está dentro do triângulo, a soma das áreas dos subtriângulos abp, bcp e acp precisa ser igual à area do triangulo abc
Isso pode ser visto intuitivamente
[TODO: desenho de um triângulo com P dentro e os 3 subtriângulos]

E quanto maior o subtriângulo formado entre uma aresta de abc e p, mais perto p está do ponto aposto à aresta, e mais longe ele está dos
outras 2 pontos, que formam a aresta.
Isso também pode ser visto intuitivamente
[TODO: imagem mostrando p em cima de um ponto (subtriângulo = abc), p no meio de uma aresta, p no meio do triângulo]

Outra forma de ver isso é comparando o subtriângulo com o triângulo completo:

- pedaço (b,c,p) → base bc, altura = distância de p até a reta bc
- triângulo (a,b,c) → base bc (a mesma!), altura = distância de a até a reta bc

Os dois têm a mesma base. E quando dois triângulos compartilham a base, a razão entre as áreas é a razão entre as alturas:

```text
       base·h_p / 2        h_p        distância de p até a aresta bc
 l0 = ─────────────────  = ─────  =  ────────────────────────────────
       base·h_a / 2        h_a        distância de a até a aresta bc
```

Ou seja: l0 mede o quanto p está longe da aresta oposta, em relação ao quanto o vértice a está.
E se l0 = 1, p está tão longe de bc quanto a: p está em cima de a

Então podemos usar a proporção (área do subtriangulo)/(área do triângulo) para descobrir o quão perto p está de cada vértice, e usar isso como
coeficiente de interpolação

## Pesos sob perspectiva

Por que os pesos medidos na tela servem direto sob projeção ortográfica e
precisam de conserto sob perspectiva.

A seção anterior mostrou como sair das edge functions pros pesos: cada peso é a
razão entre a área de um subtriângulo e a área do triângulo inteiro. Só que essa
medida foi feita na tela, depois da projeção — e o que a gente quer interpolar
(normal, uv, cor, profundidade) vive no mundo, antes dela.

### Ortográfica: nada a fazer

A projeção ortográfica é afim, e transformações afins preservam razões ao longo
de uma reta: o ponto médio continua o ponto médio, o ponto a 1/3 continua a 1/3.
Como consequência preservam também razões entre áreas — e peso baricêntrico é
exatamente uma razão entre áreas. Ou seja: o peso medido na tela já é o peso no
mundo, é só interpolar.

É o mesmo caminho que o aluno percorre quando desliga `perspective_correct` sob
perspectiva — e aí a distorção clássica aparece na profundidade e nos atributos
ao mesmo tempo.

### Perspectiva: a razão não sobrevive à divisão por z

A projeção perspectiva não é afim, porque cada ponto é dividido pelo seu próprio
`z` (variável, não constante). Ela mantém as retas, mas distorce as proporções
ao longo delas.

Uma aresta de `A = (x=-1, z=1)` até `B = (x=1, z=5)`. O ponto médio `P`, antes de
projetar, sai da interpolação normal:

```text
> (-1, 1) + ((1, 5) - (-1, 1)) * 0.5 = (x=0, z=3)

A = (x=-1, z=1)
P = (x= 0, z=3)
B = (x= 1, z=5)
```

Projetando os três (dividindo x por z):

```text
A' = (-1.0, 1)
P' = ( 0.0, 3)
B' = ( 0.2, 5)
```

A distância em x de A->P e de P->B era 1 dos dois lados antes de projetar; depois
da projeção, a metade mais longe ocupa um pedaço bem menor da tela. É o mesmo
efeito que faz o corredor parecer estreitar ao longe.

E é por isso que interpolar direto na tela erra: tentando achar P a partir de A'
e B' com t = 0.5,

```text
> (-1.0, 1) + ((0.2, 5) - (-1.0, 1)) * 0.5 = (x=-0.4, z=3)   *X errado*
```

o resultado não é `P'`.

### O conserto: interpolar em 1/z

O que anda em passo constante na tela não é `z`, é `1/z`. Então os pesos são
levados pra esse espaço, somados e renormalizados:

```text
inversos = [l0/z0, l1/z1, l2/z2]
z        = 1 / (inversos[0] + inversos[1] + inversos[2])
pesos    = inversos * z
```

A soma dos inversos é o `1/z` do próprio ponto, então invertê-la já devolve a
profundidade dele. E o peso corrigido de cada vértice é a fatia dele nesse `1/z`,
renormalizada pra somar 1 de novo. Interpolar qualquer atributo com esses pesos
é andar em passo constante pelo triângulo **no mundo**, não pela tela.

A profundidade sai junto de graça: interpolar os próprios `z` com esses pesos
devolve exatamente o mesmo `z` calculado acima, porque

```text
∑ (lᵢ/zᵢ)·z·zᵢ = z·∑ lᵢ = z
```

[TODO: demonstrar direito por que é 1/z que interpola linearmente na tela.]

## Backface culling

O sinal da área diz que lado do triângulo está virado pra câmera.

Sobre o backface culling:
Daria pra usar as normais informadas dos vértices pra saber qual o lado da frente do triângulo,
mas a convenção é usar a ordem dos vértices (winding order) pra isso: os vértices das faces de fora de um objeto
são enviados em ordem anti-horária e as faces de dentro (por consequência) em ordem horária

Se o triângulo é formado pelos pontos A B e C, tenho dois jeitos de avançar partindo de A:

```text
A
|\
| \
B--C
```

A -> B -> C e A -> C -> B.

Um desses sentidos vai ser horário e o outro anti-horário.
Note que se os pontos B e C do triângulo estivessem trocados, o sentido das travessias ABC e ACB trocariam, mas uma delas
continuaria sendo horária e a outra anti-horária.

Se a gente olhar pro mesmo triângulo de trás (espelhado na horizontal) o sentido das travessias se inverte:

```text
    A
   /|
  / |
 /  |
C---B
```

Considerando um triângulo ABC achatado no plano XY, o produto vetorial entre as arestas AB e AC dá um vetor perpendicular às duas apontando
'pra frente' (pra fora da tela) ou 'pra trás' (pra dentro da tela), dependendo da ordem das arestas:

```text
A
|\
| \
B--C
```

```text
A = (0, 1, 0)
B = (0, 0, 0)
C = (1, 0, 0)
```

```text
B - A = (0, -1, 0)
C - A = (1, -1, 0)
```

```text
AB x AC = ((-1 * 0) - (0 * -1), (0 * 1) - (0 * 0), (0 * -1) - (-1 * 1)) = (0, 0, 1)
```

Ou seja, uma normal apontando pra +z. Agora invertendo a ordem das arestas:

```text
AC x AB = ((-1 * 0) - (0 * -1), (0 * 0) - (1 * 0), (1 * -1) - (-1 * 0)) = (0, 0, -1)
```

Ou seja, uma normal apontando pra -z.

Note que quando as arestas do produto vetorial estão em sentido horário, o resultado do produto
vetorial tem z positivo (vetor saindo da tela), e quando estão em sentido anti-horário o resultado é um vetor entrando na tela (com z negativo)

Note também que a edge function que a gente já usa é exatamente a fórmula do componente z de um produto vetorial entre dois vetores 3d:

```text
A.x * B.y - A.y * B.x
```

Então a gente pode usar o sinal da área calculada pela edge function pra saber qual lado do triângulo projetado em 2d está virada para a câmera

Mas como a projeção ndc -> tela inverte o eixo y (espelha o triângulo em y), olhamos se a área é negativa ao invés de positiva
