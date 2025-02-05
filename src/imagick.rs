use std::process::{self, Command};
use std::io::{self, Write};

/*Creates PDFs containing the difference between the input images, the result PDFs have transparent background.
Parameters:
- orig_images: &[String] = A slice containing the converted images from the original PDF.
- modif_images: &[String] = A slice containing the converted images from the modified PDF.
- out_dir: &str = The path to the output dir.
All paths should be Unix valid.
Return:
- Result<Vec<String>> = The Result containing a vector of strings of the paths of the resulting PDFs.
*/
pub fn mask_and_alpha(orig_images: &[String], modif_images: &[String], out_dir: &str) -> anyhow::Result<Vec<String>> {
    //Comparamos el vector con menor longitud para que haya correspondencia 1-1 entre ambas carpetas
    //Si es mayor o igual devolvemos modif porque es menor o igual, si es menor, devolvemos orig porque es menor
    let min_len = match orig_images.len() >= modif_images.len() {
        true => modif_images.len(),
        false => orig_images.len()
    };
    let mut final_vec: Vec<String> = Vec::new();
    final_vec.reserve_exact(min_len);

    for i in 0..min_len {
        let out_file: String = format!("{}/{}.pdf", out_dir, i);

        let mask_args: String = format!("compare {} {} -compose src -highlight-color white -lowlight-color black -quiet miff:-", modif_images[i], orig_images[i]);
        let recognize_args: String = format!("composite -compose Dst_In \\( - -alpha copy \\) {} -alpha Set {}", orig_images[i], out_file);
        let complete_args: String = format!("{} | {}", mask_args, recognize_args);

        let out_message: process::Output = Command::new("bash")
            .args(["-c", &complete_args])
            .output()?;
        io::stdout().write_all(&out_message.stdout)?;
        io::stderr().write_all(&out_message.stderr)?;

        final_vec.push(out_file);
    }

    Ok(final_vec)
}

