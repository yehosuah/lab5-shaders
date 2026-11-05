use glam::Vec3;
use lab5_shaders::{
    framebuffer::{Fragment, Framebuffer, ProjectedVertex},
    mesh,
    render::{Renderer, Scene, Settings},
    shader::{self, Body, Uniforms},
    BACKGROUND,
};

fn triangle(z: f32) -> [ProjectedVertex; 3] {
    [
        Vec3::new(1.0, 1.0, z),
        Vec3::new(7.0, 1.0, z),
        Vec3::new(1.0, 7.0, z),
    ]
    .map(|p| ProjectedVertex {
        screen: p,
        object: p,
        world: p * 2.0,
        normal: Vec3::Z,
    })
}

#[test]
fn rasterizer_interpolates_surface_attributes_in_both_windings() {
    for reverse in [false, true] {
        let mut t = triangle(2.0);
        if reverse {
            t.reverse();
        }
        let mut f = Framebuffer::new(9, 9);
        let mut saw_pixel = false;
        f.triangle(t, |fragment| {
            if fragment.object.distance(Vec3::new(2.5, 2.5, 2.0)) < 1e-5 {
                saw_pixel = true;
                assert!(fragment.world.distance(fragment.object * 2.0) < 1e-5);
                assert_eq!(fragment.normal, Vec3::Z);
            }
            Some(Vec3::ONE)
        });
        assert!(saw_pixel);
        assert_eq!(f.pixels[2 * 9 + 2], 0xffffff);
        assert_eq!(f.pixels[6 * 9 + 6], BACKGROUND);
    }
}

#[test]
fn depth_is_order_independent_and_discard_does_not_occlude() {
    for reverse in [false, true] {
        let mut f = Framebuffer::new(9, 9);
        let mut layers = [(8.0, Vec3::X), (2.0, Vec3::Y)];
        if reverse {
            layers.reverse();
        }
        for (z, c) in layers {
            f.triangle(triangle(z), |_| Some(c));
        }
        f.triangle(triangle(1.0), |_| None);
        assert_eq!(f.pixels[2 * 9 + 2], 0x00ff00);
        assert!((f.depth[2 * 9 + 2] - 2.0).abs() < 1e-5);
    }
}

#[test]
fn invalid_and_offscreen_triangles_are_safe() {
    let mut f = Framebuffer::new(9, 9);
    f.triangle([triangle(0.0)[0]; 3], |_| panic!("degenerate"));
    let mut t = triangle(0.0);
    t[0].screen.x = f32::NAN;
    f.triangle(t, |_| panic!("nonfinite"));
    let t = triangle(0.0).map(|v| ProjectedVertex {
        screen: v.screen - Vec3::new(20.0, 20.0, 0.0),
        ..v
    });
    f.triangle(t, |_| panic!("offscreen"));
    assert!(f.pixels.iter().all(|&c| c == BACKGROUND));
}

#[test]
fn sphere_is_closed_non_degenerate_outward_and_indexed() {
    let m = mesh::sphere(32, 16);
    let mut edges = std::collections::HashMap::new();
    for &[a, b, c] in &m.faces {
        let [p, q, r] = [a, b, c].map(|i| m.vertices[i].position);
        assert!((q - p).cross(r - p).dot(p + q + r) > 1e-7);
        for (i, j) in [(a, b), (b, c), (c, a)] {
            *edges.entry((i.min(j), i.max(j))).or_insert(0usize) += 1;
        }
    }
    assert!(edges.values().all(|&count| count == 2));
    assert!(m
        .vertices
        .iter()
        .all(|v| (v.position.length() - 1.0).abs() < 1e-5));
}

#[test]
fn required_bodies_have_four_visible_layers_and_reproducible_animation() {
    let renderer = Renderer::new();
    for scene in [Scene::Star, Scene::Rocky, Scene::Gas] {
        let mut a = Framebuffer::new(256, 160);
        let mut b = Framebuffer::new(256, 160);
        let settings = Settings {
            scene,
            hud: false,
            ..Settings::default()
        };
        renderer.render(&mut a, settings);
        renderer.render(&mut b, settings);
        assert_eq!(a.pixels, b.pixels, "deterministic {scene:?}");
        for layers in 1..=4 {
            renderer.render(&mut b, Settings { layers, ..settings });
            if layers > 1 {
                let changed = a
                    .pixels
                    .iter()
                    .zip(&b.pixels)
                    .filter(|(x, y)| x != y)
                    .count();
                assert!(
                    changed > 30,
                    "{scene:?} layer {layers}: only {changed} pixels changed"
                );
            }
            a.pixels.clone_from(&b.pixels);
        }
        renderer.render(
            &mut b,
            Settings {
                time: 8.0,
                ..settings
            },
        );
        assert_ne!(a.pixels, b.pixels, "animation {scene:?}");
    }
}

#[test]
fn ring_is_a_separate_annulus_with_gaps_and_a_planet_shadow() {
    let renderer = Renderer::new();
    assert_ne!(renderer.moon.vertices.len(), renderer.sphere.vertices.len());
    assert!(renderer.rings.vertices.iter().all(|v| v.position.y == 0.0));
    assert!(renderer
        .rings
        .vertices
        .iter()
        .all(|v| v.position.length() > 1.0));
    let u = Uniforms {
        body: Body::Ring,
        time: 0.0,
        layers: 4,
        center: Vec3::ZERO,
        radius: 1.0,
        ring_normal: Vec3::Y,
    };
    let fragment = |object, world| Fragment {
        object,
        world,
        normal: Vec3::Y,
    };
    assert!(shader::shade(fragment(Vec3::X * 1.55, Vec3::X * 1.55), &u).is_none());
    let direction = Vec3::new(-shader::light().x, 0.0, -shader::light().z).normalize();
    let object = direction * 1.45;
    let shadow = shader::shade(fragment(object, object), &u).unwrap();
    let bright = shader::shade(fragment(-object, -object), &u).unwrap();
    assert!(shadow.length() < bright.length() * 0.25);
}
