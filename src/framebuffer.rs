use glam::Vec3;

#[derive(Clone, Copy, Default)]
pub struct ProjectedVertex {
    pub screen: Vec3,
    pub object: Vec3,
    pub world: Vec3,
    pub normal: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub struct Fragment {
    pub object: Vec3,
    pub world: Vec3,
    pub normal: Vec3,
}

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    pub depth: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0 && height > 0);
        Self {
            width,
            height,
            pixels: vec![crate::BACKGROUND; width * height],
            depth: vec![f32::INFINITY; width * height],
        }
    }

    /// La proyección ortográfica permite interpolación baricéntrica afín exacta.
    /// El shader puede descartar fragmentos, por ejemplo en huecos entre anillos.
    pub fn triangle(
        &mut self,
        vertices: [ProjectedVertex; 3],
        mut shade: impl FnMut(Fragment) -> Option<Vec3>,
    ) {
        let [a, b, c] = vertices.map(|v| v.screen);
        if vertices.iter().any(|v| !v.screen.is_finite()) {
            return;
        }
        let area = edge(a, b, c);
        if area.abs() < 1e-6 {
            return;
        }
        let min = a.min(b).min(c).floor().max(Vec3::ZERO);
        let max = a.max(b).max(c).ceil().min(Vec3::new(
            self.width as f32 - 1.0,
            self.height as f32 - 1.0,
            f32::INFINITY,
        ));
        if min.x > max.x || min.y > max.y {
            return;
        }
        for y in min.y as usize..=max.y as usize {
            for x in min.x as usize..=max.x as usize {
                let p = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);
                let weights = Vec3::new(edge(b, c, p), edge(c, a, p), edge(a, b, p)) / area;
                if weights.min_element() < -1e-6 {
                    continue;
                }
                let z = weights.dot(Vec3::new(a.z, b.z, c.z));
                let index = y * self.width + x;
                if z >= self.depth[index] {
                    continue;
                }
                let interpolate = |f: fn(ProjectedVertex) -> Vec3| {
                    f(vertices[0]) * weights.x
                        + f(vertices[1]) * weights.y
                        + f(vertices[2]) * weights.z
                };
                let fragment = Fragment {
                    object: interpolate(|v| v.object),
                    world: interpolate(|v| v.world),
                    normal: interpolate(|v| v.normal).normalize_or_zero(),
                };
                if let Some(color) = shade(fragment) {
                    self.depth[index] = z;
                    self.pixels[index] = to_rgb(color);
                }
            }
        }
    }

    /// La ventana y el PNG usan exactamente el mismo framebuffer RGB.
    pub fn write_png(&self, path: impl AsRef<std::path::Path>) -> image::ImageResult<()> {
        let image = image::RgbImage::from_fn(self.width as u32, self.height as u32, |x, y| {
            let rgb = self.pixels[y as usize * self.width + x as usize];
            image::Rgb([(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8])
        });
        image.save_with_format(path, image::ImageFormat::Png)
    }
}

pub fn to_rgb(color: Vec3) -> u32 {
    // Las paletas se definen en RGB de pantalla; el color es artístico.
    let [r, g, b] = (color.clamp(Vec3::ZERO, Vec3::ONE) * 255.0)
        .round()
        .as_uvec3()
        .to_array();
    (r << 16) | (g << 8) | b
}

fn edge(a: Vec3, b: Vec3, p: Vec3) -> f32 {
    (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
}
