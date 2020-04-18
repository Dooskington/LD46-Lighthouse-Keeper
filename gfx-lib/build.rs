//extern crate glsl_to_spirv;

//use glsl_to_spirv::ShaderType;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // TODO Disabled for now, as rerun-if-changed doesn't work for directories.
    // It also has no way to notice new files.
    /*
    // Create destination path if necessary
    if !std::path::Path::new("res/shaders/bin/").exists() {
        std::fs::create_dir_all("res/shaders/bin")?;
        println!("Created res/shaders/bin/");
    }

    for entry in std::fs::read_dir("res/shaders/src")? {
        let entry = entry?;

        if !entry.file_type()?.is_file() {
            continue;
        }

        let path = entry.path();

        println!("TESTING: {:?}", path);

        // Tell the build script to only run again if we modified these shaders
        //println!("cargo:rerun-if-changed={:#?}", path);

        // Only support vertex and fragment shaders
        let shader_type = path
            .extension()
            .and_then(|ext| match ext.to_string_lossy().as_ref() {
                "glslv" => Some(ShaderType::Vertex),
                "glslf" => Some(ShaderType::Fragment),
                _ => None,
            });

        if let Some(shader_type) = shader_type {
            use std::io::Read;

            let source = std::fs::read_to_string(&path)?;
            let mut compiled_file = glsl_to_spirv::compile(&source, shader_type)?;

            let mut compiled_bytes = Vec::new();
            compiled_file.read_to_end(&mut compiled_bytes)?;

            let output_path = format!(
                "res/shaders/bin/{}.spv",
                path.file_name().unwrap().to_string_lossy()
            );

            std::fs::write(&output_path, &compiled_bytes)?;
        }

        // TODO (Declan, 9/25/2018)
        // Right now, if a shader fails compiling, you get a real nasty error message.
        // Need to work on some proper error handling and display.
    }

    println!("Shaders compiled.");
    */
    Ok(())
}
