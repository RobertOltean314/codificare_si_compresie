import requests
import os
import sys
import time
import base64
from pathlib import Path
import matplotlib.pyplot as plt
import pandas as pd
from datetime import datetime

SERVER_URL = "http://localhost:8080"
ENCODE_ENDPOINT = f"{SERVER_URL}/api/encode"

class CompressionTester:
    def __init__(self, output_dir="analysis_results"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        self.results = []
        
    def compress_file(self, file_path, auto_update=True, manual_bits=None, show_codes=False):
        file_path = Path(file_path)
        
        if not file_path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")
        
        with open(file_path, 'rb') as f:
            files = {'file': (file_path.name, f, 'application/octet-stream')}
            
            params = {
                'auto_update_index': str(auto_update).lower(),
                'show_emitted_codes': str(show_codes).lower()
            }
            
            if not auto_update and manual_bits:
                params['manual_index_bits'] = str(manual_bits)
            
            try:
                response = requests.post(ENCODE_ENDPOINT, files=files, params=params)
                response.raise_for_status()
                return response.json()
            except requests.exceptions.RequestException as e:
                print(f"Error during compression: {e}")
                if hasattr(e.response, 'text'):
                    print(f"Response: {e.response.text}")
                raise
    
    def test_file(self, file_path):
        file_path = Path(file_path)
        print(f"\n{'='*80}")
        print(f"Testare fisier: {file_path.name}")
        print(f"Dimensiune originala: {file_path.stat().st_size:,} bytes")
        print(f"{'='*80}\n")
        
        test_results = []
        
        print("Testare mod Auto-update (dinamic 9→15 biti)...")
        try:
            result = self.compress_file(file_path, auto_update=True)
            test_results.append({
                'fisier': file_path.name,
                'mod': 'Auto-update',
                'biti': '9→15',
                'strategie': 'Empty',
                'dimensiune_originala': result['original_size'],
                'dimensiune_comprimata': result['compressed_size'],
                'dimensiune_antet': result['header_size'],
                'rata_compresie': result['compression_ratio'],
                'spatiu_salvat': result['space_saved'],
                'procent_salvat': result['percentage_saved']
            })
            print(f"  ✓ Comprimat: {result['compressed_size']:,} bytes "
                  f"({result['percentage_saved']:.2f}% salvat)\n")
        except Exception as e:
            print(f"  ✗ Esuat: {e}\n")
        
        for bits in range(9, 16): 
            print(f"Testare mod Manual ({bits} biti, strategie Empty)...")
            try:
                result = self.compress_file(
                    file_path, 
                    auto_update=False, 
                    manual_bits=bits
                )
                test_results.append({
                    'fisier': file_path.name,
                    'mod': 'Manual',
                    'biti': str(bits),
                    'strategie': 'Empty',
                    'dimensiune_originala': result['original_size'],
                    'dimensiune_comprimata': result['compressed_size'],
                    'dimensiune_antet': result['header_size'],
                    'rata_compresie': result['compression_ratio'],
                    'spatiu_salvat': result['space_saved'],
                    'procent_salvat': result['percentage_saved']
                })
                print(f"  ✓ Comprimat: {result['compressed_size']:,} bytes "
                      f"({result['percentage_saved']:.2f}% salvat)\n")
            except Exception as e:
                print(f"  ✗ Esuat: {e}\n")
        
        self.results.extend(test_results)
        return test_results
    
    def generate_graphs(self, file_results, filename):
        df = pd.DataFrame(file_results)
        
        fig, ax = plt.subplots(figsize=(12, 8))
        
        configs = []
        sizes = []
        
        auto_row = df[df['mod'] == 'Auto-update'].iloc[0]
        configs.append('Dynamic\n9→15')
        sizes.append(auto_row['dimensiune_comprimata'])
        
        for bits in range(9, 16):
            manual_row = df[(df['mod'] == 'Manual') & (df['biti'] == str(bits))]
            if not manual_row.empty:
                manual_row = manual_row.iloc[0]
                configs.append(f'Static\n{bits}b')
                sizes.append(manual_row['dimensiune_comprimata'])
        
        colors = ['
        bars = ax.bar(range(len(configs)), sizes, color=colors, alpha=0.7, edgecolor='black', linewidth=1.5)
        
        ax.axhline(y=auto_row['dimensiune_originala'], color='red', linestyle='--', linewidth=2,
                   label=f'Original: {auto_row["dimensiune_originala"]:,} bytes')
        
        ax.set_xlabel('Configuration', fontweight='bold', fontsize=12)
        ax.set_ylabel('Compressed Size (bytes)', fontweight='bold', fontsize=12)
        ax.set_title(f'LZW Compression Analysis: {filename}', fontweight='bold', fontsize=14, pad=15)
        ax.set_xticks(range(len(configs)))
        ax.set_xticklabels(configs, fontsize=10)
        ax.legend(fontsize=11)
        ax.grid(axis='y', alpha=0.3)
        
        for i, (bar, size) in enumerate(zip(bars, sizes)):
            height = bar.get_height()
            pct = df.iloc[i]['procent_salvat']
            ax.text(bar.get_x() + bar.get_width()/2., height,
                    f'{int(height):,}\n({pct:.1f}%)',
                    ha='center', va='bottom', fontsize=8, fontweight='bold')
        
        best_idx = df['procent_salvat'].idxmax()
        best = df.iloc[best_idx]
        bars[best_idx].set_edgecolor('lime')
        bars[best_idx].set_linewidth(4)
        
        summary = (f"BEST: {best['mod']} - {best['biti']} bits | "
                  f"{best['dimensiune_comprimata']:,} bytes | "
                  f"{best['procent_salvat']:.1f}% saved | "
                  f"{best['rata_compresie']:.2f}× compression")
        
        fig.text(0.5, 0.02, summary, ha='center', fontsize=10, fontweight='bold',
                bbox=dict(boxstyle='round', facecolor='lightgreen', alpha=0.7))
        
        plt.tight_layout(rect=[0, 0.05, 1, 1])
        
        pdf_path = self.output_dir / f"{Path(filename).stem}_analysis.pdf"
        plt.savefig(pdf_path, dpi=300, bbox_inches='tight')
        print(f"\n✓ PDF saved: {pdf_path}")
        
        plt.close()
    
    def export_to_excel(self):
        if not self.results:
            print("Nu exista rezultate de exportat!")
            return
        
        df = pd.DataFrame(self.results)
        
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        excel_path = self.output_dir / f"analiza_compresie_{timestamp}.xlsx"
        
        with pd.ExcelWriter(excel_path, engine='openpyxl') as writer:
            df.to_excel(writer, sheet_name='Toate rezultatele', index=False)
            
            for filename in df['fisier'].unique():
                file_df = df[df['fisier'] == filename]
                file_df.to_excel(writer, sheet_name=filename[:31], index=False)
        
        print(f"\n✓ Fisier Excel salvat: {excel_path}")
    
    def print_summary(self):
        if not self.results:
            print("Nu exista rezultate de sumarizat!")
            return
        
        df = pd.DataFrame(self.results)
        
        print("\n" + "="*80)
        print("SUMAR ANALIZA COMPRESIE")
        print("="*80)
        
        for filename in df['fisier'].unique():
            file_df = df[df['fisier'] == filename]
            best_idx = file_df['procent_salvat'].idxmax()
            best = file_df.loc[best_idx]
            
            print(f"\nFisier: {filename}")
            print(f"  Dimensiune originala: {best['dimensiune_originala']:,} bytes")
            print(f"  Cea mai buna configuratie: {best['mod']} - {best['biti']} biti - {best['strategie']}")
            print(f"  Dimensiune comprimata: {best['dimensiune_comprimata']:,} bytes")
            print(f"  Procent salvat: {best['procent_salvat']:.2f}%")
            print(f"  Rata de compresie: {best['rata_compresie']:.2f}")


def main():
    try:
        response = requests.get(SERVER_URL, timeout=2)
        print("✓ Serverul ruleaza")
    except:
        print("✗ EROARE: Serverul nu ruleaza!")
        print(f"  Va rugam porniti serverul mai intai cu: cargo run")
        sys.exit(1)

    if len(sys.argv) > 1:
        file_paths = sys.argv[1:]
    else:
        print("\nIntroduceti caile fisierelor de testat (una pe linie, linie goala pentru finalizare):")
        file_paths = []
        while True:
            path = input("Cale fisier: ").strip()
            if not path:
                break
            file_paths.append(path)
    
    if not file_paths:
        print("Niciun fisier specificat! Iesire.")
        sys.exit(1)
    
    tester = CompressionTester()
    
    for file_path in file_paths:
        try:
            file_results = tester.test_file(file_path)
            tester.generate_graphs(file_results, Path(file_path).name)
        except FileNotFoundError as e:
            print(f"✗ Eroare: {e}")
            continue
        except Exception as e:
            print(f"✗ Eroare neasteptata la testarea {file_path}: {e}")
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
