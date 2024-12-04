Antes de usar el programa debemos tener instalado image-magick, Version: ImageMagick 6.9.11-60 Q16 x86_64 2021-01-25 https://imagemagick.org
- Tambien debemos instalar opencv con: sudo apt install libopencv-dev clang libclang-dev
  ref: https://github.com/twistedfall/opencv-rust/blob/master/INSTALL.md



ref: https://stackoverflow.com/questions/49639342/how-can-i-downgrade-or-install-an-older-version-of-a-tool-i-installed-with-carg

- Debo tener instalado pdftoppm 22.02.0
- img2pdf 0.4.2

Lo compilo abriendo una terminal en la carpeta donde esta el Cargo.toml y ejecuto:
- "cargo run -- --p1 ./path/orig.pdf --p2 ./path/modif.pdf --tr ./path/trans.pdf"