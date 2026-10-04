//! BVH (*bounding volume hierarchy*): uma árvore de caixas em que o raio
//! descarta de uma vez tudo o que está dentro de uma caixa que ele não cruza.
//!
//! Teoria: bvh.md
//!
//! São dois níveis com a mesma construção: uma BLAS sobre os triângulos de
//! cada `Mesh`, em espaço local, e a TLAS sobre os objetos da cena, em espaço
//! de mundo. As duas são construídas no começo de cada `render()`, cortando
//! cada nó pela mediana ou pelo SAH, conforme `AccelerationOptions.split`.

use std::collections::VecDeque;

use crate::math::{Aabb, Ray, Vec3};
use crate::parameters::{BvhSplit, TransformMethod};
use crate::raytracer::Contador;
use crate::scene::{Instance, Primitive};

// Teoria: bvh.md#quando-parar-de-dividir
/// Na mediana, a folha fecha com até este número de itens.
const ITENS_POR_FOLHA: usize = 2;

// Teoria: bvh.md#o-custo-esperado
/// O custo de testar o raio contra uma caixa, na mesma unidade do `CUSTO_PRIMITIVO`.
const CUSTO_CAIXA: f32 = 0.5;
/// O custo de testar o raio contra um triângulo, uma esfera ou um cubo.
const CUSTO_PRIMITIVO: f32 = 1.0;

// Teoria: bvh.md#os-baldes
const BALDES: usize = 12;

/// No SAH, acima disto o nó é dividido mesmo que a folha pareça mais barata.
const MAX_ITENS_POR_FOLHA: usize = 8;

// Teoria: bvh.md#dois-niveis
/// As duas árvores da cena.
pub(crate) struct Aceleracao {
    pub tlas: Bvh,
    /// Uma por instância, na ordem de `Scene::flatten`. Só malha tem.
    pub blas: Vec<Option<Bvh>>,
}

impl Aceleracao {
    pub fn build(instancias: &[Instance], split: BvhSplit, method: TransformMethod) -> Aceleracao {
        let blas: Vec<Option<Bvh>> = instancias
            .iter()
            .map(|instancia| match instancia.geometry {
                Primitive::Mesh(malha) => Some(Bvh::build(
                    (0..malha.face_count())
                        .filter_map(|i| {
                            Some(Item {
                                indice: i,
                                caixa: Aabb::around(malha.face(i)?.map(|v| v.position)),
                                custo: CUSTO_PRIMITIVO,
                            })
                        })
                        .collect(),
                    split,
                )),
                _ => None,
            })
            .collect();

        let itens = instancias
            .iter()
            .zip(&blas)
            .enumerate()
            .map(|(i, (instancia, blas))| Item {
                indice: i,
                // A caixa local girada deixa de ser alinhada aos eixos; o que vai pra TLAS é a caixa alinhada em volta dos oito cantos dela.
                caixa: Aabb::around(
                    instancia
                        .geometry
                        .bounds()
                        .corners()
                        .map(|canto| instancia.chain.apply_point(canto, method)),
                ),
                // Teoria: bvh.md#o-custo-de-um-objeto
                custo: blas.as_ref().map_or(CUSTO_PRIMITIVO, Bvh::custo_esperado),
            })
            .collect();

        Aceleracao {
            tlas: Bvh::build(itens, split),
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

    /// As AABBs dos nós da TLAS e da BLAS até `profundidade`.
    pub fn aabbs(
        &self,
        instancias: &[Instance],
        method: TransformMethod,
        profundidade: u32,
    ) -> Vec<(u32, [Vec3; 8])> {
        let tlas = self
            .tlas
            .aabbs(profundidade)
            .into_iter()
            .map(|(nivel, caixa)| (nivel, caixa.corners()));

        let blas = self
            .blas
            .iter()
            .zip(instancias)
            .filter_map(|(blas, instancia)| Some((blas.as_ref()?, instancia)))
            .flat_map(|(blas, instancia)| {
                blas.aabbs(profundidade).into_iter().map(|(nivel, caixa)| {
                    (
                        nivel,
                        caixa
                            .corners()
                            .map(|canto| instancia.chain.apply_point(canto, method)),
                    )
                })
            });

        tlas.chain(blas).collect()
    }
}

/// O que uma árvore organiza: um triângulo na BLAS, um objeto na TLAS.
#[derive(Debug, Clone, Copy)]
struct Item {
    indice: usize,
    caixa: Aabb,
    /// Quanto custa testar o raio contra ele. Só o SAH olha.
    custo: f32,
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
    fn build(mut itens: Vec<Item>, split: BvhSplit) -> Bvh {
        let mut nos = Vec::new();

        if !itens.is_empty() {
            divide(&mut nos, &mut itens, split);
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

    /// As AABBs dos nós até `profundidade`.
    fn aabbs(&self, profundidade: u32) -> Vec<(u32, Aabb)> {
        let Some(raiz) = self.raiz() else {
            return Vec::new();
        };

        let mut aabbs = Vec::new();
        let mut pendentes = VecDeque::from([(1, raiz)]);

        while let Some((nivel, i)) = pendentes.pop_front() {
            if nivel > profundidade {
                break;
            }

            aabbs.push((nivel, self.nos[i].caixa()));

            if let &No::Interno { filhos, .. } = &self.nos[i] {
                pendentes.extend(filhos.map(|filho| (nivel + 1, filho)));
            }
        }

        aabbs
    }

    // Teoria: bvh.md#o-custo-de-um-objeto
    /// O custo esperado de um raio que cruza a caixa da raiz, com todo item
    /// custando `CUSTO_PRIMITIVO`: o teste da raiz, mais o de cada nó pesado
    /// pela chance de o raio chegar nele.
    fn custo_esperado(&self) -> f32 {
        let Some(raiz) = self.raiz() else {
            return 0.0;
        };

        let area_raiz = self.nos[raiz].caixa().area();
        let chance = |caixa: Aabb| {
            if area_raiz > 0.0 {
                caixa.area() / area_raiz
            } else {
                1.0
            }
        };

        let nos: f32 = self
            .nos
            .iter()
            .map(|no| match no {
                No::Folha { caixa, itens } => chance(*caixa) * itens.len() as f32 * CUSTO_PRIMITIVO,
                No::Interno { caixa, .. } => chance(*caixa) * 2.0 * CUSTO_CAIXA,
            })
            .sum();

        CUSTO_CAIXA + nos
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

/// Empilha o nó que cobre `itens` — folha, ou interno sobre as duas partes do
/// corte que `split` escolher — e devolve o índice dele.
fn divide(nos: &mut Vec<No>, itens: &mut [Item], split: BvhSplit) -> usize {
    let caixa = itens
        .iter()
        .fold(Aabb::EMPTY, |caixa, item| caixa.union(item.caixa));

    let corte = match split {
        BvhSplit::Median => corte_na_mediana(itens),
        BvhSplit::Sah => corte_pelo_sah(itens, caixa),
    };

    let no = match corte {
        Some(meio) => {
            let (esquerda, direita) = itens.split_at_mut(meio);

            No::Interno {
                caixa,
                filhos: [divide(nos, esquerda, split), divide(nos, direita, split)],
            }
        }
        None => No::Folha {
            caixa,
            itens: itens.iter().map(|item| item.indice).collect(),
        },
    };

    nos.push(no);
    nos.len() - 1
}

// Teoria: bvh.md#construcao-pela-mediana
/// Reordena `itens` com os menores centros na metade de baixo e devolve onde
/// ela termina, ou `None` se eles cabem numa folha.
fn corte_na_mediana(itens: &mut [Item]) -> Option<usize> {
    if itens.len() <= ITENS_POR_FOLHA {
        return None;
    }

    let centros = Aabb::around(itens.iter().map(|item| item.caixa.center()));
    let eixo = eixo_mais_comprido(centros.max - centros.min);
    let meio = itens.len() / 2;

    itens.select_nth_unstable_by(meio, |a, b| {
        componente(a.caixa.center(), eixo).total_cmp(&componente(b.caixa.center(), eixo))
    });

    Some(meio)
}

// Teoria: bvh.md#construcao-pelo-sah
/// Reordena `itens` pelo corte de menor custo esperado entre os planos dos
/// baldes, nos três eixos, e devolve onde a parte de baixo termina — ou `None`
/// se a folha sai mais barata que o melhor corte.
fn corte_pelo_sah(itens: &mut [Item], caixa: Aabb) -> Option<usize> {
    if itens.len() <= 1 {
        return None;
    }

    let centros = Aabb::around(itens.iter().map(|item| item.caixa.center()));
    let balde = |eixo: usize, item: &Item| -> usize {
        let (min, max) = (componente(centros.min, eixo), componente(centros.max, eixo));
        let x = (componente(item.caixa.center(), eixo) - min) / (max - min);

        ((x * BALDES as f32) as usize).min(BALDES - 1)
    };

    // Teoria: bvh.md#os-baldes
    // Os custos ficam multiplicados pela área do pai: ela é a mesma pra todo
    // corte, e pode ser zero. `(custo, eixo, plano)`; o plano `p` passa entre
    // os baldes `p - 1` e `p`.
    let mut melhor: Option<(f32, usize, usize)> = None;

    for eixo in 0..3 {
        // Todos os centros no mesmo plano: nada a separar neste eixo.
        if componente(centros.max, eixo) <= componente(centros.min, eixo) {
            continue;
        }

        let mut baldes = [Grupo::VAZIO; BALDES];
        for item in itens.iter() {
            let b = balde(eixo, item);
            baldes[b] = baldes[b].junta(Grupo {
                caixa: item.caixa,
                custo: item.custo,
                itens: 1,
            });
        }

        // `acima[p]` junta os baldes de `p` em diante.
        let mut acima = [Grupo::VAZIO; BALDES + 1];
        for p in (0..BALDES).rev() {
            acima[p] = baldes[p].junta(acima[p + 1]);
        }

        let mut abaixo = Grupo::VAZIO;
        for p in 1..BALDES {
            abaixo = abaixo.junta(baldes[p - 1]);

            // Um lado vazio não é corte (e a área da caixa vazia é infinita).
            if abaixo.itens == 0 || acima[p].itens == 0 {
                continue;
            }

            let custo = abaixo.caixa.area() * abaixo.custo + acima[p].caixa.area() * acima[p].custo;

            if melhor.is_none_or(|(menor, ..)| custo < menor) {
                melhor = Some((custo, eixo, p));
            }
        }
    }

    if melhor.is_none() {
        return if itens.len() > MAX_ITENS_POR_FOLHA {
            Some(itens.len() / 2)
        } else {
            None
        };
    }

    let (custo, eixo, plano) = melhor.unwrap();

    // custo_folha = soma dos custos dos itens
    // custo_corte = 2·CUSTO_CAIXA + (área_esq / área_pai)·custo_esq + (área_dir / área_pai)·custo_dir
    //             = 2·CUSTO_CAIXA + (área_esq·custo_esq + área_dir·custo_dir) / área_pai
    //             = 2·CUSTO_CAIXA + custo / área_pai
    //
    // Pra evitar dividir por 0, multiplicamos os dois lados por área_pai:
    //
    // custo_corte · area = 2·CUSTO_CAIXA·area + custo
    // custo_folha · area = soma(custos) · area
    let custo_corte = 2.0 * CUSTO_CAIXA * caixa.area() + custo;
    let custo_folha = itens.iter().map(|item| item.custo).sum::<f32>() * caixa.area();

    if custo_corte >= custo_folha && itens.len() <= MAX_ITENS_POR_FOLHA {
        return None;
    }

    itens.sort_unstable_by_key(|it| balde(eixo, it) >= plano);
    let meio = itens.partition_point(|it| balde(eixo, it) < plano);

    Some(meio)
}

/// Os itens de um balde, ou de vários baldes juntos.
#[derive(Clone, Copy)]
struct Grupo {
    caixa: Aabb,
    custo: f32,
    itens: usize,
}

impl Grupo {
    const VAZIO: Grupo = Grupo {
        caixa: Aabb::EMPTY,
        custo: 0.0,
        itens: 0,
    };

    fn junta(self, outro: Grupo) -> Grupo {
        Grupo {
            caixa: self.caixa.union(outro.caixa),
            custo: self.custo + outro.custo,
            itens: self.itens + outro.itens,
        }
    }
}

fn eixo_mais_comprido(extensao: Vec3) -> usize {
    if extensao.x >= extensao.y && extensao.x >= extensao.z {
        0
    } else if extensao.y >= extensao.z {
        1
    } else {
        2
    }
}

fn componente(v: Vec3, eixo: usize) -> f32 {
    match eixo {
        0 => v.x,
        1 => v.y,
        _ => v.z,
    }
}
