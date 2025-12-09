#!/bin/bash

echo "======================================="
echo "Analiza Compresie LZW"
echo "======================================="
echo ""
echo "Acest script va testa compresie LZW cu toate combinatiile valide de:"
echo "  - Mod Auto-update: 9→15 biti (dinamic)"
echo "  - Mod Manual: 9-15 biti (static) cu strategie Empty"
echo ""
echo "Total combinatii per fisier: 8 (1 dynamic + 7 static)"
echo ""
echo "Puteti specifica fisiere de testat in trei moduri:"
echo "  1. Treceti fisiere ca argumente: ./run_analysis.sh fisier1.txt fisier2.csv fisier3.png"
echo "  2. Rulati fara argumente si introduceti fisierele interactiv"
echo "  3. Editati acest script pentru a folosi fisiere implicite"
echo ""

if ! curl -s http://localhost:8080 > /dev/null 2>&1; then
    echo "Serverul nu ruleaza!"
    echo ""
    echo "Va rugam porniti serverul intr-un alt terminal cu:"
    echo "  cargo run"
    echo ""
    read -p "Apasati Enter cand serverul ruleaza, sau Ctrl+C pentru iesire..."
fi

echo "Verificare Python dependencies..."
if ! python3 -c "import requests, matplotlib, pandas, openpyxl, seaborn, numpy" 2>/dev/null; then
    echo ""
    echo "Lipsesc pachete Python necesare. Se instaleaza..."
    pip3 install requests matplotlib pandas openpyxl seaborn numpy
    echo ""
fi

if [ $# -eq 0 ]; then
    echo "Niciun fisier specificat ca argument."
    echo "Rulare in mod interactiv..."
    echo ""
    python3 compression_analysis.py
else
    echo "Testare fisiere: $@"
    echo ""
    python3 compression_analysis.py "$@"
fi

echo ""
echo "Rezultate salvate in directorul 'analysis_results/':"
echo "  - Grafice PDF pentru fiecare fisier (heatmap)"
echo "  - Fisier Excel cu toate datele"
echo ""
echo "Pentru a vizualiza graficele PDF:"
echo "  xdg-open analysis_results/*.pdf"
echo ""
