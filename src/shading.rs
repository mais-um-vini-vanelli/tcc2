use crate::color::Rgb;
use crate::math::{Vec2, Vec3};
use crate::parameters::ReflectanceModel;
use crate::scene::{Light, LightInstance, MaterialSample};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surface {
    pub point: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub material: MaterialSample,
}

pub fn shade(
    surface: &Surface,
    view: Vec3,
    light_instances: &[LightInstance],
    visibility: &dyn Fn(&Surface, &LightInstance) -> f32,
    reflectance_model: ReflectanceModel,
) -> Rgb {
    let mut color = (&surface.material).emission;

    for light_instance in light_instances {
        if let Light::Environment {
            color: environment_color,
            intensity,
        } = light_instance.local
        {
            // Vem de todas as direções, então não tem direção pra testar/cosseno pra aplicar.
            color = color + surface.material.albedo * environment_color * intensity;
            continue;
        }

        let (to_light, radiance) = incident_radiance(surface.point, light_instance);

        if surface.normal.dot(to_light) <= 0.0 || radiance == Rgb::BLACK {
            // Luz chegando por trás da superfície.
            continue;
        }

        let visibility = visibility(surface, light_instance);
        if visibility <= 0.0 {
            continue;
        }

        // A fração da luz que chega por `light` que sai por `view`.
        let reflectance = match reflectance_model {
            ReflectanceModel::BlinnPhong => {
                blinn_phong(surface.normal, view, to_light, &surface.material)
            }
        };

        color = color + reflectance * radiance * visibility;
    }

    color
}

pub fn always_visible(_surface: &Surface, _light_instance: &LightInstance) -> f32 {
    1.0
}

/// Calcula a direção **para** a luz e a radiância que chega no ponto.
fn incident_radiance(point: Vec3, light_instance: &LightInstance) -> (Vec3, Rgb) {
    match light_instance.local {
        Light::Point { color, intensity } => {
            let to_light = light_instance.position - point;

            // v · v = |v|^2, então d^2 = to_light.dot(to_light).
            let d2 = to_light.dot(to_light).max(1e-6);

            // intensity já vem em watts / 4*PI, então watts por feixe = intensity / d2.
            (to_light.normalized(), color * (intensity / d2))
        }

        Light::Spot {
            inner_angle,
            outer_angle,
            color,
            intensity,
        } => {
            // Um spot é uma luz pontual com um cone por cima.
            let (to_light, radiance) = incident_radiance(
                point,
                &LightInstance {
                    local: Light::Point { color, intensity },
                    ..*light_instance
                },
            );

            let cos_theta = light_instance.forward().dot(-to_light);

            // Comparar os cossenos pra evitar um acos por ponto; o cosseno
            // diminui quando o ângulo cresce, então cos_interno > cos_externo.
            let inner_cos = inner_angle.to_radians().cos();
            let outer_cos = outer_angle.to_radians().cos();

            // 0 fora do cone externo, 1 dentro do interno, rampa entre os dois.
            let t = ((cos_theta - outer_cos) / (inner_cos - outer_cos).max(1e-4)).clamp(0.0, 1.0);

            (to_light, radiance * (t * t))
        }

        Light::Directional { color, intensity } => (-light_instance.forward(), color * intensity),

        Light::Environment { .. } => unreachable!(),
    }
}

/// Blinn-Phong como ele aparece na literatura: kd · albedo · (n · l)  +  ks · (n · h)^shininess
fn blinn_phong(n: Vec3, v: Vec3, l: Vec3, m: &MaterialSample) -> Rgb {
    let cos_theta = n.dot(l);

    let alpha = m.roughness * m.roughness;
    let shininess = (2.0 / (alpha * alpha) - 2.0).max(1.0);

    // O fresnel divide o difuso (luz que entrou na superfície e voltou espalhada) e o especular
    // (luz que voltou refletida, sem entrar na superfície).
    let fresnel = fresnel_schlick(f0(m), n.dot(v).abs());

    let diffuse = {
        let kd = (Rgb::WHITE - fresnel)
            * ((1.0 - m.metallic) * (1.0 - m.transparency) / std::f32::consts::PI);

        m.albedo * kd * cos_theta
    };

    let specular = {
        let ks = fresnel * ((shininess + 8.0) / (8.0 * std::f32::consts::PI));

        // h é o vetor do meio do caminho entre a luz e o olho: quando ele bate com a normal, o reflexo
        // especular aponta exatamente para a câmera.
        let h = (v + l).normalized();
        let ndoth = n.dot(h).max(0.0);

        ks * ndoth.powf(shininess)
    };

    diffuse + specular
}

/// A refletância de frente (`f0`): quanto da luz um material devolve como
/// especular quando ela chega perpendicular à superfície.
pub fn f0(material: &MaterialSample) -> Rgb {
    // f0 = ((n1 - n2) / (n1 + n2))², com n1 = 1 no ar.
    let n2 = material.refractive_index;
    let dieletric = ((1.0 - n2) / (1.0 + n2)).powi(2);

    Rgb::gray(dieletric).lerp(material.albedo, material.metallic)
}

/// Schlick (1994): a refletância cresce de `f0` até 1 conforme a luz passa a
/// raspar a superfície.
pub fn fresnel_schlick(f0: Rgb, cos: f32) -> Rgb {
    let cos = cos.clamp(0.0, 1.0);
    f0 + (Rgb::WHITE - f0) * (1.0 - cos).powi(5)
}
