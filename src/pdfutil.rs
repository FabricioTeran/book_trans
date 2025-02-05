use std::process::{self, Command};
use std::io::{self, Write};
use crate::fileutil;

/*Converts PDFs to a collection of images with the command: gs -sDEVICE=png256 -o bar_%03d.png -r200x200 input.pdf
Parameters:
- pdf_path: &str = The path to the PDF.
- out_dir_path: &str = The path to the output dir.
- ext: &str = The extension of the created images, only .png supported for now.
All paths should be Unix valid.
Return:
- Result<Vec<String>> = The Result containing a vector of strings of all the paths of the extracted images.
*/
pub fn pdf2imgs(pdf_path: &str, out_dir_path: &str, ext: &str) -> anyhow::Result<Vec<String>> { //Solo convierte a png por el momento
    let result_paths: Vec<String>;
    let out_name: String = format!("{}/%03d.png", out_dir_path);

    //We are using -sDEVICE=png16m because png256 causes problems on image difference operations
    let out_message: process::Output = Command::new("gs")
        .args(["-sDEVICE=png16m", "-o", &out_name, "-r600", "-dDownScaleFactor=3", "-dUseCropBox", "-q", pdf_path])
        .output()?;
    io::stdout().write_all(&out_message.stdout)?;
    io::stderr().write_all(&out_message.stderr)?;

    result_paths = fileutil::list_files_from_path(out_dir_path, ext)?;

    Ok(result_paths)
}

/*Merges PDFs with the command: pdftk *.pdf cat output result.pdf
Parameters:
- pdf_images: &Vec<String> = The list (as reference) of the paths of all PDFs to be merged.
- out_dir: &str = The path of the output dir to be passed to the command.
All paths should be Unix valid.
Return:
- Result<String> = The Result containing the path of the final PDF produced by the command.
*/
pub fn merge_pdf(pdf_images: &Vec<String>, out_dir: &str) -> anyhow::Result<String> { //El ordenamiento de las carpetas es aleatorio, asi que depende del comando pdftk tomar los archivos en orden
    let out_file: String = format!("{}/merged.pdf", out_dir);

    //Converting &Vec<String> to Vec<&String> in order to be able to pass it to Command:new
    let mut swap_vec: Vec<&str> = Vec::new();
    swap_vec.reserve_exact(pdf_images.len());
    for img in pdf_images {
        swap_vec.push(img);
    }
    //Using the same type for the other arguments
    let args: Vec<&str> = vec!["cat", "output", &out_file];

    let out_message: process::Output = Command::new("pdftk")
        .args([&swap_vec[..], &args[..]].concat()) //Concatenating the 2 vectors in only one
        .output()?;
    io::stdout().write_all(&out_message.stdout)?;
    io::stderr().write_all(&out_message.stderr)?;

    Ok(out_file)
}

/*Overlays a PDF with transparent background on top of another PDF with the command: qpdf original.pdf --overlay result.pdf -- stamped.pdf
Parameters:
- bottom_pdf_path: &str = The path to the bottom PDF.
- top_pdf_path: &str = The path to the top PDF, it should have transparent background.
- out_dir: &str = The path to the output dir.
All paths should be Unix valid.
Return:
- Result<()> = If no errors, the result is an empty tuple.
*/
pub fn overlay_pdf(bottom_pdf_path: &str, top_pdf_path: &str, out_dir: &str) -> anyhow::Result<()> {
    let out_file: String = format!("{}/result.pdf", out_dir);

    let out_message: process::Output = Command::new("qpdf")
        .args([&bottom_pdf_path, "--overlay", &top_pdf_path, "--", &out_file])
        .output()?;
    io::stdout().write_all(&out_message.stdout)?;
    io::stderr().write_all(&out_message.stderr)?;

    Ok(())
}