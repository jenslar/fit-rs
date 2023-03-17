# fit-rs

Parse most Garmin FIT files. Supports custom developer data, but not compressed timestamp headers.

Example:
```rust
use fit_rs::Fit;
fn main() -> std::io::Result<()> {
    let path = PathBuf::from("MYFITFILE.fit");
    let fit = Fit::new(&path)?;
    println!("{fit:#?}");
    Ok(())
}
```