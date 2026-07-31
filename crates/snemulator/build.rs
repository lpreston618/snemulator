use std::io;

fn main() -> io::Result<()> {
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        winresource::WindowsResource::new()
            .set_icon("assets/logo/snemulator-logo.ico")
            .compile()?;
    }
    Ok(())
}