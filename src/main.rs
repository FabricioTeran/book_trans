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

fn main() -> anyhow::Result<()> {
    env::set_var("RUST_BACKTRACE", "1");

    let args: Args = Args::parse();

    let created_outdir_path: String = create_outdir(args.o)?;

    //Creamos las carpetas orig y modif
    let orig_dir: String = format!("{}/orig", &created_outdir_path);
    fs::create_dir(&orig_dir)?;
    let modif_dir: String = format!("{}/modif", &created_outdir_path);
    fs::create_dir(&modif_dir)?;

    //Generar imagenes a partir de los dos pdf
    let res1: Vec<String> = pdfutil::pdf2imgs(&args.orig, &orig_dir, &args.iext)?;
    let orig_image_paths_slice: &[String] = res1.as_slice();
    let res2: Vec<String> = pdfutil::pdf2imgs(&args.modif, &modif_dir, &args.iext)?;
    let modif_image_paths_slice: &[String] = res2.as_slice();

    let mask_dir: String = format!("{}/mask", &created_outdir_path);
    fs::create_dir(&mask_dir)?;
    let res3: Vec<String> = imagick::mask_images(orig_image_paths_slice, modif_image_paths_slice, &mask_dir, &args.iext)?;
    let image_mask_paths_slice: &[String] = res3.as_slice();
    println!("{:?}", image_mask_paths_slice);

    //Sacar las coordenadas de los rectangulos blancos con imagemagick
    let crop_image_coords: Vec<Vec<imagick::Coords>> = imagick::paralel_rectangle_recognition(image_mask_paths_slice)?;
    println!("{:?}", &crop_image_coords);

    let crop_dir: String = format!("{}/crop", &created_outdir_path);
    fs::create_dir(&crop_dir)?;
    //Recortar las imagenes originales con las coordenadas obtenidas
    let crop_image_paths: Vec<Vec<String>> = imagick::crop_images(orig_image_paths_slice, &crop_image_coords, &crop_dir, &args.iext)?;
    println!("{:?}", &crop_image_paths);

    let trans_dir: String = format!("{}/trans", &created_outdir_path);
    fs::create_dir(&trans_dir)?;
    let res4: Vec<String> = pdfutil::pdf2imgs(&args.trans, &trans_dir, &args.iext)?;
    let trans_image_paths_slice: &[String] = res4.as_slice();
    //Pegamos las imagenes recortadas sobre las imagenes traducidas
    let res5: Vec<String> = imagick::paste_crop_images(trans_image_paths_slice, &crop_image_paths, &crop_image_coords)?;
    let last_image_paths_slice: &[String] = res5.as_slice();

    //Juntar todas las imagenes en un solo pdf
    //De momento lo hago manual porque no puede leer rutas relativas creo (al hacerlo manual si lee rutas relativas)
    //imgs2pdf(&trans_dir, &created_outdir_path)?;

    Ok(())
}

fn create_outdir(ostring: String) -> anyhow::Result<String> {
    //Crear el directorio a partir de un PathBuf porque a veces el usuario nos puede pasar un path de carpeta que no termina en "/" o el path "./"
    //Entonces usamos los path estandar porque ellos ya manejan estos casos
    let mut out_dir_path: PathBuf = PathBuf::from(&ostring);
    //Generar una cadena random para no sobreescribir los outputs anteriores
    let rand_str: String = random_string::generate(15, "abcdefghijklmnopqrstuvwxyz1234567890");
    let complete_dir_name: String = format!("BT_out_{}", &rand_str);
    out_dir_path.push(&complete_dir_name);
    //Debemos manejar el caso de que la carpeta exista
    fs::create_dir(&out_dir_path)?;
    
    Ok(out_dir_path.display().to_string())
}

