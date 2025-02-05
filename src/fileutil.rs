use std::fs::{self, DirEntry};

/*List files on a directory
Parameters:
- path: &str = The path to the directory.
- ext: &str = The extension used to filter the files in the directory.
All paths should be Unix valid.
Return:
- Result<Vec<String>> = The Result containing the paths of all the filtered files in the directory.
*/
pub fn list_files_from_path(path: &str, ext: &str) -> anyhow::Result<Vec<String>> { //El parametro ext no debe tener el punto, por ej: "png", "pdf"
    let items: fs::ReadDir = fs::read_dir(path)?;
    let mut str_path_items: Vec<String> = Vec::new();

    //Ignore Error() items
    for item in items {
        if let Ok(item_ok) = item {
            if is_file_with_ext(&item_ok, ext) {
                str_path_items.push(item_ok.path().display().to_string());
            }
        }
    }
    //str_path_items.sort_by() doesnt work on file paths
    alphanumeric_sort::sort_str_slice(&mut str_path_items[..]);

    Ok(str_path_items)
}
/*Filters a file depending on its extension.
Parameters:
- item: &DirEntry = An item of the list obtained from fs::read_dir
- ext: &str = The extension used to filter the item.
Return:
- bool = true if the item extension matches the extension, false otherwise.
*/
pub fn is_file_with_ext(item: &DirEntry, ext: &str) -> bool {
    //Returns false if item: doesn't have metadata, is not a file or doesn't match the extension, otherwise returns true.
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

    //Future: with try-catch we don't need so much nested ifs
}
