fn main() {
    println!("cargo::rerun-if-changed=assets/icons.toml");
    println!("cargo::rerun-if-changed=assets/bsp.svg");
    println!("cargo::rerun-if-changed=assets/vstack.svg");
    println!("cargo::rerun-if-changed=assets/rmvstack.svg");
    println!("cargo::rerun-if-changed=assets/grid.svg");
    println!("cargo::rerun-if-changed=assets/hstack.svg");
    println!("cargo::rerun-if-changed=assets/uwvstack.svg");
    println!("cargo::rerun-if-changed=assets/columns.svg");
    println!("cargo::rerun-if-changed=assets/rows.svg");
    iced_fontello::build("assets/icons.toml").expect("Generate icons font");

    println!("cargo::rerun-if-changed=assets/komorice.ico");
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/komorice.ico");
        res.compile().unwrap();
    }
}
