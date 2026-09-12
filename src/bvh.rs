//! BVH (*bounding volume hierarchy*): uma árvore de caixas em que o raio
//! descarta de uma vez tudo o que está dentro de uma caixa que ele não cruza.
//!
//! Teoria: bvh.md
//!
//! São dois níveis com a mesma construção: uma BLAS sobre os triângulos de
//! cada `Mesh`, em espaço local, e a TLAS sobre os objetos da cena, em espaço
//! de mundo. As duas são construídas no começo de cada `render()`.

use crate::math::{Aabb, Ray, Vec3};
use crate::raytracer::Contador;
use crate::scene::{Primitive, Scene};

// Teoria: bvh.md#quando-parar-de-dividir
const ITENS_POR_FOLHA: usize = 2;

// Teoria: bvh.md#dois-niveis
/// As duas árvores da cena.
pub(crate) struct Aceleracao {
    pub tlas: Bvh,
    /// Uma por objeto, na ordem de `Scene.objects`. Só malha tem.
    pub blas: Vec<Option<Bvh>>,
}

impl Aceleracao {
    pub fn build(scene: &Scene) -> Aceleracao {
        let blas = scene
            .objects
            .iter()
            .map(|objeto| match &objeto.geometry {
                Primitive::Mesh(malha) => Some(Bvh::build(
                    (0..malha.face_count())
                        .filter_map(|i| Some((i, Aabb::around(malha.face(i)?.map(|v| v.position)))))
                        .collect(),
                )),
                _ => None,
            })
            .collect();

        // A caixa local girada deixa de ser alinhada aos eixos; o que vai pra TLAS é a caixa alinhada em volta dos oito cantos dela.
        let caixas = scene
            .objects
            .iter()
            .map(|objeto| {
                Aabb::around(
                    objeto
                        .geometry
                        .bounds()
                        .corners()
                        .map(|canto| objeto.transform.apply_point(canto)),
                )
            })
            .enumerate();

        Aceleracao {
            tlas: Bvh::build(caixas.collect()),
            blas,
        }
    }

    // Teoria: bvh.md#as-duas-profundidades
    /// A profundidade de cada nível, separada: a da TLAS e a da BLAS mais
    /// funda. Fundidas num número só elas viram uma cota, não a profundidade.
    pub fn depth(&self) -> (u32, u32) {
        let blas = self
            .blas
            .iter()
            .flatten()
            .map(|blas| blas.depth())
            .max()
            .unwrap_or(0);

        (self.tlas.depth(), blas)
    }
}

pub(crate) struct Bvh {
    /// Cada nó vem depois dos seus filhos, então a raiz é o último.
    nos: Vec<No>,
}

enum No {
    Folha { caixa: Aabb, itens: Vec<usize> },
    Interno { caixa: Aabb, filhos: [usize; 2] },
}

impl No {
    /// A caixa é o que os dois têm; a travessia testa o nó sem olhar qual é.
    fn caixa(&self) -> Aabb {
        match *self {
            No::Folha { caixa, .. } | No::Interno { caixa, .. } => caixa,
        }
    }
}

impl Bvh {
    /// `caixas` tem um `(item, caixa)` por item, e o item é o que as folhas
    /// entregam pra quem percorre.
    pub fn build(mut caixas: Vec<(usize, Aabb)>) -> Bvh {
        let mut nos = Vec::new();

        if !caixas.is_empty() {
            divide(&mut nos, &mut caixas);
        }

        Bvh { nos }
    }

    fn raiz(&self) -> Option<usize> {
        self.nos.len().checked_sub(1)
    }

    /// Nós no caminho mais longo da raiz até uma folha.
    pub fn depth(&self) -> u32 {
        fn desce(bvh: &Bvh, i: usize) -> u32 {
            match &bvh.nos[i] {
                No::Folha { .. } => 1,
                &No::Interno { filhos: [a, b], .. } => 1 + desce(bvh, a).max(desce(bvh, b)),
            }
        }

        match self.raiz() {
            Some(raiz) => desce(self, raiz),
            None => 0,
        }
    }

    // Teoria: bvh.md#a-travessia
    /// O item mais próximo entre `t_min` e `t_max`, com o que `testa` devolveu
    /// sobre ele. `testa` recebe o item e o limite atual — que só encolhe — e
    /// responde `(t, acerto)` se o raio o acerta; guardar o melhor até agora e
    /// encolher o limite é da travessia, não dela.
    pub fn percorre<H>(
        &self,
        raio: Ray,
        t_min: f32,
        t_max: f32,
        contador: &Contador,
        mut testa: impl FnMut(usize, f32) -> Option<(f32, H)>,
    ) -> Option<(usize, H)> {
        let raiz = self.raiz()?;

        contador.no();

        let entrada = self.nos[raiz].caixa().hit(raio, t_min, t_max)?;

        let mut melhor = None;
        let mut limite = t_max;

        // Os nós que falta visitar, com o `t` em que o raio entra em cada um.
        let mut pendentes = vec![(raiz, entrada)];

        while let Some((i, entrada)) = pendentes.pop() {
            // O limite pode ter encolhido desde que o nó entrou na pilha.
            if entrada >= limite {
                continue;
            }

            match &self.nos[i] {
                No::Folha { itens, .. } => {
                    for &item in itens {
                        let Some((t, acerto)) = testa(item, limite) else {
                            continue;
                        };

                        if t >= t_min && t < limite {
                            limite = t;
                            melhor = Some((item, acerto));
                        }
                    }
                }

                &No::Interno { filhos, .. } => {
                    let [a, b] = filhos.map(|filho| {
                        contador.no();

                        let entrada = self.nos[filho].caixa().hit(raio, t_min, limite)?;
                        Some((filho, entrada))
                    });

                    // Teoria: bvh.md#o-filho-mais-perto-primeiro
                    match (a, b) {
                        (Some(a), Some(b)) => {
                            let (perto, longe) = if a.1 <= b.1 { (a, b) } else { (b, a) };
                            pendentes.push(longe);
                            pendentes.push(perto);
                        }
                        (Some(um), None) | (None, Some(um)) => pendentes.push(um),
                        (None, None) => {}
                    }
                }
            }
        }

        melhor
    }
}

// Teoria: bvh.md#construcao-pela-mediana
/// Empilha o nó que cobre `itens` — folha se forem poucos, senão interno sobre
/// as duas metades da mediana — e devolve o índice dele.
fn divide(nos: &mut Vec<No>, itens: &mut [(usize, Aabb)]) -> usize {
    let caixa = itens
        .iter()
        .fold(Aabb::EMPTY, |caixa, (_, item)| caixa.union(*item));

    let no = if itens.len() <= ITENS_POR_FOLHA {
        No::Folha {
            caixa,
            itens: itens.iter().map(|(item, _)| *item).collect(),
        }
    } else {
        ordena_pelo_eixo_mais_comprido(itens);

        let meio = itens.len() / 2;
        let (esquerda, direita) = itens.split_at_mut(meio);

        No::Interno {
            caixa,
            filhos: [divide(nos, esquerda), divide(nos, direita)],
        }
    };

    nos.push(no);
    nos.len() - 1
}

fn ordena_pelo_eixo_mais_comprido(itens: &mut [(usize, Aabb)]) {
    let eixo_mais_comprido = |extensao: Vec3| -> usize {
        if extensao.x >= extensao.y && extensao.x >= extensao.z {
            0
        } else if extensao.y >= extensao.z {
            1
        } else {
            2
        }
    };

    let componente = |v: Vec3, eixo: usize| -> f32 {
        match eixo {
            0 => v.x,
            1 => v.y,
            _ => v.z,
        }
    };

    let centros = Aabb::around(itens.iter().map(|(_, caixa)| caixa.center()));
    let eixo = eixo_mais_comprido(centros.max - centros.min);

    itens.sort_unstable_by(|(_, a), (_, b)| {
        componente(a.center(), eixo).total_cmp(&componente(b.center(), eixo))
    });
}
