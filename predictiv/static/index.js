// ==================== STATE ====================
let originalBmpBytes = null;
let currentOriginal = null;
let currentError = null;
let currentEncodedData = null;
let currentDecoded = null;
let currentDecodedBmp = null;
let currentDecodedFilename = null;
let currentHist = { original: [], error: [], decoded: [] };
let currentScale = 1.5;

// ==================== DOM ====================
const imageInput = document.getElementById('imageFileInput');
const encodedInput = document.getElementById('encodedFileInput');

const originalCtx = document.getElementById('originalCanvas').getContext('2d');
const errorCtx = document.getElementById('errorCanvas').getContext('2d');
const decodedCtx = document.getElementById('decodedCanvas').getContext('2d');
const histCtx = document.getElementById('histogramCanvas').getContext('2d');

const scaleSlider = document.getElementById('errorScale');
const scaleValue = document.getElementById('scaleValue');
const statusEl = document.getElementById('status');

// Buttons
const loadBtn = document.getElementById('loadImageBtn');
const predictBtn = document.getElementById('predictBtn');
const storeBtn = document.getElementById('storeBtn');
const showErrorBtn = document.getElementById('showErrorBtn');
const showHistBtn = document.getElementById('showHistogramBtn');
const loadEncodedBtn = document.getElementById('loadEncodedBtn');
const saveDecodedBtn = document.getElementById('saveDecodedBtn');

// Initial button states
predictBtn.disabled = true;
storeBtn.disabled = true;
showErrorBtn.disabled = true;
showHistBtn.disabled = true;
saveDecodedBtn.disabled = true;

// ==================== LOAD IMAGE ====================
loadBtn.addEventListener('click', () => imageInput.click());

imageInput.addEventListener('change', async (e) => {
  const file = e.target.files[0];
  if (!file) return;

  statusEl.textContent = `Loaded: ${file.name}. Select predictor and click Predict.`;

  const buffer = await file.arrayBuffer();
  originalBmpBytes = new Uint8Array(buffer);

  // Clear all panels
  originalCtx.clearRect(0, 0, 256, 256);
  errorCtx.clearRect(0, 0, 256, 256);
  decodedCtx.clearRect(0, 0, 256, 256);
  histCtx.clearRect(0, 0, 512, 200);

  predictBtn.disabled = false;
  storeBtn.disabled = true;
  showErrorBtn.disabled = true;
  showHistBtn.disabled = true;
  saveDecodedBtn.disabled = true;
});

// ==================== PREDICT ====================
predictBtn.addEventListener('click', async () => {
  if (!originalBmpBytes) {
    statusEl.textContent = 'Load an image first';
    return;
  }

  const selected = document.querySelector('input[name="predictor"]:checked');
  if (!selected) {
    statusEl.textContent = 'Select a predictor';
    return;
  }

  const predictionNumber = parseInt(selected.value, 10);
  statusEl.textContent = 'Predicting...';

  try {
    const res = await fetch('/api/encode', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        file_name: 'temp.bmp',
        file_data: Array.from(originalBmpBytes),
        prediction_number: predictionNumber
      })
    });

    if (!res.ok) throw new Error('Encode failed');

    const data = await res.json();

    currentOriginal = data.original_image;
    currentError = data.error_matrix;
    currentEncodedData = data.encoded_data;
    currentHist.original = data.original_histogram || [];
    currentHist.error = data.error_histogram || [];

    drawGrayscale(originalCtx, currentOriginal);
    drawError(errorCtx, currentError, currentScale);
    drawHistogram(histCtx, currentHist.original);

    storeBtn.disabled = false;
    showErrorBtn.disabled = false;
    showHistBtn.disabled = false;

    statusEl.textContent = `Prediction complete (predictor ${predictionNumber})`;
  } catch (err) {
    statusEl.textContent = `Error: ${err.message}`;
  }
});

// ==================== STORE ====================
storeBtn.addEventListener('click', () => {
  if (!currentEncodedData) {
    statusEl.textContent = 'Predict first';
    return;
  }

  const blob = new Blob([new Uint8Array(currentEncodedData)], { type: 'application/octet-stream' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  const pred = document.querySelector('input[name="predictor"]:checked').value;
  a.download = `image.bmp[${pred}].pre`;
  a.click();
  URL.revokeObjectURL(url);

  statusEl.textContent = 'Downloaded .pre file!';
});

// ==================== LOAD ENCODED (Auto Decode) ====================
loadEncodedBtn.addEventListener('click', () => encodedInput.click());

encodedInput.addEventListener('change', async (e) => {
  const file = e.target.files[0];
  if (!file) return;

  statusEl.textContent = 'Decoding .pre file...';

  const buffer = await file.arrayBuffer();
  const fileData = new Uint8Array(buffer);

  try {
    const res = await fetch('/api/decode', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        file_name: file.name,
        file_data: Array.from(fileData)
      })
    });

    if (!res.ok) throw new Error('Decode failed');

    const data = await res.json();

    currentDecoded = data.decoded_image;
    currentDecodedBmp = data.decoded_bmp_data;
    currentDecodedFilename = data.decoded_filename;
    currentHist.decoded = data.decoded_histogram || [];

    // DRAW THE DECODED IMAGE
    decodedCtx.clearRect(0, 0, 256, 256);
    drawGrayscale(decodedCtx, currentDecoded);

    // Show decoded histogram
    drawHistogram(histCtx, currentHist.decoded);

    saveDecodedBtn.disabled = false;

    statusEl.textContent = `Decoded successfully with predictor ${data.prediction_type ?? 'unknown'}`;
  } catch (err) {
    statusEl.textContent = `Error: ${err.message}`;
  }
});

// ==================== SAVE DECODED ====================
saveDecodedBtn.addEventListener('click', () => {
  if (!currentDecodedBmp) {
    statusEl.textContent = 'No decoded image to save';
    return;
  }

  const blob = new Blob([new Uint8Array(currentDecodedBmp)], { type: 'image/bmp' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = currentDecodedFilename || 'decoded.bmp';
  a.click();
  URL.revokeObjectURL(url);

  statusEl.textContent = 'Saved decoded BMP!';
});

// ==================== SHOW ERROR MATRIX ====================
showErrorBtn.addEventListener('click', () => {
  if (currentError) {
    drawError(errorCtx, currentError, currentScale);
  }
});

// ==================== SCALE SLIDER ====================
scaleSlider.addEventListener('input', (e) => {
  currentScale = parseFloat(e.target.value);
  scaleValue.textContent = currentScale.toFixed(1);
  if (currentError) {
    drawError(errorCtx, currentError, currentScale);
  }
});

// ==================== SHOW HISTOGRAM ====================
showHistBtn.addEventListener('click', () => {
  const source = document.querySelector('input[name="histSource"]:checked').value;
  let hist = [];

  if (source === 'original' && currentHist.original.length > 0) hist = currentHist.original;
  else if (source === 'error' && currentHist.error.length > 0) hist = currentHist.error;
  else if (source === 'decoded' && currentHist.decoded.length > 0) hist = currentHist.decoded;

  if (hist.length > 0) {
    drawHistogram(histCtx, hist);
  } else {
    statusEl.textContent = 'No histogram data for selected source';
  }
});

// ==================== DRAWING FUNCTIONS ====================
function drawGrayscale(ctx, matrix) {
  const imgData = ctx.createImageData(256, 256);
  const d = imgData.data;
  let i = 0;
  for (let y = 0; y < 256; y++) {
    for (let x = 0; x < 256; x++) {
      let v = matrix[y][x];
      v = Math.max(0, Math.min(255, v));
      d[i++] = v; d[i++] = v; d[i++] = v; d[i++] = 255;
    }
  }
  ctx.putImageData(imgData, 0, 0);
}

function drawError(ctx, matrix, scale) {
  const imgData = ctx.createImageData(256, 256);
  const d = imgData.data;
  let i = 0;
  for (let y = 0; y < 256; y++) {
    for (let x = 0; x < 256; x++) {
      let v = matrix[y][x] * scale + 128;
      v = Math.max(0, Math.min(255, v));
      d[i++] = v; d[i++] = v; d[i++] = v; d[i++] = 255;
    }
  }
  ctx.putImageData(imgData, 0, 0);
}

function drawHistogram(ctx, hist) {
  ctx.clearRect(0, 0, 512, 200);
  if (hist.length === 0) return;

  const max = Math.max(...hist);
  if (max === 0) return;

  ctx.fillStyle = '#3498db';
  for (let i = 0; i < 256; i++) {
    const h = (hist[i] / max) * 200;
    ctx.fillRect(i * 2, 200 - h, 2, h);
  }
}

// Initial scale display
scaleValue.textContent = currentScale.toFixed(1);
