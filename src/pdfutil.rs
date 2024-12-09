use std::process::{self, Command};
use std::io::{self, Write};
use crate::fileutil;

//gs -sDEVICE=png256 -o bar_%03d.png -r200x200 input.pdf
//Solo convierte a png por el momento
pub fn pdf2imgs(pdf_path: &str, out_dir_path: &str, ext: &str) -> anyhow::Result<Vec<String>> {
    let result_paths: Vec<String>;
    let out_name: String = format!("{}/%03d.png", out_dir_path);

    let out_message: process::Output = Command::new("gs")
        .args(["-sDEVICE=png16m", "-o", &out_name, "-r150", "-q", pdf_path])
        .output()?;
    io::stdout().write_all(&out_message.stdout)?;
    io::stderr().write_all(&out_message.stderr)?;

    result_paths = fileutil::list_files_from_path(out_dir_path, ext)?;

    Ok(result_paths)
}

//pdftk *.pdf cat output result.pdf
//El ordenamiento de las carpetas es aleatorio, asi que depende del comando pdftk tomar los archivos en orden
pub fn merge_pdf(pdf_images_dir: &str, out_dir: &str) -> anyhow::Result<String> {
    let all_pdf: String = format!("{}/*.pdf", pdf_images_dir);
    let out_file: String = format!("{}/merged.pdf", out_dir);

    let out_message: process::Output = Command::new("pdftk")
        .args([&all_pdf, "cat", "output", &out_file])
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