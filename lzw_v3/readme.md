## Cerinte

### Rust

- Rust 1.70 sau mai recent
- Cargo (vine cu Rust)

### Python (pentru analiza)

- Python 3.8 sau mai recent
- Biblioteca: requests, matplotlib, pandas, openpyxl

## Instalare

### Instalare dependente Rust

```bash
cargo build
```

### Instalare dependente Python

```bash
pip install -r requirements.txt
```

## Utilizare

### Pornire Server

```bash
cargo run
```

Serverul va rula la adresa: http://localhost:8080 in browser

### Analiza Compresiei

Am decis sa fac un script in Python pentru a obtine rezultate, dat fiind faptul ca in trecut am mai facut prelucrare de date la alte materii

Pentru a rula scriptul:

```bash
./run_analysis.sh
```

Sau manual:

```bash
python3 compression_analysis.py fisier1.txt fisier2.png
```

Script-ul va:

- Testa fiecare fisier cu 8 configuratii diferite (1 auto-update + 7 manual cu 9-15 biti)
- Genera grafice PDF cu analiza comparativa
- Exporta rezultatele in Excel
- Identifica automat cea mai buna configuratie

Rezultatele se salveaza in directorul `analysis_results/`.

## Testare

Fisiere de test incluse:

- `test_input.txt` - Fisier text
- `test_59_2.csv` - Fisier csv

Pentru a rula analiza pe aceste fisiere:

```bash
./run_analysis.sh
```

## Limitari

- Am implementat doar strategia Empty, nu si Freeze
