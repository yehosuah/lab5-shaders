use lab5_shaders::{
    framebuffer::Framebuffer,
    hud,
    render::{Renderer, Scene, Settings},
};
use std::{
    error::Error,
    path::{Path, PathBuf},
};

struct Options {
    width: usize,
    height: usize,
    settings: Settings,
    output: PathBuf,
    render_only: bool,
    paused: bool,
    frames: Option<usize>,
    layer_sheet: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let Some(mut options) = parse()? else {
        return Ok(());
    };
    if options.layer_sheet {
        options.render_only = true;
    }
    if !options.render_only && !cfg!(feature = "window") {
        return Err("Usa --solo-render en una compilación sin la característica window".into());
    }
    let renderer = Renderer::new();
    let mut framebuffer = Framebuffer::new(options.width, options.height);
    if options.layer_sheet {
        layer_sheet(&renderer, &mut framebuffer, options.settings);
    } else {
        renderer.render(&mut framebuffer, options.settings);
    }
    save(&framebuffer, &options.output)?;
    println!(
        "COSMOS / GP | {} | {}x{} | t={:.2} | capas={} | {}",
        options.settings.scene.name(),
        options.width,
        options.height,
        options.settings.time,
        options.settings.layers,
        options.output.display()
    );
    if !options.render_only {
        show_window(&renderer, &mut framebuffer, options)?;
    }
    Ok(())
}

fn parse() -> Result<Option<Options>, Box<dyn Error>> {
    let mut o = Options {
        width: 1280,
        height: 800,
        settings: Settings::default(),
        output: PathBuf::from("renders/captura.png"),
        render_only: false,
        paused: false,
        frames: None,
        layer_sheet: false,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--solo-render" | "--render-only" => o.render_only = true,
            "--pausado" | "--paused" => o.paused = true,
            "--sin-titulos" | "--no-hud" => o.settings.hud = false,
            "--comparar-capas" | "--layer-sheet" => o.layer_sheet = true,
            "--salida" | "--output" => {
                o.output = args.next().ok_or("Falta la ruta de --salida")?.into()
            }
            "--escena" | "--scene" => {
                o.settings.scene = Scene::parse(&args.next().ok_or("Falta el nombre de --escena")?)
                    .ok_or("Escenas: sistema, estrella, rocoso, gaseoso")?
            }
            "--ancho" | "--width" => o.width = args.next().ok_or("Falta --ancho")?.parse()?,
            "--alto" | "--height" => o.height = args.next().ok_or("Falta --alto")?.parse()?,
            "--tiempo" | "--time" => {
                o.settings.time = args.next().ok_or("Falta --tiempo")?.parse()?
            }
            "--capas" | "--layers" => {
                o.settings.layers = args.next().ok_or("Falta --capas")?.parse()?
            }
            "--fotogramas" | "--frames" => {
                o.frames = Some(args.next().ok_or("Falta --fotogramas")?.parse()?)
            }
            "--ayuda" | "--help" | "-h" => {
                println!("COSMOS / GP - Laboratorio de shaders\n\ncargo run --release -- [opciones]\n\n--escena sistema|estrella|rocoso|gaseoso\n--solo-render       Exportar sin abrir ventana\n--salida ruta.png   PNG del framebuffer (predeterminado: renders/captura.png)\n--ancho 1280 --alto 800\n--tiempo 0          Tiempo inicial reproducible, en segundos\n--capas 1..4        Capas acumulativas\n--comparar-capas    Exportar comparación 2x2 de las cuatro capas\n--sin-titulos       Ocultar títulos y controles\n--pausado           Iniciar pausado\n--fotogramas N      Cerrar ventana después de N fotogramas\n\n0 sistema | 1 estrella | 2 rocoso | 3 gaseoso | Tab siguiente\nC cambia capas | Espacio pausa | R reinicia tiempo | H oculta títulos\nS guarda el framebuffer visible | Escape cierra");
                return Ok(None);
            }
            _ => return Err(format!("Opción desconocida: {arg}. Usa --ayuda.").into()),
        }
    }
    if !(160..=4096).contains(&o.width) || !(160..=4096).contains(&o.height) {
        return Err("Dimensiones permitidas: de 160 a 4096 píxeles".into());
    }
    if !(1..=4).contains(&o.settings.layers) {
        return Err("--capas debe estar entre 1 y 4".into());
    }
    if !o.settings.time.is_finite() {
        return Err("--tiempo debe ser finito".into());
    }
    if o.frames == Some(0) {
        return Err("--fotogramas debe ser mayor que cero".into());
    }
    if o.layer_sheet && o.settings.scene == Scene::System {
        return Err("--comparar-capas necesita --escena estrella, rocoso o gaseoso".into());
    }
    Ok(Some(o))
}

fn save(frame: &Framebuffer, path: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    frame.write_png(path)?;
    Ok(())
}

fn layer_sheet(renderer: &Renderer, frame: &mut Framebuffer, settings: Settings) {
    let w = frame.width / 2;
    let h = frame.height / 2;
    let mut panel = Framebuffer::new(w, h);
    for i in 0..4usize {
        renderer.render(
            &mut panel,
            Settings {
                layers: i as u8 + 1,
                hud: false,
                ..settings
            },
        );
        let scale = (w / 350).max(1);
        hud::text(
            &mut panel,
            &format!(
                "{} / CAPAS 1-{}",
                settings.scene.name().to_uppercase(),
                i + 1
            ),
            18,
            16,
            scale,
            0xe1e8e1,
        );
        let ox = (i % 2) * w;
        let oy = (i / 2) * h;
        for y in 0..h {
            frame.pixels[(oy + y) * frame.width + ox..(oy + y) * frame.width + ox + w]
                .copy_from_slice(&panel.pixels[y * w..(y + 1) * w]);
        }
    }
    hud::rect(frame, w, 0, 1, frame.height, 0x324452);
    hud::rect(frame, 0, h, frame.width, 1, 0x324452);
}

#[cfg(feature = "window")]
fn show_window(
    renderer: &Renderer,
    frame: &mut Framebuffer,
    mut options: Options,
) -> Result<(), Box<dyn Error>> {
    use minifb::{Key, KeyRepeat, Window, WindowOptions};
    let mut window = Window::new(
        "COSMOS / GP | 0-3 vistas | C capas | Espacio pausa | S PNG | ESC",
        frame.width,
        frame.height,
        WindowOptions::default(),
    )?;
    window.set_target_fps(30);
    let mut last = std::time::Instant::now();
    let mut count = 0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(last).as_secs_f32().min(0.15);
        last = now;
        for (key, scene) in [
            (Key::Key0, Scene::System),
            (Key::Key1, Scene::Star),
            (Key::Key2, Scene::Rocky),
            (Key::Key3, Scene::Gas),
        ] {
            if window.is_key_pressed(key, KeyRepeat::No) {
                options.settings.scene = scene;
            }
        }
        if window.is_key_pressed(Key::Tab, KeyRepeat::No) {
            options.settings.scene = match options.settings.scene {
                Scene::System => Scene::Star,
                Scene::Star => Scene::Rocky,
                Scene::Rocky => Scene::Gas,
                Scene::Gas => Scene::System,
            };
        }
        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            options.paused = !options.paused;
        }
        if window.is_key_pressed(Key::C, KeyRepeat::No) {
            options.settings.layers = options.settings.layers % 4 + 1;
        }
        if window.is_key_pressed(Key::H, KeyRepeat::No) {
            options.settings.hud = !options.settings.hud;
        }
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            options.settings.time = 0.0;
        } else if !options.paused {
            options.settings.time += dt;
        }
        renderer.render(frame, options.settings);
        window.update_with_buffer(&frame.pixels, frame.width, frame.height)?;
        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            save(frame, &options.output)?;
            println!(
                "Guardado: {} | t={:.3}",
                options.output.display(),
                options.settings.time
            );
        }
        count += 1;
        if options.frames.is_some_and(|limit| count >= limit) {
            break;
        }
    }
    Ok(())
}

#[cfg(not(feature = "window"))]
fn show_window(_: &Renderer, _: &mut Framebuffer, _: Options) -> Result<(), Box<dyn Error>> {
    Err("Ventana deshabilitada".into())
}
