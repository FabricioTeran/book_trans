use std::process::{self, Command};
use std::io::{self, Write};

//Resolver: no funcionan los pipes
pub fn mask_and_recognize(orig_images: &[String], modif_images: &[String]) -> anyhow::Result<Vec<Vec<Coords>>> {
    //Comparamos el vector con menor longitud para que haya correspondencia 1-1 entre ambas carpetas
    //Si es mayor o igual devolvemos modif porque es menor o igual, si es menor, devolvemos orig porque es menor
    let min_len = match orig_images.len() >= modif_images.len() {
        true => modif_images.len(),
        false => orig_images.len()
    };
    let ignore_file: &str = "null.png";
    let mut final_vec: Vec<Vec<Coords>> = Vec::new();
    final_vec.reserve_exact(min_len);

    for i in 0..min_len {
        let mask_args: String = format!("compare {} {} -compose src -highlight-color white -lowlight-color black -quiet miff:-", modif_images[i], orig_images[i]);
        let recognize_args: String = format!("convert - -define connected-components:exclude-header=true -define connected-components:area-threshold=100 -define connected-components:verbose=true -connected-components 4 {}", ignore_file);
        let complete_args: String = format!("{} | {}", mask_args, recognize_args);

        let out_message: process::Output = Command::new("bash")
            .args(["-c", &complete_args])
            .output()?;
        io::stdout().write_all(&out_message.stdout)?;
        io::stderr().write_all(&out_message.stderr)?;

        //Filtrar las lineas que acaben en gray(255) porque son las coords de las figuras blancas, gray(0) son figuras de color negro
        //Luego separar el output por saltos de linea, luego separarlo por espacios
        //Tomar solo la 2da frase de cada linea
        //Ahora como transimitimos el formato miff, cambiamos la conicidencia a srgba(255,255,255,1)
        let out_str: String = String::from_utf8(out_message.stdout)?;
        let str_lines: Vec<&str> = out_str.lines()
            .filter(|&x| x.ends_with("srgba(255,255,255,1)"))
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

        final_vec.push(str_coords);
    }

    Ok(final_vec)
}

//Crear struct Coords x,y,w,h
#[derive(Debug)]
pub struct Coords {
    x: i32,
    y: i32,
    w: i32,
    h: i32
}

//convert orig.png -crop 1x1+2+3 result.png
//convert orig.png -crop 400x400+50+50 miff:- | composite -geometry +50+50 - modif.png result.png
pub fn crop_and_paste(orig_images: &[String], trans_images: &[String], imcoords: &Vec<Vec<Coords>>) -> anyhow::Result<()> {
    for i in 0..trans_images.len() {
        for j in 0..imcoords[i].len() {
            let fmt_coords: String = format!("{}x{}+{}+{}", imcoords[i][j].w, imcoords[i][j].h, imcoords[i][j].x, imcoords[i][j].y);
            let xy_coords: String = format!("+{}+{}", imcoords[i][j].x, imcoords[i][j].y);
            
            let crop_args: String = format!("convert {} -crop {} -quiet miff:-", orig_images[i], fmt_coords);
            let paste_args: String = format!("composite -geometry {} - {} {}", xy_coords, trans_images[i], trans_images[i]);
            let complete_args: String = format!("{} | {}", crop_args, paste_args);

            let out_message: process::Output = Command::new("bash")
                .args(["-c", &complete_args])
                .output()?;
            io::stdout().write_all(&out_message.stdout)?;
            io::stderr().write_all(&out_message.stderr)?;
        }
    }

    Ok(())
}
