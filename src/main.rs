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
    let image_difference_paths: Vec<String> = image_difference_dssim(&orig_images, &modif_images, &(out_dir_path.display().to_string()))?;
    println!("{:?}", image_difference_paths);

    //Sacar las coordenadas de los rectangulos blancos con opencv

    //Recortar las imagenes originales con las coordenadas obtenidas

    //Hacer DLA con libreria layout-ort y marcar los bloques de texto con algun distintivo que pueda sobrevivir la traduccion
    //Agregar algun texto al final de cada parrafo para indicar su id (que va a estar asociado con sus coordenadas)

    //Traducir los parrafos exportando todo el texto como un archivo pdf

    //Ubicar el texto con sus coordenadas y las imagenes recortadas con sus coordenadas en un archivo pdf
    //Algoritmo para empujar bloques de texto hacia abajo si se superponen

    Ok(())
}

fn image_difference_dssim(orig_images: &Vec<String>, modif_images: &Vec<String>, out_path: &String) -> anyhow::Result<Vec<String>> {
    //Comparamos el vector con menor longitud para que haya correspondencia 1-1 entre ambas carpetas
    //Si es mayor o igual devolvemos modif porque es menor o igual, si es menor, devolvemos orig porque es menor
    let min_len = match orig_images.len() >= modif_images.len() {
        true => modif_images.len(),
        false => orig_images.len()
    };
    let mut dssim_out_files: Vec<String> = Vec::new();
    //Mejoramos el rendimiento porque hacemos menos llamadas al heap para crecer el tamano del vector
    dssim_out_files.reserve_exact(min_len);

    for i in 0..min_len {
        let out_arg: String = format!("{}/{}.png", out_path, i);

        //Dificil problema solucionado: El comando dssim fallaba porque tomaba el -o como otro argumento aparte, entonces al concatenarlo con out_path fallaba
        //En proximas ocasiones probar varias combinaciones, pero cada palabra separada por espacios es considerada como otro parametro
        /*let out_message: process::Output = Command::new("dssim")
            .args(["-o", &out_arg, &orig_images[i], &modif_images[i]])
            .output()?;
        */
        //Tuvimos que usar el flag -quiet porque las alertas de imagemagick hacian que el programa los tomara como errores
        let out_message: process::Output = Command::new("compare")
            .args([&modif_images[i], &orig_images[i], "-compose", "src", "-highlight-color", "white", "-lowlight-color", "none", "-quiet", &out_arg])
            .output()?;

        //Checkear salida y errores de la ejecucion del comando
        //Si no hay salida en el comando, no devuelve nada ni produce error a pesar de usar ?
        io::stdout().write_all(&out_message.stdout)?;
        io::stderr().write_all(&out_message.stderr)?;

        //Eliminar los archivos en diferentes escalas que produce dssim
        //dssim agrega la string -0, -1, -2, -3, etc al nombre del arhcivo que especifiquemos en -o, asi que solo nos quedamos con los archivos que tengan el -0 y eliminamos el resto
        //Ya tenemos el nombre de los arhcivos creados recientemente, asi que solo les aumentamos el -1, -2, a su path para eliminarlos, no necesitamos hacer un read_dir
        //Cada llamada a dssim produce 5 archivos, entonces eliminamos los archivos (-1,-2,-3,-4)
        //dssim solo produce archivos .png
        //let delete_file1: String = format!("{}-1.png", &out_arg); fs::remove_file(&delete_file1)?;
        //let delete_file2: String = format!("{}-2.png", &out_arg); fs::remove_file(&delete_file2)?;
        //let delete_file3: String = format!("{}-3.png", &out_arg); fs::remove_file(&delete_file3)?;
        //let delete_file4: String = format!("{}-4.png", &out_arg); fs::remove_file(&delete_file4)?;

        //let keep_file0: String = format!("{}-0.png", &out_arg);
        dssim_out_files.push(out_arg);
    }

    //Ya tenemos los paths de los archivos guardados asi que no tenemos que listarlos de nuevo
    //Investigar si opencv requiere de archivos abiertos o solo paths

    Ok(dssim_out_files)
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