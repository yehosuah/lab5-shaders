use lab5_shaders::{
    framebuffer::Framebuffer,
    render::{Renderer, Scene, Settings},
};
use std::{fs, process::Command};

#[test]
fn png_is_exactly_the_software_framebuffer_even_from_another_directory() {
    let path = std::env::temp_dir().join(format!("orbital-gp-{}.png", std::process::id()));
    let result = Command::new(env!("CARGO_BIN_EXE_lab5-shaders"))
        .current_dir(std::env::temp_dir())
        .args([
            "--solo-render",
            "--escena",
            "gaseoso",
            "--ancho",
            "256",
            "--alto",
            "160",
            "--tiempo",
            "2.5",
            "--sin-titulos",
            "--salida",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let image = image::open(&path).unwrap().to_rgb8();
    fs::remove_file(path).unwrap();
    let mut expected = Framebuffer::new(256, 160);
    Renderer::new().render(
        &mut expected,
        Settings {
            scene: Scene::Gas,
            time: 2.5,
            hud: false,
            ..Settings::default()
        },
    );
    assert_eq!(image.dimensions(), (256, 160));
    for (pixel, &rgb) in image.pixels().zip(&expected.pixels) {
        assert_eq!(pixel.0, [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8]);
    }
}

#[test]
fn invalid_cli_inputs_fail_with_a_message() {
    for args in [
        vec!["--unknown"],
        vec!["--tiempo", "NaN"],
        vec!["--ancho", "0"],
        vec!["--capas", "5"],
        vec!["--escena", "pluto"],
        vec!["--salida"],
        vec!["--fotogramas", "0"],
        vec!["--comparar-capas"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_lab5-shaders"))
            .args(args)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(!result.stderr.is_empty());
    }
}
