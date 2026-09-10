Para uma matriz 3×3 cujas colunas são os vetores a, b, c:
> det(a, b, c) = volume com sinal do paralelepípedo gerado por a, b e c

       ┌────────────┐
      ╱           ╱│
     ╱           ╱ │
    └────────────┘  ╱      volume = det(a, b, c)
    │   c       │╱
────●─────────────      a e b na base, c na altura
    ╱ a
    b

Propriedades:
1) É linear em cada coluna, separadamente. Se você duplica a, o volume dobra; se a = a' + a'', o volume é a soma dos dois volumes. As outras colunas ficam paradas:
> det(k·a, b, c)      = k · det(a, b, c)
> det(a' + a'', b, c) = det(a', b, c) + det(a'', b, c)

2) Duas colunas iguais → zero. O paralelepípedo achata num plano e o volume some.
> det(a, a, c) = 0

3) Trocar duas colunas inverte o sinal. Ele guarda a orientação, não só o tamanho.

Agora voltando pro problema:

┌            ┐ ┌    ┐   ┌                        ┐      ┌    ┐        ┌    ┐        ┌    ┐
│ a₁  b₁  c₁ │ │ x₁ │   │ a₁·x₁ + b₁·x₂ + c₁·x₃  │      │ a₁ │        │ b₁ │        │ c₁ │
│ a₂  b₂  c₂ │ │ x₂ │ = │ a₂·x₁ + b₂·x₂ + c₂·x₃  │ = x₁ │ a₂ │  +  x₂ │ b₂ │  +  x₃ │ c₂ │  = x₁·a + x₂·b + x₃·c
│ a₃  b₃  c₃ │ │ x₃ │   │ a₃·x₁ + b₃·x₂ + c₃·x₃  │      │ a₃ │        │ b₃ │        │ c₃ │
└            ┘ └    ┘   └                        ┘      └    ┘        └    ┘        └    ┘

Uma multiplicação matriz x vetor pode ser vista como uma combinação linear das colunas da matriz, com os componentes do vetor como coeficientes.

Pegando a equação de moller trombore como exemplo, dá de ver que é a mesma coisa:
> T = t·(−D) + u·E1 + v·E2
Um jeito de ler isso é: T é uma combinação dos vetores -D, E1 e E2, pesados por t, u e v.

Para descobrir quando T é u·E1 por exemplo, se a gente pode aplicar uma operação dos dois lados da equação que cancele -D e E2, podemos descobrir 
o valor de u. 
Essa operação é: f(w) = det( −D ,  w , E2 ). Note:
(a) f(−D) = 0. Porque det(−D, −D, E2) tem duas colunas iguais.
(b) f(E2) = 0. Porque det(−D, E2, E2) tem duas colunas iguais.
(c) f(E1) = det(M). É a matriz original, sem troca nenhuma.
(d) f é linear: f(w₁ + w₂) = f(w₁) + f(w₂) e f(k·w) = k·f(w)

Aplicando na equação:
T = t·(−D) + u·E1 + v·E2
f(T) = f( t·(−D) + u·E1 + v·E2 )
f(T) = f(t·−D) + f(u·E1) + f(v·E2)
f(T) = t·f(−D) + u·f(E1) + v·f(E2)
f(T) = t·0 + u·det(M) + v·0
u = f(T) / det(M)
u = det(-D, T, E2) / det(M)

Repetindo o processo pra t e v:
t = det( T, E1, E2) / det(M)
v = det(−D, E1,  T) / det(M)