use std::env;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=RSS-strings.ico");

    let icon_path = generate_scaled_icon()?;

    let mut res = winres::WindowsResource::new();
    res.set_icon(&icon_path);
    res.set_manifest(MANIFEST);
    res.set("ProductName", "RSS-strings");
    res.set("FileDescription", "RSS-strings");
    res.set("OriginalFilename", "RSS-strings.exe");
    res.set("CompanyName", "RSS-strings");
    res.compile()?;
    Ok(())
}

fn generate_scaled_icon() -> Result<String, Box<dyn std::error::Error>> {
    let bytes = std::fs::read("RSS-strings.ico")?;

    let base_rgba = {
        let mut cursor = Cursor::new(bytes.as_slice());
        let decoded = ico::IconDir::read(&mut cursor)
            .ok()
            .and_then(|dir| {
                dir.entries()
                    .iter()
                    .cloned()
                    .max_by_key(|e| e.width() * e.height())
                    .and_then(|e| e.decode().ok())
            })
            .map(|img| {
                image::RgbaImage::from_raw(img.width(), img.height(), img.rgba_data().to_vec())
                    .expect("valid rgba data")
            });

        if let Some(img) = decoded {
            img
        } else {
            image::load_from_memory(&bytes)?.into_rgba8()
        }
    };

    let target_sizes = [256u32, 128, 64, 32];
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    for &size in &target_sizes {
        let resized = if base_rgba.width() == size && base_rgba.height() == size {
            base_rgba.clone()
        } else {
            image::imageops::resize(
                &base_rgba,
                size,
                size,
                image::imageops::FilterType::Lanczos3,
            )
        };
        let icon_image = ico::IconImage::from_rgba_data(size, size, resized.into_raw());
        icon_dir.add_entry(ico::IconDirEntry::encode(&icon_image)?);
    }

    let mut out_path = PathBuf::from(env::var("OUT_DIR")?);
    out_path.push("generated-icon.ico");
    let mut file = File::create(&out_path)?;
    icon_dir.write(&mut file)?;
    Ok(out_path.display().to_string())
}

// requireAdministrator manifest
const MANIFEST: &str = r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
    </dependentAssembly>
  </dependency>
</assembly>
"#;
