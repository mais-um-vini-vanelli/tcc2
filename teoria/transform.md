# Transform — teoria

Como um objeto sai do espaço em que a geometria dele foi escrita e vai parar
onde a cena quer que ele esteja. Os pontos do código que dependem de cada seção
apontam pra cá com um comentário `// Teoria: transform.md#secao`.

É a primeira seção de teoria que vale pros **dois** pipelines, e por um motivo
que importa: rasterizador e ray tracer resolvem o mesmo problema em sentidos
opostos.

## O transform

A geometria de um primitivo é escrita em **espaço local**: o cubo tem centro na
origem e aresta `2 * half_edge`, a esfera tem centro e raio, o triângulo tem
três pontos. Nada disso sabe onde o objeto está na cena.

Quem sabe é o `Transform` do `Object`, com três campos:

```text
scale        quanto o objeto estica em cada eixo
rotation     três ângulos de Euler, em graus, na ordem X, Y, Z
translation  pra onde o objeto vai
```

A identidade (`scale = 1`, `rotation = 0`, `translation = 0`) é o `Default`, e
é por isso que não existe `Option<Transform>` na API: "objeto sem transform" e
"objeto com o transform que não faz nada" são a mesma coisa, e uma delas é uma
palavra a menos pra explicar.

O que essas três operações **não** fazem também é conteúdo: não há cisalhamento,
não há projeção, não há reflexão pedida por nome. É o subconjunto que se chama
de transformação de corpo rígido mais escala — o que basta pra montar cena, e o
que cabe em três campos que um aluno lê sem consultar tabela.

### Por que os campos do primitivo não somem

`Sphere` continua tendo `center` e `radius` mesmo com `translation` e `scale`
existindo do lado. Os dois níveis não competem: o do primitivo é **geometria em
espaço local**, e o do objeto vem por cima. Uma esfera de raio 2 com escala 3 é
uma esfera de raio 6 no mundo, e as duas formas de escrever isso são
igualmente válidas.

A regra de composição é sempre a mesma: **primeiro a geometria local, depois o
transform**.

## A ordem das três etapas

O ponto vai pelas etapas nesta ordem, e nenhuma outra:

```text
p_local --escala--> --rotação--> --translação--> p_mundo
```

As duas trocas que parecem inofensivas quebram a cena:

**Transladar antes de rotacionar.** A rotação gira em torno da origem do
sistema. Se o objeto já foi movido pra longe dela, a rotação não gira o objeto:
ela põe o objeto em órbita.

```text
   ordem certa (rotaciona, depois translada)   ordem trocada (translada, depois rotaciona)

        y                                            y
        |     ___                                    |            ___
        |    /   \  <- objeto girado no lugar        |           /   \
        |    \___/                                   |    o      \___/  <- objeto foi parar
   -----+---------- x                           -----+---------------- x   do outro lado
        |                                            |
```

**Escalar depois de rotacionar.** A escala age nos eixos do sistema em que ela é
aplicada. Depois da rotação, os eixos do objeto não são mais os eixos do mundo,
então esticar em `x` estica numa direção que não é a "largura" do objeto — o
resultado é cisalhamento, e um cubo vira um paralelepípedo torto.

Escalando **antes**, a escala pega o objeto ainda alinhado com os próprios
eixos, que é o que o usuário quis dizer com "achata esse eixo".

## Girar em torno de um eixo

Pra girar ao redor do eixo X, giramos o ponto pelo plano YZ que passa por ele.

Todo ponto (y, z) pode ser escrito em termos de uma distância até a origem (o raio r) e um ângulo θ a partir do eixo y:

y = r·cos(θ)
z = r·sin(θ)

Se eu giro esse ponto por um ângulo adicional a, o novo ponto (y', z') tem o mesmo raio r (girar não muda a distância até a origem), mas o ângulo agora é θ + a:

y' = r·cos(θ + a)
z' = r·sin(θ + a)

Considere as identidades de soma de ângulos:

cos(θ + a) = cos(θ) * cos(a) - sin(θ) * sin(a)
sin(θ + a) = sin(θ) * cos(a) + cos(θ) * sin(a)

Substituindo:

y' = r·[cos(θ)cos(a) - sin(θ)sin(a)] = r·cos(θ)cos(a) - r·sin(θ)sin(a)
z' = r·[sin(θ)cos(a) + cos(θ)sin(a)] = r·sin(θ)cos(a) + r·cos(θ)sin(a)

Trocando y = r·cos(θ) e z = r·sin(θ) do passo 1:

y' = y * cos(a) - z * sin(a)
z' = z * cos(a) + y * sin(a)

A mesma lógica vale para a rotação ao redor dos outros eixos.

### Euler XYZ

Três ângulos, aplicados em sequência: primeiro em torno de X, depois de Y,
depois de Z. Escrito como composição de matrizes fica `Rz · Ry · Rx`, que se lê
da direita pra esquerda — a última matriz escrita é a primeira aplicada.

A ordem importa: rotação não comuta. Girar 90° em X e depois 90° em Y não leva
ao mesmo lugar que girar 90° em Y e depois 90° em X. Por isso "ângulos de Euler"
sempre vem com uma ordem colada no nome, e a daqui é a mesma do Blender.

O preço dos ângulos de Euler é o *gimbal lock*: quando a rotação do meio (aqui,
Y) chega em 90°, o primeiro e o terceiro eixo passam a girar em torno da mesma
direção, e um grau de liberdade some. Quatérnions resolvem, e não estão aqui —
o que se ganha com Euler é poder digitar `rotation: Vec3::new(0.0, 45.0, 0.0)`
e saber o que vai acontecer.

## A normal usa a escala recíproca

O ponto e a normal saem do mesmo vértice e vão pro mesmo mundo, mas **não
passam pelo mesmo transform**. Essa é a assimetria mais fácil de errar do
módulo, e a mais fácil de ver quando erra: a iluminação fica torta em objeto
achatado, e certa em objeto de escala uniforme — o que faz o bug passar
despercebido justamente nas cenas de teste mais simples.

### O desenho

Achatando um círculo pela metade em `y`:

```text
   antes                        depois, com a normal errada

        ^ n                             ^ n  <- continua apontando pra 45°,
       /                               /        mas a superfície não está
     _/                              _/         mais a 45°
   /                               /
   ----                        ---------
```

A superfície achatou e ficou **mais deitada**; a normal, escalada do mesmo
jeito, também ficou mais deitada. Mas a normal é perpendicular à superfície, e
perpendicular a algo mais deitado é algo mais **em pé**. Os dois deviam ter
girado em sentidos opostos, e escalar os dois igual gira ambos pro mesmo lado.

Dito de outro jeito: a intuição de "a normal acompanha a superfície" está certa;
o que está errado é achar que "acompanhar" quer dizer "sofrer a mesma conta".
Sob escala não uniforme, ângulo não se preserva — e a normal é definida por um
ângulo.

### A conta, com números

Uma superfície a 45° no plano XY, escala `s = (1, 0.5, 1)`.

O que a superfície tem de concreto no ponto é o vetor **tangente**: a direção em
que ela anda. Aqui `t = (1, 1, 0)`, e a normal é `n = (-1, 1, 0)` —
perpendicular, `n · t = -1 + 1 = 0`.

O tangente é geometria de verdade, então ele anda junto com o objeto:

```text
t' = S t = (1, 0.5, 0)
```

Escalando a normal do mesmo jeito:

```text
n_errada = S n = (-1, 0.5, 0)
n_errada · t' = -1 + 0.25 = -0.75      != 0, não é mais perpendicular
```

Escalando pelo recíproco `(1, 2, 1)`:

```text
n' = S⁻¹ n = (-1, 2, 0)
n' · t' = -1 + 1 = 0                   perpendicular de novo
```

### A conta, em geral

O mesmo argumento sem números. Procuramos a matriz `M` que leva a normal, tal
que `n' = M n` continue perpendicular ao tangente transformado `t' = S t`:

```text
n' · t' = 0
(M n) · (S t) = 0
(M n)ᵀ (S t) = 0            Reescrevendo notação. Produto escalar de dois vetores coluna é a · b = aᵀ b (a vira linha e multiplica matricial)
nᵀ Mᵀ S t = 0               Reescrevendo notação. (A B)ᵀ = Bᵀ Aᵀ
```

A última linha diz: `Mᵀ S` tem que transformar todo vetor perpendicular a `n`
em algo que continua perpendicular a `n`. O jeito de garantir isso
para todo `n` e todo `t` de uma vez é `Mᵀ S = I`, ou seja:

```text
M = (S⁻¹)ᵀ
```

Como a escala é diagonal, transpor não muda nada e sobra `M = S⁻¹`: a normal é
escalada pelo **recíproco** da escala do ponto.

```text
ponto    escala por (sx, sy, sz)
normal   escala por (1/sx, 1/sy, 1/sz)
```

## O raio vai para o espaço local

Aqui está o contraste que justifica os dois pipelines existirem na mesma API.

O rasterizador percorre vértices, então transformar a geometria é natural: cada
vértice passa por `apply_point`, cada normal por `apply_normal`, e daí em diante
o pipeline nem sabe que houve transform.

O ray tracer não percorre vértices — a esfera dele não tem vértices. Tesselar a
esfera só pra poder transformá-la jogaria fora uma das vantagens do ray tracer, que é intersectar a superfície exata. Então ele faz o contrário:
**leva o raio pro espaço local do objeto**, com `undo_ray`, e intersecta lá,
contra a esfera não transformada.

```text
rasterizador                      ray tracer

  geometria --T--> mundo            mundo --T⁻¹--> geometria
  (o objeto vai até a cena)         (a cena vai até o objeto)
```

O resultado é o mesmo desenho, e de graça vem uma capacidade que a tesselação
não daria: com escala não uniforme, a esfera analítica vira **elipsoide** sem
ganhar um triângulo sequer. O rasterizador chega no mesmo elipsoide por outro
caminho, deformando os vértices da esfera tesselada.

### A direção não é normalizada, e é de propósito

`Ray::new` normaliza a direção. `undo_ray` **não** usa `Ray::new`: ele monta o
raio local com a direção transformada crua.

O motivo é que o `t` da interseção é medido em unidades da direção do raio.
Sendo o transform afim, ele leva a reta do raio na reta do raio local e preserva
a razão entre distâncias ao longo dela — então, com a direção local mantida
"esticada" pelo mesmo fator que a escala aplicou, o mesmo `t` vale nos dois
espaços:

```text
p(t)      = o + t d           no mundo
p_local(t) = T⁻¹(o) + t · T⁻¹(d)     no espaço local, mesmo t
```

## A cadeia da árvore

Até aqui cada objeto tinha **um** transform, e a cena era uma lista plana. Com o
scene graph a cena vira árvore: cada nó tem o transform dele, relativo ao pai, e
o que leva a geometria de uma folha até o mundo é a sequência de transforms dali
até a raiz.

```text
raiz
 └── grupo        T_g   (gira 24° em Y)
      └── cubo    T_c   (escala 1, 1.3, 1)
           └── esfera   T_e

um ponto local da esfera vai pro mundo assim:

p_mundo = T_g( T_c( T_e(p) ) )
```

`Scene::flatten` percorre a árvore uma vez e devolve uma **instância** por nó com
geometria. A instância não copia a geometria — aponta pra ela — e carrega a
cadeia `[T_e, T_c, T_g]`, guardada **do nó para a raiz**. Nó sem geometria some
no achatamento: ele existe só pelo transform, e o transform dele continua vivo
como um elo na cadeia dos filhos.

### Por que a cadeia não vira um transform só

O reflexo é compor os elos num `Transform` só e voltar pro caso de sempre. Não
dá: `Transform` é um TRS, escala → rotação → translação nessa ordem, e o conjunto
dos TRS **não é fechado** sob composição.

O contraexemplo é o desenho acima. O pai gira, o filho tem escala não uniforme,
e a composição é

```text
R_g · (T_c ∘ S_c)
```

onde a escala do filho acontece **antes** da rotação do pai. Um TRS só sabe
escalar antes de girar, e aqui é preciso girar (o filho), escalar de novo... a
ordem alterna, e o que sai é uma matriz com *shear* — um paralelepípedo torto,
que nenhum `(translation, rotation, scale)` descreve.

A saída padrão é guardar uma `Mat4`, que é fechada sob composição justamente
porque aceita o shear. Mas `Mat4` foi deixada de fora de propósito em M1.5: a
ordem das três etapas fica explícita no código, e `apply_normal` fica sendo a
escala recíproca em vez da inversa transposta de uma matriz.

Guardar o **array** de elos preserva as duas coisas. Cada elo continua sendo um
TRS legível, e a composição vira um laço:

```text
apply_point   p, do nó para a raiz:   elo.apply_point(p)   pra cada elo, em ordem
undo_point    p, da raiz para o nó:   elo.undo_point(p)    pra cada elo, ao contrário
apply_normal  n:                      a escala recíproca, elo a elo
```

O preço é um laço curto por ponto em vez de uma multiplicação de matriz. Numa
árvore de editor a cadeia tem dois ou três elos, e o custo some perto do resto
do render.

### A normal, elo a elo

`Transform::apply_normal` normaliza no fim. Aplicá-lo elo a elo normaliza no
meio do caminho — e isso não muda a direção que sai, porque escala e rotação são
lineares: `S(k·v) = k·S(v)`. O comprimento intermediário é jogado fora de
qualquer jeito.

### Câmera e luz também são nós

A cena inteira mora na árvore: um nó ou tem geometria, ou é a câmera, ou é uma
luz — ou não é nada e só agrupa.

O que não muda de lugar é `position`/`look_at` da câmera e `position`/`direction`
da luz. Eles continuam onde sempre estiveram, só que agora querem dizer
**local ao nó**, e o `flatten` aplica a cadeia neles antes de entregar:

```text
Light::Point { position: (1,0,0) }   pendurada num pai girado 90° em Y
                                      e transladado para (2,3,-1)

           (1,0,0)  --gira-->  (0,0,-1)  --translada-->  (2,3,-2)
```

Num nó de raiz com transform identidade a cadeia não faz nada, e local é igual a
mundo — que é exatamente o que a cena era antes de existir hierarquia. Por isso o
`shade()` nunca soube, e continua não sabendo: o que chega nele é uma `Light` de
mundo, como sempre foi.

O que a cadeia **não** alcança é o que não é ponto nem direção: `fov`, `near` e
`far`, `width` e `height`, os ângulos do cone do spot. São escalares, e uma escala
na cadeia não tem como entrar neles. A direção da câmera escapa de graça, porque
sai de `look_at − position` e é renormalizada em `basis()`. O glTF tem o mesmo
buraco, e resolve desaconselhando escalar um nó de câmera.

### O ray tracer continua andando na contramão

Nada muda no contraste dos dois pipelines; só o número de passos. O rasterizador
leva o vértice pro mundo subindo a cadeia, e o ray tracer traz o raio pro espaço
local da folha descendo-a:

```text
rasterizador                          ray tracer

  p --T_e--> --T_c--> --T_g--> mundo    mundo --T_g⁻¹--> --T_c⁻¹--> --T_e⁻¹--> p
```

E o `t` continua valendo nos dois espaços pelo mesmo motivo de antes: cada
`undo_direction` da cadeia deixa a direção esticada pelo fator que a escala
daquele elo aplicou, então a razão entre distâncias ao longo da reta se mantém
em todos eles.
