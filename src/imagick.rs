use rayon::prelude::*;
use std::process::{self, Command};
use std::io::{self, Write};

//Crear struct Coords x,y,w,h
#[derive(Debug)]
pub struct Coords {
    x: i32,
    y: i32,
    w: i32,
    h: i32
}
//La funcion devuelve Vec<Vec<Coords>> [0][1].x = Pagina 0, img 1, coordenada X
//Hemos bajado de 106s a 70s en 50 archivos
pub fn paralel_rectangle_recognition(image_paths: &[String]) -> anyhow::Result<Vec<Vec<Coords>>> {
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
pub fn rectangle_recognition(path: &str) -> anyhow::Result<Vec<Coords>> {
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

pub fn mask_images(orig_images: &[String], modif_images: &[String], out_path: &str, ext: &str) -> anyhow::Result<Vec<String>> {
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

//convert orig.png -crop 1x1+2+3 result.png
pub fn crop_images(orig_images: &[String], imcoords: &Vec<Vec<Coords>>, out_path: &str, ext: &str) -> anyhow::Result<Vec<Vec<String>>> {
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
pub fn paste_crop_images(trans_images: &[String], crop_images: &Vec<Vec<String>>, crop_coords: &Vec<Vec<Coords>>) -> anyhow::Result<Vec<String>> {
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

//Crear un supercomando que haga todo en vez de crear archivos
//Dirigir cada mascara al reconocimiento de rectangulos sin guardar las mascaras
//Luego cropear las imagenes con las coords obtenidas y dirigir estos cortes a las imagenes traducidas
//No puedo conectar a menos que halle la forma de cortar el output de convert para pipearlo al siguiente comando