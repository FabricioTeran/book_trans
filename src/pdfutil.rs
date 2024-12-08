use std::process::{self, Command};
use std::io::{self, Write};
use crate::fileutil;

//gs -sDEVICE=png256 -o bar_%03d.png -r200x200 input.pdf
//Solo convierte a png por el momento
pub fn pdf2imgs(pdf_path: &str, out_dir_path: &str, ext: &str) -> anyhow::Result<Vec<String>> {
    let result_paths: Vec<String>;
    let out_name: String = format!("{}/%03d.png", out_dir_path);

    let out_message: process::Output = Command::new("gs")
        .args(["-sDEVICE=png256", "-o", &out_name, "-r200x200", pdf_path])
        .output()?;
    io::stdout().write_all(&out_message.stdout)?;
    io::stderr().write_all(&out_message.stderr)?;

    result_paths = fileutil::list_files_from_path(out_dir_path, ext)?;

    Ok(result_paths)
}

//img2pdf path/*.png -o out.pdf
//Confiamos en el orden de las imagenes porque terminan en -001, -002, entonces un ordenamiento alfanumerico sirve
pub fn imgs2pdf(img_dir: &str, out_dir: &str) -> anyhow::Result<()> {
    let all_png: String = format!("{}/*.png", img_dir);
    let out_file: String = format!("{}/out.pdf", out_dir);

    let out_message: process::Output = Command::new("img2pdf")
        .args([&all_png, "-o", &out_file])
        .output()?;
    io::stdout().write_all(&out_message.stdout)?;
    io::stderr().write_all(&out_message.stderr)?;

    Ok(())
}