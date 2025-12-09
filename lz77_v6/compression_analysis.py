import requests
import os
import sys
import time
from pathlib import Path
import matplotlib.pyplot as plt
import pandas as pd
from datetime import datetime

SERVER_URL = "http://localhost:8080"
ENCODE_ENDPOINT = f"{SERVER_URL}/api/encode"

class LZ77CompressionTester:
    def __init__(self, output_dir="analysis_results"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        self.results = []
        
    def compress_file(self, file_path, offset_bits, length_bits):
        file_path = Path(file_path)
        
        if not file_path.exists():
            raise FileNotFoundError(f"Fisier negasit: {file_path}")
        
        with open(file_path, 'rb') as f:
            file_data = list(f.read())
        
        payload = {
            'filename': file_path.name,
            'file_data': file_data,
            'offset_bits': offset_bits,
            'length_bits': length_bits,
            'display_tokens': False
        }
        
        try:
            response = requests.post(ENCODE_ENDPOINT, json=payload)
            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            print(f"Eroare la compresie: {e}")
            if hasattr(e.response, 'text'):
                print(f"Raspuns: {e.response.text}")
            raise
    
    def test_file(self, file_path):
        file_path = Path(file_path)
        print(f"\n{'='*80}")
        print(f"Testare fisier: {file_path.name}")
        print(f"Dimensiune originala: {file_path.stat().st_size:,} bytes")
        print(f"{'='*80}\n")
        
        test_results = []
        
        total_tests = 14 * 6
        current_test = 0
        
        for offset_bits in range(2, 16):
            for length_bits in range(2, 8):
                current_test += 1
                print(f"[{current_test}/{total_tests}] Testare: offset={offset_bits} biti, length={length_bits} biti...")
                
                try:
                    result = self.compress_file(file_path, offset_bits, length_bits)
                    
                    test_results.append({
                        'fisier': file_path.name,
                        'biti_offset': offset_bits,
                        'biti_length': length_bits,
                        'offset_maxim': (1 << offset_bits) - 1,
                        'length_maxim': (1 << length_bits) - 1,
                        'dimensiune_originala': result['original_size'],
                        'dimensiune_comprimata': result['compressed_size'],
                        'rata_compresie': result['compression_ratio'],
                        'spatiu_salvat': result['original_size'] - result['compressed_size'],
                        'procent_salvat': result['compression_ratio']
                    })
                    
                    print(f"  ✓ Comprimat: {result['compressed_size']:,} bytes "
                          f"({result['compression_ratio']:.2f}% rata)\n")
                    
                except Exception as e:
                    print(f"  ✗ Esuat: {e}\n")
                    continue
        
        self.results.extend(test_results)
        return test_results
    
    def generate_graphs(self, file_results, filename):
        if not file_results:
            print(f"Nu exista rezultate pentru grafic: {filename}")
            return
            
        df = pd.DataFrame(file_results)
        
        fig, ax = plt.subplots(figsize=(12, 8))
        
        pivot_size = df.pivot(index='biti_length', columns='biti_offset', values='dimensiune_comprimata')
        
        im = ax.imshow(pivot_size, cmap='RdYlGn_r', aspect='auto')
        ax.set_xlabel('Biti Offset (Dimensiune Search Buffer)', fontweight='bold', fontsize=12)
        ax.set_ylabel('Biti Length (Match Length)', fontweight='bold', fontsize=12)
        ax.set_title(f'Impactul Dimensiunii Bufferelor asupra Compresiei\n{filename}', 
                     fontweight='bold', fontsize=14, pad=20)
        
        ax.set_xticks(range(len(pivot_size.columns)))
        ax.set_xticklabels(pivot_size.columns)
        ax.set_yticks(range(len(pivot_size.index)))
        ax.set_yticklabels(pivot_size.index)
        
        cbar = plt.colorbar(im, ax=ax)
        cbar.set_label('Dimensiune Comprimata (bytes)', rotation=270, labelpad=20, fontweight='bold')
        
        for i in range(len(pivot_size.index)):
            for j in range(len(pivot_size.columns)):
                value = pivot_size.iloc[i, j]
                ax.text(j, i, f'{int(value):,}',
                       ha="center", va="center", color="black", fontsize=7)
        
        plt.tight_layout()
        
        pdf_path = self.output_dir / f"{Path(filename).stem}_heatmap.pdf"
        plt.savefig(pdf_path, dpi=300, bbox_inches='tight')
        print(f"✓ Heatmap PDF salvat: {pdf_path}")
        
        plt.close()
        
        self.generate_results_file(df, filename)
    
    def generate_results_file(self, df, filename):
        best_idx = df['rata_compresie'].idxmax()
        best = df.loc[best_idx]
        worst_idx = df['rata_compresie'].idxmin()
        worst = df.loc[worst_idx]
        
        original_size = best['dimensiune_originala']
        
        results_text = f"""
{'='*90}
ANALIZA COMPRESIE LZ77 - REZULTATE
{'='*90}

Fisier: {filename}
Dimensiune Originala: {original_size:,} bytes
Data Analizei: {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}

{'='*90}
CEA MAI BUNA CONFIGURATIE
{'='*90}

Offset: {best['biti_offset']} biti
  → Buffer cautare: {best['offset_maxim']} bytes
  → Poate cauta pana la {best['offset_maxim']} bytes inapoi pentru potriviri

Length: {best['biti_length']} biti
  → Potrivire maxima: {best['length_maxim']} bytes
  → Poate coda potriviri de pana la {best['length_maxim']} bytes

Dimensiune Comprimata: {best['dimensiune_comprimata']:,} bytes
Rata Compresie: {best['rata_compresie']:.2f}%
Spatiu Salvat: {best['spatiu_salvat']:,} bytes

{'='*90}
CEA MAI SLABA CONFIGURATIE
{'='*90}

Offset: {worst['biti_offset']} biti (Buffer: {worst['offset_maxim']} bytes)
Length: {worst['biti_length']} biti (Match: {worst['length_maxim']} bytes)
Dimensiune Comprimata: {worst['dimensiune_comprimata']:,} bytes
Rata Compresie: {worst['rata_compresie']:.2f}%

{'='*90}
EXPLICATIE
{'='*90}

BITI OFFSET (Orizontal pe heatmap):
  - Controleaza cat de departe in trecut poate cauta encoderul pentru potriviri
  - Valori mai mari = buffer de cautare mai mare = mai multe potentiale potriviri
  - Interval testat: 2-15 biti (3 pana la 32,767 bytes lookback)
  - Tradeoff: Mai multi biti = mai multa flexibilitate, dar token-uri mai mari

BITI LENGTH (Vertical pe heatmap):
  - Controleaza lungimea maxima a unei singure potriviri
  - Valori mai mari = pattern-uri mai lungi pot fi codate eficient
  - Interval testat: 2-7 biti (3 pana la 127 bytes per potrivire)
  - Tradeoff: Mai multi biti = potriviri mai lungi, dar token-uri mai mari

CODIFICARE CULORI (Heatmap):
  - Verde = Compresie mai buna (output mai mic)
  - Galben = Compresie moderata
  - Rosu = Compresie slaba (output mai mare)

DE CE ACEASTA CONFIGURATIE ESTE CEA MAI BUNA:
  
  Configuratia optima ({best['biti_offset']} biti offset, {best['biti_length']} biti length) 
  obtine cel mai bun echilibru intre:
  
  1. Flexibilitatea de cautare: Un buffer de {best['offset_maxim']} bytes permite 
     gasirea de potriviri suficient de departe in trecut.
  
  2. Lungimea potrivirilor: Potriviri de pana la {best['length_maxim']} bytes permit
     codarea eficienta a pattern-urilor repetitive.
  
  3. Dimensiunea token-urilor: Fiecare token foloseste 
     {best['biti_offset']} + {best['biti_length']} + 8 = {best['biti_offset'] + best['biti_length'] + 8} biti
     pentru a coda (offset, length, next_char).
  
  Pentru acest tip de continut, configuratia balansata produce cea mai mica
  dimensiune de fisier comprimat.

{'='*90}
TOATE COMBINATIILE TESTATE (Top 10)
{'='*90}

"""
        
        top10 = df.nlargest(10, 'rata_compresie')
        for idx, (_, row) in enumerate(top10.iterrows(), 1):
            results_text += f"{idx:2d}. Offset={row['biti_offset']:2d} biti, Length={row['biti_length']} biti → "
            results_text += f"{row['dimensiune_comprimata']:8,} bytes ({row['rata_compresie']:6.2f}%)\n"
        
        results_text += f"\n{'='*90}\n"
        results_text += f"Total combinatii testate: {len(df)}\n"
        results_text += f"{'='*90}\n"
        
        results_path = self.output_dir / f"{Path(filename).stem}_results.txt"
        with open(results_path, 'w', encoding='utf-8') as f:
            f.write(results_text)
        
        print(f"✓ Fisier rezultate salvat: {results_path}")
    
    def export_to_excel(self):
        if not self.results:
            print("Nu exista rezultate de exportat!")
            return
        
        df = pd.DataFrame(self.results)
        
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        excel_path = self.output_dir / f"lz77_combinatii_{timestamp}.xlsx"
        
        with pd.ExcelWriter(excel_path, engine='openpyxl') as writer:
            for filename in df['fisier'].unique():
                file_df = df[df['fisier'] == filename]
                sheet_name = filename[:31]
                file_df.to_excel(writer, sheet_name=sheet_name, index=False)
        
        print(f"\n✓ Fisier Excel salvat: {excel_path}")
    
    def print_summary(self):
        if not self.results:
            print("Nu exista rezultate de sumarizat!")
            return
        
        df = pd.DataFrame(self.results)
        
        print("\n" + "="*80)
        print("SUMAR ANALIZA COMPRESIE LZ77")
        print("="*80)
        
        for filename in df['fisier'].unique():
            file_df = df[df['fisier'] == filename]
            best_idx = file_df['rata_compresie'].idxmax()
            best = file_df.loc[best_idx]
            
            print(f"\nFisier: {filename}")
            print(f"  Dimensiune originala: {best['dimensiune_originala']:,} bytes")
            print(f"  Cea mai buna configuratie: Offset={best['biti_offset']} biti, Length={best['biti_length']} biti")
            print(f"  Offset maxim: {best['offset_maxim']}, Length maxim: {best['length_maxim']}")
            print(f"  Dimensiune comprimata: {best['dimensiune_comprimata']:,} bytes")
            print(f"  Rata compresie: {best['rata_compresie']:.2f}%")
            print(f"  Spatiu salvat: {best['spatiu_salvat']:,} bytes")


def main():
    try:
        response = requests.get(SERVER_URL, timeout=2)
        print("✓ Serverul ruleaza")
    except:
        print("✗ EROARE: Serverul nu ruleaza!")
        print(f" cargo run pentru a porni serverul")
        sys.exit(1)

    if len(sys.argv) > 1:
        file_paths = sys.argv[1:]
    else:
        file_paths = []
        while True:
            path = input("Cale fisier: ").strip()
            if not path:
                break
            file_paths.append(path)
    
    if not file_paths:
        print("Niciun fisier specificat!")
        sys.exit(1)
    
    tester = LZ77CompressionTester()
    
    for file_path in file_paths:
        try:
            file_results = tester.test_file(file_path)
            if file_results:
                tester.generate_graphs(file_results, Path(file_path).name)
        except FileNotFoundError as e:
            print(f"✗ Eroare: {e}")
            continue
        except Exception as e:
            print(f"✗ Eroare neasteptata la testarea {file_path}: {e}")
            import traceback
            traceback.print_exc()
            continue
    
    if tester.results:
        tester.export_to_excel()
        tester.print_summary()
        
        print(f"\n{'='*80}")
        print(f"Analiza completa! Rezultate salvate in: {tester.output_dir}")
        print(f"{'='*80}\n")
    else:
        print("\nNu au fost generate rezultate.")


if __name__ == "__main__":
    main()
