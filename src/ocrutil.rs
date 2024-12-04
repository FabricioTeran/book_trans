/*
0. tesseract input.png stdout -l eng --psm 1 hocr

1. call_ocr_get_bboxes_and_produce_txt_files: Analizamos el xml de tesseract y buscamos los items con la clase "ocr_par"
   - Obtenemos el bbox con el atributo "title" y creamos una matriz de arreglos de tuplas asociando el bbox con un 
     id (inventando un id)
   - A la misma vez obtenemos el texto del parrafo analizando todos los items "word" (sin hacer caso a los padres
     "line"), lo convertimos a string y le agregamos su ID al final de cada parrafo
     - Cada parrafo va a estar escrito en una linea, su id se va a encontrar antes de cada salto de linea, esto
       nos va a facilitar mucho la posterior re-lectura del archivo
   - Escribimos el texto en el archivo .txt de salida
   - 2 saltos de linea seguidos indican una nueva pagina

2. get_trans_txt_and_bboxes: Una vez traducido el archivo, lo leemos y creamos una nueva matriz de arreglos de tuplas usando la informacion 
   del anterior, buscando el id al final de cada linea del .txt obtenemos el bbox del anterior arreglo, lo guardamos 
   en la tupla y tambien guardamos todo el texto traducido del parrafo
   - Cada arreglo lo guardamos en una matriz, al leer dos saltos de linea en el .txt pasamos a la siguiente 
     iteracion del bucle padre para pasar a la siguiente pagina
*/

use std::process::{self, Command};

pub fn paralel_ocr(modif_images: &Vec<String>, lang: &String) -> anyhow::Result<()> {


  Ok(())
}

