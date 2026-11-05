use crate::{
    framebuffer::{to_rgb, Framebuffer, ProjectedVertex},
    hud,
    mesh::{self, Mesh},
    shader::{self, Body, Uniforms},
};
use glam::{Quat, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scene {
    System,
    Star,
    Rocky,
    Gas,
}

impl Scene {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "sistema" | "system" => Some(Self::System),
            "estrella" | "star" => Some(Self::Star),
            "rocoso" | "rocky" => Some(Self::Rocky),
            "gaseoso" | "gas" => Some(Self::Gas),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::System => "sistema",
            Self::Star => "estrella",
            Self::Rocky => "rocoso",
            Self::Gas => "gaseoso",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Settings {
    pub scene: Scene,
    pub time: f32,
    pub layers: u8,
    pub hud: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            scene: Scene::System,
            time: 0.0,
            layers: 4,
            hud: true,
        }
    }
}

pub struct Renderer {
    pub sphere: Mesh,
    pub moon: Mesh,
    pub rings: Mesh,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            sphere: mesh::sphere(128, 80),
            moon: mesh::sphere(64, 40),
            rings: mesh::rings(256, 1.28, 1.98),
        }
    }

    pub fn render(&self, frame: &mut Framebuffer, settings: Settings) {
        background(frame);
        frame.depth.fill(f32::INFINITY);
        let overview = settings.scene == Scene::System;
        let scale = (frame.height as f32 / if overview { 6.4 } else { 5.8 })
            .min(frame.width as f32 / if overview { 10.24 } else { 8.0 });
        let bodies: &[(Body, f32, f32)] = match settings.scene {
            Scene::System => &[
                (Body::Star, -3.15, 0.96),
                (Body::Rocky, 0.0, 0.87),
                (Body::Gas, 3.0, 0.82),
            ],
            Scene::Star => &[(Body::Star, 0.0, 1.64)],
            Scene::Rocky => &[(Body::Rocky, -0.25, 1.52)],
            Scene::Gas => &[(Body::Gas, 0.0, 1.23)],
        };
        let tilt = Quat::from_rotation_z(-0.34) * Quat::from_rotation_x(0.53);
        for &(body, x, radius) in bodies {
            let center = Vec3::new(x, 0.16, 0.0);
            if body == Body::Star {
                glow(frame, center, radius, scale);
            }
            let rotation = match body {
                Body::Star => {
                    Quat::from_rotation_z(0.06) * Quat::from_rotation_y(settings.time * 0.09)
                }
                Body::Rocky => {
                    Quat::from_rotation_z(-0.12) * Quat::from_rotation_y(settings.time * 0.12)
                }
                _ => tilt * Quat::from_rotation_y(settings.time * 0.1),
            };
            let u = Uniforms {
                body,
                time: settings.time,
                layers: settings.layers,
                center,
                radius,
                ring_normal: tilt * Vec3::Y,
            };
            draw(frame, &self.sphere, center, rotation, radius, scale, u);
            if body == Body::Gas {
                draw(
                    frame,
                    &self.rings,
                    center,
                    tilt,
                    radius,
                    scale,
                    Uniforms {
                        body: Body::Ring,
                        ..u
                    },
                );
            }
            if body == Body::Rocky {
                // Malla esférica y transformación independientes. Órbita lenta y rotación propia.
                let orbit = settings.time * 0.08;
                let offset = Vec3::new(
                    1.62 * orbit.cos(),
                    0.95 * orbit.cos() + 0.22 * orbit.sin(),
                    1.75 * orbit.sin(),
                );
                let moon_center = center + offset * radius;
                draw(
                    frame,
                    &self.moon,
                    moon_center,
                    Quat::from_rotation_y(settings.time * 0.18),
                    radius * 0.20,
                    scale,
                    Uniforms {
                        body: Body::Moon,
                        center: moon_center,
                        radius: radius * 0.20,
                        ..u
                    },
                );
            }
        }
        if settings.hud {
            overlay(frame, settings);
        }
    }
}

fn draw(
    frame: &mut Framebuffer,
    mesh: &Mesh,
    center: Vec3,
    rotation: Quat,
    radius: f32,
    scale: f32,
    uniforms: Uniforms,
) {
    let vertices: Vec<_> = mesh
        .vertices
        .iter()
        .map(|v| {
            let world = rotation * v.position * radius + center;
            ProjectedVertex {
                screen: Vec3::new(
                    frame.width as f32 * 0.5 + world.x * scale,
                    frame.height as f32 * 0.5 - world.y * scale,
                    -world.z,
                ),
                object: v.position,
                world,
                normal: rotation * v.normal,
            }
        })
        .collect();
    // Recorrido de triángulos por índices heredado de lab-4; color por fragmento.
    for indices in &mesh.faces {
        let triangle = indices.map(|i| vertices[i]);
        // Los anillos tienen dos caras. En esferas se omiten caras totalmente posteriores.
        if uniforms.body != Body::Ring && triangle.iter().all(|v| v.normal.z < 0.0) {
            continue;
        }
        frame.triangle(triangle, |f| shader::shade(f, &uniforms));
    }
}

fn background(f: &mut Framebuffer) {
    for y in 0..f.height {
        for x in 0..f.width {
            let nx = x as f32 / f.width as f32 - 0.5;
            let ny = y as f32 / f.height as f32 - 0.5;
            let haze = (1.0 - (nx * nx + ny * ny) * 1.6).max(0.0);
            f.pixels[y * f.width + x] =
                to_rgb(Vec3::new(0.012, 0.022, 0.042) + Vec3::new(0.01, 0.018, 0.026) * haze);
        }
    }
    // Semilla y posiciones normalizadas fijas: capturas reproducibles a cualquier resolución.
    let mut state = 0x4f315247u32;
    for i in 0..380 {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let x = ((state as u64 * f.width as u64) >> 32) as usize;
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let y = ((state as u64 * f.height as u64) >> 32) as usize;
        let color = if i % 7 == 0 { 0x859fae } else { 0x344759 };
        hud::rect(f, x, y, 1, 1, color);
        if i % 47 == 0 {
            hud::rect(f, x.saturating_sub(2), y, 5, 1, 0x536879);
            hud::rect(f, x, y.saturating_sub(2), 1, 5, 0x536879);
        }
    }
}

fn glow(f: &mut Framebuffer, center: Vec3, radius: f32, scale: f32) {
    let cx = f.width as f32 * 0.5 + center.x * scale;
    let cy = f.height as f32 * 0.5 - center.y * scale;
    let r = radius * scale;
    for y in 0..f.height {
        for x in 0..f.width {
            let dx = (x as f32 + 0.5 - cx) / r;
            let dy = (y as f32 + 0.5 - cy) / r;
            let d = (dx * dx + dy * dy).sqrt();
            if !(1.0..=2.8).contains(&d) {
                continue;
            }
            let angle = dy.atan2(dx);
            let rays = 0.75 + 0.25 * (angle * 19.0 + (angle * 7.0).sin() * 2.0).sin();
            let halo = (-(d - 1.0) * 11.0).exp() * 0.55 + (-(d - 1.0) * 3.5).exp() * 0.13 * rays;
            let old = f.pixels[y * f.width + x];
            let base = Vec3::new(
                ((old >> 16) & 255) as f32,
                ((old >> 8) & 255) as f32,
                (old & 255) as f32,
            ) / 255.0;
            f.pixels[y * f.width + x] = to_rgb(base + Vec3::new(1.0, 0.22, 0.025) * halo);
        }
    }
}

fn overlay(f: &mut Framebuffer, s: Settings) {
    let unit = (f.width / 650).max(1);
    let margin = f.width / 24;
    let top = f.height / 18;
    hud::rect(f, margin, top, 4 * unit, 19 * unit, 0xf43738);
    hud::text(
        f,
        "COSMOS / GP",
        margin + 12 * unit,
        top,
        3 * unit,
        0xe4e9df,
    );
    hud::text(
        f,
        "UN SISTEMA SOLAR INSPIRADO EN FÓRMULA 1",
        margin,
        top + 31 * unit,
        unit,
        0x718c9a,
    );
    let lights_x = f.width.saturating_sub(margin + 96 * unit);
    for i in 0..5 {
        hud::rect(
            f,
            lights_x + i * 20 * unit,
            top,
            12 * unit,
            12 * unit,
            0xe52d38,
        );
    }
    hud::rect(
        f,
        margin,
        f.height - margin - 25 * unit,
        f.width - 2 * margin,
        1,
        0x243e4c,
    );
    let (title, subtitle, accent) = match s.scene {
        Scene::System => (
            "LA PARRILLA",
            "01 CINCO LUCES / 02 MÓNACO / 03 REBUFO",
            0xea5243,
        ),
        Scene::Star => (
            "01 / CINCO LUCES",
            "ESTRELLA / CINCO FOCOS. UN SOLO SOL.",
            0xf97335,
        ),
        Scene::Rocky => (
            "02 / MÓNACO",
            "MUNDO ROCOSO / MÁRMOL ROJO + LUNA AUTO DE SEGURIDAD",
            0xef4751,
        ),
        Scene::Gas => (
            "03 / REBUFO",
            "GIGANTE GASEOSO / VERDE CARRERAS + HURACÁN COBRIZO",
            0x61d3bc,
        ),
    };
    let bottom = f.height - margin - 68 * unit;
    hud::text(f, title, margin, bottom, 2 * unit, 0xe4e9df);
    hud::text(f, subtitle, margin, bottom + 23 * unit, unit, accent);
    hud::text(
        f,
        &format!("CAPAS DE COLOR: {}/4", s.layers),
        margin,
        f.height - margin - 13 * unit,
        unit,
        0x718c9a,
    );
    let help = "0-3 VISTA / C CAPAS / ESPACIO PAUSA / S PNG / H TÍTULOS";
    let text_width = help.len() * 6 * unit;
    if f.width > text_width + margin * 2 + 120 * unit {
        hud::text(
            f,
            help,
            f.width - margin - text_width,
            f.height - margin - 13 * unit,
            unit,
            0x718c9a,
        );
    }
}
