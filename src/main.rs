use std::env;

use clap::Parser;
use std::path::PathBuf;
use std::fs;

use book_trans::imagick;
use book_trans::pdfutil;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    ///The path to the original pdf.
    #[arg(long)]
    orig: String,

    //The path to the modified pdf.
    #[arg(long)]
    modif: String,

    ///The path to the folder of the translated pdf. DEFAULT:./trans
    #[arg(long)]
    trans: String,

    ///The path of the output folder, the program creates a folder named booktrans_out in the specified path. DEFAULT:./
    #[arg(long, default_value = "./")]
    o: String,

    ///The file extension of the images located in the orig and modif folders, you cant specify 2 file extensions
    #[arg(long, default_value = "png")]
    iext: String,
}

/* The main function
Parameters:
Return:
*/
fn main() -> anyhow::Result<()> {
    env::set_var("RUST_BACKTRACE", "1");

    let args: Args = Args::parse();

    let created_outdir_path: String = create_outdir(args.o)?;

    //Create orig and modif folders
    let orig_dir: String = format!("{}/orig", &created_outdir_path);
    fs::create_dir(&orig_dir)?;
    let modif_dir: String = format!("{}/modif", &created_outdir_path);
    fs::create_dir(&modif_dir)?;
    let pdf_dir: String = format!("{}/pdf", &created_outdir_path);
    fs::create_dir(&pdf_dir)?;

    //Convert orig and modif pdfs to images
    let res1: Vec<String> = pdfutil::pdf2imgs(&args.orig, &orig_dir, &args.iext)?;
    let orig_image_paths_slice: &[String] = res1.as_slice();
    let res2: Vec<String> = pdfutil::pdf2imgs(&args.modif, &modif_dir, &args.iext)?;
    let modif_image_paths_slice: &[String] = res2.as_slice();

    //Generate pdfs from the difference between the generated images
    let pdf_img_paths: Vec<String> = imagick::mask_and_alpha(orig_image_paths_slice, modif_image_paths_slice, &pdf_dir)?;
    println!("{:?}", &pdf_img_paths);

    let merged_pdf: String = pdfutil::merge_pdf(&pdf_img_paths, &pdf_dir)?;
    pdfutil::overlay_pdf(&args.trans, &merged_pdf, &created_outdir_path)?;

    Ok(())
}

/* Creates a directory from a String, it's not validated.
Parameters:
- ostring: String = The String representing the path of the directory that will be created, it should be unix valid path.
Return:
- Result<String> = A result containing a string of the path of the created directory.
*/
fn create_outdir(ostring: String) -> anyhow::Result<String> {
    //The pathbuff validates the path to be valid on the current OS platform, and resolve relative paths like "./", "/" or "../"
    let mut out_dir_path: PathBuf = PathBuf::from(&ostring);
    //Generate a random string to not overwrite previous outputs
    let rand_str: String = random_string::generate(15, "abcdefghijklmnopqrstuvwxyz1234567890");
    let complete_dir_name: String = format!("BT_out_{}", &rand_str);
    out_dir_path.push(&complete_dir_name);
    
    //We need to manage the case of unexisting folder
    fs::create_dir(&out_dir_path)?;
    
    Ok(out_dir_path.display().to_string())
}

