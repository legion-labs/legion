fn main() {
    let mut debug = false;
    let mut opt_level = 3;

    if std::env::var("PROFILE").unwrap().contains("debug") {
        debug = true;
        opt_level = 0;
    }

    cc::Build::new()
        .define("MINIMP4_IMPLEMENTATION", None)
        .file("csrc/minimp4.c")
        .debug(debug)
        .opt_level(opt_level)
        .warnings(false)
        .compile("minimp4");

    println!("cargo:rustc-link-lib=static=minimp4");
}
