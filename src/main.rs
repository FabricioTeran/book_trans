extern crate clap;
extern crate anyhow;
extern crate alphanumeric_sort;
extern crate random_string;

use std::env;

use clap::Parser;
use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry};
use std::process::{self, Command};
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    ///The path to the folder of the original images. DEFAULT:./orig
    #[arg(long, default_value = "./orig")]
    orig: String,

    ///The path to the folder of the modified images. DEFAULT:./modif
    #[arg(long, default_value = "./modif")]
    modif: String,

    ///The input language, the language of the source files. DEFAULT:eng
    #[arg(long, default_value = "eng")]
    il: String,

    ///The output language, the language of the source files. DEFAULT:esp
    #[arg(long, default_value = "esp")]
    ol: String,

    ///The number of generated pdf files. DEFAULT:1
    #[arg(long, default_value_t = 1)]
    pdfcount: i32,

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
    //Tomar los paths de orig,modif y o
    let orig_path: &Path = Path::new(&args.orig);
    let modif_path: &Path = Path::new(&args.modif);
    let orig_images: Vec<String> = list_files_from_path(orig_path, &args.iext)?;
    let modif_images: Vec<String> = list_files_from_path(modif_path, &args.iext)?;

    //Crear el directorio a partir de un PathBuf porque a veces el usuario nos puede pasar un path de carpeta que no termina en "/" o el path "./"
    //Entonces usamos los path estandar porque ellos ya manejan estos casos
    let mut out_dir_path: PathBuf = PathBuf::from(&args.o);
    //Generar una cadena random para no sobreescribir los outputs anteriores
    let rand_str: String = random_string::generate(15, "abcdefghijklmnopqrstuvwxyz1234567890");
    let complete_dir_name: String = format!("BT_out_{}", &rand_str);
    out_dir_path.push(&complete_dir_name);
    //Debemos manejar el caso de que la carpeta exista
    fs::create_dir(&out_dir_path)?;

    //Pasar los paths de orig y modif a una funcion que va a llamar al comando dssim
    let image_mask_paths: Vec<String> = mask_images(&orig_images, &modif_images, &(out_dir_path.display().to_string()))?;
    println!("{:?}", image_mask_paths);

    //Sacar las coordenadas de los rectangulos blancos con opencv
    //Con el .len de image_mask_paths sabemos cuanto espacio debe ocupar el vector final
    let page_image_coords: Vec<Vec<Coords>> = rectangle_recognition(&image_mask_paths, image_mask_paths.len())?;
    println!("{:?}", page_image_coords);

    //Recortar las imagenes originales con las coordenadas obtenidas

    //Hacer DLA con libreria layout-ort y marcar los bloques de texto con algun distintivo que pueda sobrevivir la traduccion
    //Agregar algun texto al final de cada parrafo para indicar su id (que va a estar asociado con sus coordenadas)
    //Extraer el texto del archivo producido por layout-ort y colocarlo en un archivo pdf creado y exportarlo

    //Esperamos que el usuario traduzca el archivo y luego presione alguna tecla

    //Extraer el texto del pdf traducido y asociar los parrafos a su id para asociarles sus coordenadas
    //Ubicar el texto con sus coordenadas y las imagenes recortadas con sus coordenadas en un archivo pdf
    //Algoritmo para empujar bloques de texto hacia abajo si se superponen

    Ok(())
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
fn rectangle_recognition(image_paths: &Vec<String>, image_count: usize) -> anyhow::Result<Vec<Vec<Coords>>> {
    //Reservar espacio con el numero de paginas para mejorar rendimiento
    let mut result_vec: Vec<Vec<Coords>> = Vec::new();
    result_vec.reserve_exact(image_count); //Mejora de rendimiento
    let ignore_file: &str = "null.png";

    for path in image_paths {
        //Ejecutar procesos paralelos para mejorar el rendimiento
        let out_message: process::Output = Command::new("convert")
            .args([path, "-morphology", "close", "disk:8", "-type", "bilevel", "-define", "connected-components:exclude-header=true", "-define", "connected-components:mean-color=true", "-define", "connected-components:area-threshold=100", "-define", "connected-components:verbose=true", "-connected-components", "8", ignore_file])
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

        result_vec.push(str_coords);
    }

    Ok(result_vec)
}

fn mask_images(orig_images: &Vec<String>, modif_images: &Vec<String>, out_path: &String) -> anyhow::Result<Vec<String>> {
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
        let out_arg: String = format!("{}/{}.png", out_path, i);

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

fn list_files_from_path(path: &Path, ext: &str) -> anyhow::Result<Vec<String>> {
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