ref: https://stackoverflow.com/questions/49639342/how-can-i-downgrade-or-install-an-older-version-of-a-tool-i-installed-with-carg

- ImageMagick 6.9.11-60 Q16 x86_64 2021-01-25 https://imagemagick.org
- qpdf 10.6.3
- pdftk port to java 3.2.2

Lo compilo abriendo una terminal en la carpeta donde esta el Cargo.toml y ejecuto:
- "cargo run -- --p1 ./path/orig.pdf --p2 ./path/modif.pdf --tr ./path/trans.pdf"