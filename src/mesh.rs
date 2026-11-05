use glam::Vec3;
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<[usize; 3]>,
}

/// Esfera triangulada por latitud y longitud; sin UV, texturas ni materiales.
pub fn sphere(slices: usize, stacks: usize) -> Mesh {
    assert!(slices >= 3 && stacks >= 2);
    let mut vertices = vec![Vertex {
        position: Vec3::Y,
        normal: Vec3::Y,
    }];
    for j in 1..stacks {
        let phi = PI * j as f32 / stacks as f32;
        for i in 0..slices {
            let theta = TAU * i as f32 / slices as f32;
            let p = Vec3::new(phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin());
            vertices.push(Vertex {
                position: p,
                normal: p,
            });
        }
    }
    let bottom = vertices.len();
    vertices.push(Vertex {
        position: -Vec3::Y,
        normal: -Vec3::Y,
    });
    let mut faces = Vec::new();
    for i in 0..slices {
        let k = (i + 1) % slices;
        faces.push([0, 1 + k, 1 + i]);
        for j in 0..stacks - 2 {
            let a = 1 + j * slices + i;
            let b = 1 + j * slices + k;
            faces.push([a, b, a + slices]);
            faces.push([b, b + slices, a + slices]);
        }
        faces.push([bottom, bottom - slices + i, bottom - slices + k]);
    }
    Mesh { vertices, faces }
}

/// Malla anular independiente en el plano XZ, separada de la esfera.
pub fn rings(segments: usize, inner: f32, outer: f32) -> Mesh {
    assert!(segments >= 3 && inner > 1.0 && outer > inner);
    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    for i in 0..segments {
        let theta = TAU * i as f32 / segments as f32;
        for radius in [inner, outer] {
            vertices.push(Vertex {
                position: Vec3::new(theta.cos() * radius, 0.0, theta.sin() * radius),
                normal: Vec3::Y,
            });
        }
        let a = i * 2;
        let b = ((i + 1) % segments) * 2;
        faces.push([a, a + 1, b]);
        faces.push([a + 1, b + 1, b]);
    }
    Mesh { vertices, faces }
}
