//! Tipos geométricos básicos.

use std::ops::{Add, Div, Mul, Neg, Sub};

/// Vetor de duas dimensões. Usado para coordenadas de textura (uv) e para
/// posições em espaço de tela.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    // Produto escalar.
    //
    // Significado:
    // - Mede o quanto dois vetores estão alinhados. Ele retorna:
    //   - valores positivos se o ângulo entre eles é < 90º
    //   - valor = 0 se o ângulo entre eles é = 90º
    //   - valores negativos se o ângulo entre eles é > 90º (quando eles apontam em direções opostas)
    // - Se um dos vetores é unitário, o resultado é a projeção de um dos vetores no outro.
    pub fn dot(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    // Produto vetorial ou cruzado.
    //
    // Essa operação não existe propriamente em 2D, mas pode ser simulada extendendo os dois vetores pra 3D com z = 0; ou seja tratando eles como vetores 3D 'deitados' no plano XY.
    //
    // Significado:
    // - O resultado dessa operação é a área do paralelogramo formado entre os dois vetores no plano.
    // - Se o resultado é positivo, o vetor 3D resultante apontaria pra 'fora da tela'; ou seja os vetores self x other estão em sentido anti-horário.
    pub fn cross(self, other: Vec2) -> f32 {
        Vec3::new(self.x, self.y, 0.0)
            .cross(Vec3::new(other.x, other.y, 0.0))
            .z
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, s: f32) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }
}

/// Vetor de três dimensões. Pode ser ponto, direção ou normal.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Vec3 = Vec3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    // Produto escalar.
    //
    // Significado:
    // - Mede o quanto dois vetores estão alinhados. Ele retorna:
    //   - valores positivos se o ângulo entre eles é < 90º
    //   - valor = 0 se o ângulo entre eles é = 90º
    //   - valores negativos se o ângulo entre eles é > 90º (quando eles apontam em direções opostas)
    // - Se um dos vetores é unitário, o resultado é a projeção de um dos vetores no outro.
    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    // Produto vetorial ou cruzado.
    //
    // Significado:
    // - O resultado dessa operação é um vetor perpendicular a self e other.
    // - O comprimento do vetor resultante é igual à área do paralelogramo formado entre os dois vetores no plano.
    // - O sentido do vetor resultante depende da ordem de self e other. Se você posicionar os dedos da mão direita no sentido de self e fechá-los em direção a other, o seu polegar apontará o sentido do resultado.
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// Vetor de mesma direção e comprimento 1.
    pub fn normalized(self) -> Vec3 {
        let len = self.length();
        if len == 0.0 { Vec3::ZERO } else { self / len }
    }

    /// Menor valor em cada eixo.
    pub fn min(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x.min(o.x), self.y.min(o.y), self.z.min(o.z))
    }

    /// Maior valor em cada eixo.
    pub fn max(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x.max(o.x), self.y.max(o.y), self.z.max(o.z))
    }

    /// Converte pra Vec2, descartando o z.
    pub fn xy(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, s: f32) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        v * self
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, s: f32) -> Vec3 {
        Vec3::new(self.x / s, self.y / s, self.z / s)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

/// Raio: origem e direção
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// A direção é normalizada na construção.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Ray {
            origin,
            direction: direction.normalized(),
        }
    }

    /// Devolve o ponto que está a t unidades da origem do raio.
    pub fn at(self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

// Teoria: rasterizer.md#o-plano
/// Plano em 3D pela forma `n · p + deslocamento = 0`, com `deslocamento` sendo
/// `-(n · p0)` para um ponto `p0` qualquer do plano.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub displacement: f32,
}

impl Plane {
    pub fn new(normal: Vec3, displacement: f32) -> Self {
        Plane {
            normal,
            displacement,
        }
    }

    /// Distância com sinal: zero em cima do plano, positivo do lado da normal,
    /// negativo do lado oposto.
    pub fn distance(&self, p: Vec3) -> f32 {
        self.normal.dot(p) + self.displacement
    }
}

/// Um vértice e os atributos que viajam junto com ele pelo pipeline.
///
/// `position` troca de espaço no caminho (mundo -> câmera -> tela); `normal` e
/// `uv` ficam como estão (só são ajustados na interpolação do triângulo).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

impl Vertex {
    /// O vértice a `t` do caminho entre os dois, com todo atributo andando pelo
    /// mesmo `t`.
    pub fn lerp(self, outro: Vertex, t: f32) -> Vertex {
        Vertex {
            position: self.position + (outro.position - self.position) * t,
            normal: self.normal + (outro.normal - self.normal) * t,
            uv: self.uv + (outro.uv - self.uv) * t,
        }
    }
}

// Teoria: rasterizer.md#pesos-sob-perspectiva
/// Os pesos de interpolação de um ponto de dentro de um triângulo. Somam 1.
///
/// A construção diz sob qual projeção os pesos foram medidos: sob perspectiva
/// eles precisam ser corrigidos antes de interpolar qualquer coisa.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Baricentrics {
    weights: [f32; 3],
}

impl Baricentrics {
    /// `screen` são as edge functions divididas pela área do triângulo. Sob
    /// projeção afim eles já são os pesos no mundo.
    pub fn orthographic(screen: [f32; 3]) -> Self {
        Baricentrics { weights: screen }
    }

    /// `screen` são as edge functions divididas pela área do triângulo;
    /// `depths`, o z de cada vértice em espaço de câmera.
    pub fn perspective(screen: [f32; 3], depths: [f32; 3]) -> Self {
        let [l0, l1, l2] = screen;
        let [z0, z1, z2] = depths;

        let inversos = [l0 / z0, l1 / z1, l2 / z2];
        let z = 1.0 / (inversos[0] + inversos[1] + inversos[2]);

        Baricentrics {
            weights: inversos.map(|inverso| inverso * z),
        }
    }

    pub fn interpolate_f32(&self, valores: [f32; 3]) -> f32 {
        valores[0] * self.weights[0] + valores[1] * self.weights[1] + valores[2] * self.weights[2]
    }

    pub fn interpolate_vec2(&self, valores: [Vec2; 3]) -> Vec2 {
        valores[0] * self.weights[0] + valores[1] * self.weights[1] + valores[2] * self.weights[2]
    }

    pub fn interpolate_vec3(&self, valores: [Vec3; 3]) -> Vec3 {
        valores[0] * self.weights[0] + valores[1] * self.weights[1] + valores[2] * self.weights[2]
    }
}
