use std::env;

use clap::Parser;
use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry};
use std::process::{self, Command};
use std::io::{self, Write};
use rayon::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    ///The path to the original pdf.
    #[arg(long)]
    p1: String,

    //The path to the modified pdf.
    #[arg(long)]
    p2: String,

    ///The path to the folder of the translated pdf. DEFAULT:./trans
    #[arg(long)]
    tr: String,

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
    let orig_image_paths: Vec<String> = pdf2imgs(&args.p1, &orig_dir, &args.iext)?;
    let modif_image_paths: Vec<String> = pdf2imgs(&args.p2, &modif_dir, &args.iext)?;

    let mask_dir: String = format!("{}/mask", &created_outdir_path);
    fs::create_dir(&mask_dir)?;
    let image_mask_paths: Vec<String> = mask_images(&orig_image_paths, &modif_image_paths, &mask_dir, &args.iext)?;
    println!("{:?}", &image_mask_paths);

    //Sacar las coordenadas de los rectangulos blancos con imagemagick
    let crop_image_coords: Vec<Vec<Coords>> = paralel_rectangle_recognition(&image_mask_paths)?;
    println!("{:?}", &crop_image_coords);

    let crop_dir: String = format!("{}/crop", &created_outdir_path);
    fs::create_dir(&crop_dir)?;
    //Recortar las imagenes originales con las coordenadas obtenidas
    let crop_image_paths: Vec<Vec<String>> = crop_images(&orig_image_paths, &crop_image_coords, &crop_dir, &args.iext)?;
    println!("{:?}", &crop_image_paths);

    let trans_dir: String = format!("{}/trans", &created_outdir_path);
    fs::create_dir(&trans_dir)?;
    let trans_image_paths: Vec<String> = pdf2imgs(&args.tr, &trans_dir, &args.iext)?;
    //Pegamos las imagenes recortadas sobre las imagenes traducidas
    let last_image_paths: Vec<String> = paste_crop_images(&trans_image_paths, &crop_image_paths, &crop_image_coords)?;

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

//pdftoppm -png -r 200 file.pdf outPath/name
fn pdf2imgs(pdf_path: &String, out_dir_path: &String, ext: &str) -> anyhow::Result<Vec<String>> {
    let result_paths: Vec<String>;
    let out_dir_and_name: String = format!("{}/out", out_dir_path);

    let out_message: process::Output = Command::new("pdftoppm")
        .args(["-png", "-r", "200", pdf_path, &out_dir_and_name])
        .output()?;
    io::stdout().write_all(&out_message.stdout)?;
    io::stderr().write_all(&out_message.stderr)?;

    result_paths = list_files_from_path(out_dir_path, ext)?;

    Ok(result_paths)
}

//Crear struct Coords x,y,w,h
#[derive(Debug)]
struct Coords {
    x: i32,
    y: i32,
    w: i32,
    h: i32
}
//La funcion devuelve Vec<Vec<Coords>> [0][1].x = Pagina 0, img 1, coordenada X
//Hemos bajado de 106s a 70s en 50 archivos
fn paralel_rectangle_recognition(image_paths: &Vec<String>) -> anyhow::Result<Vec<Vec<Coords>>> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()?;
    //pool.install devuelve el ultimo valor de su closure
    //pool.install solo regresa cuando todos sus threads se han ejecutado
    let res: Vec<anyhow::Result<Vec<Coords>>> = pool.install(|| {
        image_paths.par_iter().map(|p: &String| rectangle_recognition(p)).collect()
    });
    //Si hay un error, todo el vector de Results se convierte en un Result Err
    let res_check: anyhow::Result<Vec<Vec<Coords>>> = res.into_iter().collect(); //No podemos usar directamente el operador ? porque le tenemos que indicar a .collect como debe transformar el vector, es la implementacion de collect de Result
    let res_ok: Vec<Vec<Coords>> = res_check?;

    Ok(res_ok)
}
fn rectangle_recognition(path: &String) -> anyhow::Result<Vec<Coords>> {
    let ignore_file: &str = "null.png";

    //Ejecutar procesos paralelos para mejorar el rendimiento
    let out_message: process::Output = Command::new("convert")
        .args([path, "-define", "connected-components:exclude-header=true", "-define", "connected-components:area-threshold=100", "-define", "connected-components:verbose=true", "-connected-components", "4", ignore_file])
        .output()?;
    io::stderr().write_all(&out_message.stderr)?;

    //Filtrar las lineas que acaben en gray(255) porque son las coords de las figuras blancas, gray(0) son figuras de color negro
    //Luego separar el output por saltos de linea, luego separarlo por espacios
    //Tomar solo la 2da frase de cada linea
    let out_str: String = String::from_utf8(out_message.stdout)?;
    let str_lines: Vec<&str> = out_str.lines()
        .filter(|&x| x.ends_with("gray(255)"))
        .collect();

    let mut str_coords: Vec<Coords> = Vec::new(); //vector de struct Coords
    str_coords.reserve_exact(str_lines.len()); //Mejora de rendimiento
    for line in str_lines {
        //0x0+0+0 WxH+X1+Y1
        let coords: &str = line.split_whitespace().collect::<Vec<&str>>()[1];
        let individual_coords: Vec<&str> = coords.split(['x', '+']).collect();
        let into_coords: Coords = Coords{
            x: individual_coords[2].parse()?,
            y: individual_coords[3].parse()?,
            w: individual_coords[0].parse()?,
            h: individual_coords[1].parse()?
        };

        str_coords.push(into_coords);
    }

    Ok(str_coords)
}

fn mask_images(orig_images: &Vec<String>, modif_images: &Vec<String>, out_path: &String, ext: &String) -> anyhow::Result<Vec<String>> {
    //Comparamos el vector con menor longitud para que haya correspondencia 1-1 entre ambas carpetas
    //Si es mayor o igual devolvemos modif porque es menor o igual, si es menor, devolvemos orig porque es menor
    let min_len = match orig_images.len() >= modif_images.len() {
        true => modif_images.len(),
        false => orig_images.len()
    };
    let mut mask_out_files: Vec<String> = Vec::new();
    //Mejoramos el rendimiento porque hacemos menos llamadas al heap para crecer el tamano del vector
    mask_out_files.reserve_exact(min_len);

    for i in 0..min_len {
        let out_arg: String = format!("{}/{}.{}", out_path, i, ext);

        //Dificil problema solucionado: El comando dssim fallaba porque tomaba el -o como otro argumento aparte, entonces al concatenarlo con out_path fallaba
        //En proximas ocasiones probar varias combinaciones, pero cada palabra separada por espacios es considerada como otro parametro
        //Tuvimos que usar el flag -quiet porque las alertas de imagemagick hacian que el programa los tomara como errores
        let out_message: process::Output = Command::new("compare")
            .args([&modif_images[i], &orig_images[i], "-compose", "src", "-highlight-color", "white", "-lowlight-color", "black", "-quiet", &out_arg])
            .output()?;
        //Checkear salida y errores de la ejecucion del comando
        //Si no hay salida en el comando, no devuelve nada ni produce error a pesar de usar ?
        io::stdout().write_all(&out_message.stdout)?;
        io::stderr().write_all(&out_message.stderr)?;

        mask_out_files.push(out_arg);
    }

    Ok(mask_out_files)
}

//El parametro ext no debe tener el punto, por ej: "png", "pdf"
fn list_files_from_path(path: &String, ext: &str) -> anyhow::Result<Vec<String>> {
    let items: fs::ReadDir = fs::read_dir(path)?;
    let mut str_path_items: Vec<String> = Vec::new();

    //Ignoramos los items que den error
    for item in items {
        if let Ok(item_ok) = item {
            if is_file_with_ext(&item_ok, ext) {
                str_path_items.push(item_ok.path().display().to_string());
            }
        }
    }

    //Ordenamos los archivos por orden alfanumerico
    //str_path_items.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    //La anterior instruccion no funciona con paths de archivos, por eso utilice el crate alphanumeric_sort
    alphanumeric_sort::sort_str_slice(&mut str_path_items[..]);

    Ok(str_path_items)
}
fn is_file_with_ext(item: &DirEntry, ext: &str) -> bool {
    //Filtrar los elementos aqui mismo,  porque luego filtrar cadenas string es mas dificil, como filtrariamos carpetas que tengan nombre ".png"?
    //Devolvemos False si un item no tiene metadatos, si no es archivo y si no tiene la extension buscada, caso contrario devuelve True
    if let Ok(meta) = item.metadata() {
        if meta.is_file() {
            if let Some(ex) = item.path().extension() {
                if ex == ext {
                    return true
                }
            }
        }
    }

    false

    //Con un try-catch podriamos manejar los errores de falta de metadatos y falta de extension sin necesidad de anidar tantos bloques
}

//convert orig.png -crop 1x1+2+3 result.png
fn crop_images(orig_images: &Vec<String>, imcoords: &Vec<Vec<Coords>>, out_path: &String, ext: &String) -> anyhow::Result<Vec<Vec<String>>> {
    let mut result_paths: Vec<Vec<String>> = Vec::new();
    result_paths.reserve_exact(orig_images.len());
    
    for i in 0..orig_images.len() {
        result_paths.push(Vec::new());

        for j in 0..imcoords[i].len() {
            let fmt_coords: String = format!("{}x{}+{}+{}", imcoords[i][j].w, imcoords[i][j].h, imcoords[i][j].x, imcoords[i][j].y);
            let out_file: String = format!("{}/{}_{}.{}", out_path, i, j, ext);

            let out_message: process::Output = Command::new("convert")
                .args([&orig_images[i], "-crop", &fmt_coords, "-quiet", &out_file])
                .output()?;
            io::stdout().write_all(&out_message.stdout)?;
            io::stderr().write_all(&out_message.stderr)?;

            result_paths[i].push(out_file);
        }
    }

    Ok(result_paths)
}

//composite -geometry +X+Y front.png back.png out.png
fn paste_crop_images(trans_images: &Vec<String>, crop_images: &Vec<Vec<String>>, crop_coords: &Vec<Vec<Coords>>) -> anyhow::Result<Vec<String>> {
    let mut result_paths: Vec<String> = Vec::new();
    result_paths.reserve_exact(trans_images.len());
    
    for i in 0..trans_images.len() {
        //Si la pagina no tiene crops, no entra a este bucle porque se forma el rango 0..0
        for j in 0..crop_images[i].len() {
            //Recalculamos porque google trans cambia el tamano de las imagenes, les aumenta 300px X y 350px Y
            let recalc_x: i32 = crop_coords[i][j].x - 150;
            let recalc_y: i32 = crop_coords[i][j].y - 175;
            let coords: String = format!("+{}+{}", recalc_x, recalc_y);

            //Aqui tanto la back image como el resultado es el mismo path para sobreescribirlo en cada iteracion del 2do loop
            let out_message: process::Output = Command::new("composite")
                .args(["-geometry", &coords, &crop_images[i][j], &trans_images[i], &trans_images[i]])
                .output()?;
            io::stdout().write_all(&out_message.stdout)?;
            io::stderr().write_all(&out_message.stderr)?;
        }

        //Esta afuera del 2do bucle porque si no agregariamos 2 o mas veces el mismo path (ya que agregamos varias imagenes a cada imagen)
        result_paths.push(trans_images[i].clone());
    }

    Ok(result_paths)
}

//img2pdf path/*.png -o out.pdf
//Confiamos en el orden de las imagenes porque terminan en -001, -002, entonces un ordenamiento alfanumerico sirve
fn imgs2pdf(img_dir: &String, out_dir: &String) -> anyhow::Result<()> {
    let all_png: String = format!("{}/*.png", img_dir);
    let out_file: String = format!("{}/out.pdf", out_dir);

    let out_message: process::Output = Command::new("img2pdf")
        .args([&all_png, "-o", &out_file])
        .output()?;
    io::stdout().write_all(&out_message.stdout)?;
    io::stderr().write_all(&out_message.stderr)?;

    Ok(())
}