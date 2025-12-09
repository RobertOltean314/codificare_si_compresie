## Note

Am folosit scripturi Python pentru a genera grafice si excel-uri cu rezultatele testelor. Scripturile `.sh` pornesc serverul si ruleaza testele automat.

Rezultatele se salveaza in `analysis_results/` pentru fiecare proiect.

## Cerinte

- Rust 1.80+
- Python 3.8+
- Dependente Python: `requests`, `matplotlib`, `pandas`, `openpyxl`

Instalare dependente Python:

```bash
pip3 install requests matplotlib pandas openpyxl
```

## Structura

Proiectul contine 3 implementari separate:

- `huffman_v7/` - Huffman Static cu interfata web
- `lz77_v6/` - LZ77 cu analiza parametrilor
- `lzw_v3/` - LZW cu analiza performantei

Fiecare proiect are backend in Rust si o interfata web simpla folosind HTML, CSS si JS, nu am reusit sa implementez interfata folosind Angular, asa ca am ramas la o interfata simpla

## Cum se ruleaza

### Huffman Static

```bash
cd huffman_v7
cargo run --release
```

Serverul ruleaza la adresa: `http://localhost:8080` in browser.

### LZ77

```bash
cd lz77_v6
cargo run --release
```

Pentru analiza automata (testeaza toate combinatiile de parametri):

```bash
./run_analysis.sh fisier1.extensie fisier2.extensie
```

Scriptul testeaza 84 de combinatii (offset bits: 2-15, length bits: 2-7) si genereaza grafice + excel cu rezultatele in `analysis_results/`.

### LZW

```bash
cd lzw_v3
cargo run --release
```

Pentru analiza:

```bash
./run_analysis.sh fisier.txt
```

Testeaza 8 combinatii (dictionary width: 9-16 biti).
