use std::process::{self, Command};
use std::io::{self, Write};
use crate::fileutil;

//gs -sDEVICE=png256 -o bar_%03d.png -r200x200 input.pdf
//Solo convierte a png por el momento
//Averiguar otro comando que saque mejores imagenes, este saca a muy baja resolucion
pub fn pdf2imgs(pdf_path: &str, out_dir_path: &str, ext: &str) -> anyhow::Result<Vec<String>> {
    let result_paths: Vec<String>;
    let out_name: String = format!("{}/%03d.png", out_dir_path);

    //Lo tenemos que dejar -sDEVICE=png16m porque con png256 la diferencia de iagenes causa problemas
    let out_message: process::Output = Command::new("gs")
        .args(["-sDEVICE=png16m", "-o", &out_name, "-r600", "-dDownScaleFactor=3", "-dUseCropBox", "-q", pdf_path])
        .output()?;
    io::stdout().write_all(&out_message.stdout)?;
    io::stderr().write_all(&out_message.stderr)?;

    result_paths = fileutil::list_files_from_path(out_dir_path, ext)?;

    Ok(result_paths)
}

//pdftk *.pdf cat output result.pdf
//El ordenamiento de las carpetas es aleatorio, asi que depende del comando pdftk tomar los archivos en orden
pub fn merge_pdf(pdf_images: &Vec<String>, out_dir: &str) -> anyhow::Result<String> {
    let out_file: String = format!("{}/merged.pdf", out_dir);

    //Aqui convertimos el &Vec<String> a Vec<&String> para pasarle a Command::new
    let mut swap_vec: Vec<&str> = Vec::new();
    swap_vec.reserve_exact(pdf_images.len());
    for img in pdf_images {
        swap_vec.push(img);
    }
    //Usamos vector para el resto de argumentos porque no podemos concatenar tipos diferentes
    let args: Vec<&str> = vec!["cat", "output", &out_file];

    //Concatenamos los dos vectores de argumentos en un intento de emular la spread syntax
    let out_message: process::Output = Command::new("pdftk")
        .args([&swap_vec[..], &args[..]].concat()) 
        .output()?;
    io::stdout().write_all(&out_message.stdout)?;
    io::stderr().write_all(&out_message.stderr)?;

    Ok(out_file)
}

//qpdf Evading_EDR.pdf --overlay result.pdf -- stamped.pdf
pub fn overlay_pdf(bottom_pdf_path: &str, top_pdf_path: &str, out_dir: &str) -> anyhow::Result<()> {
    let out_file: String = format!("{}/result.pdf", out_dir);

    let out_message: process::Output = Command::new("qpdf")
        .args([&bottom_pdf_path, "--overlay", &top_pdf_path, "--", &out_file])
        .output()?;
    io::stdout().write_all(&out_message.stdout)?;
    io::stderr().write_all(&out_message.stderr)?;

    Ok(())
}