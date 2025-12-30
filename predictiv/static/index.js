// ==================== STATE ====================
let originalBmpBytes = null;  // Raw uploaded BMP bytes
let currentOriginal = null;
let currentError = null;
let currentEncodedData = null;
let currentHist = { original: [], error: [] };
let currentScale = 1.5;

// ==================== DOM ====================
const imageInput = document.getElementById('imageFileInput');
const originalCtx = document.getElementById('originalCanvas').getContext('2d');
const errorCtx = document.getElementById('errorCanvas').getContext('2d');
const decodedCtx = document.getElementById('decodedCanvas').getContext('2d');
const histCtx = document.getElementById('histogramCanvas').getContext('2d');

const scaleSlider = document.getElementById('errorScale');
const scaleValue = document.getElementById('scaleValue');
const statusEl = document.getElementById('status');

const loadBtn = document.getElementById('loadImageBtn');
const predictBtn = document.getElementById('predictBtn');
const storeBtn = document.getElementById('storeBtn');
const showErrorBtn = document.getElementById('showErrorBtn');
const showHistBtn = document.getElementById('showHistogramBtn');

// ==================== LOAD IMAGE (only store bytes, show nothing yet) ====================
loadBtn.addEventListener('click', () => {
  imageInput.click();
});

imageInput.addEventListener('change', async (e) => {
  const file = e.target.files[0];
  if (!file) return;

  statusEl.textContent = `Loaded: ${file.name}. Select predictor and click Predict.`;

  const arrayBuffer = await file.arrayBuffer();
  originalBmpBytes = new Uint8Array(arrayBuffer);

  // Clear canvases
  originalCtx.clearRect(0, 0, 256, 256);
  errorCtx.clearRect(0, 0, 256, 256);
  histCtx.clearRect(0, 0, 512, 200);

  // Enable Predict, disable others
  predictBtn.disabled = false;
  storeBtn.disabled = true;
});

// ==================== PREDICT (Main action) ====================
predictBtn.addEventListener('click', async () => {
  if (!originalBmpBytes) {
    statusEl.textContent = 'Please load an image first';
    return;
  }

  const selected = document.querySelector('input[name="predictor"]:checked');
  if (!selected) {
    statusEl.textContent = 'Please select a predictor';
    return;
  }

  const predictionNumber = parseInt(selected.value, 10);

  statusEl.textContent = 'Predicting...';

  try {
    const response = await fetch('/api/encode', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        file_name: 'image.bmp',
        file_data: Array.from(originalBmpBytes),
        prediction_number: predictionNumber
      })
    });

    if (!response.ok) {
      throw new Error(`Server error: ${response.status}`);
    }

    const data = await response.json();

    // Store results
    currentOriginal = data.original_image;
    currentError = data.error_matrix;
    currentEncodedData = data.encoded_data || null;
    currentHist.original = data.original_histogram || [];
    currentHist.error = data.error_histogram || [];

    // Display
    drawGrayscale(originalCtx, currentOriginal);
    drawError(errorCtx, currentError, currentScale);
    drawHistogram(histCtx, currentHist.original); // default to original

    // Enable buttons
    storeBtn.disabled = false;
    showErrorBtn.disabled = false;
    showHistBtn.disabled = false;

    statusEl.textContent = `Prediction complete (predictor ${predictionNumber})`;

  } catch (err) {
    statusEl.textContent = `Error: ${err.message}`;
    console.error(err);
  }
});

// ==================== STORE ====================
storeBtn.addEventListener('click', () => {
  if (!currentEncodedData) {
    statusEl.textContent = 'No encoded data (predict first)';
    return;
  }

  const blob = new Blob([new Uint8Array(currentEncodedData)], { type: 'application/octet-stream' });
  const url = URL.createObjectURL(blob);

  const a = document.createElement('a');
  a.href = url;
  const predictor = document.querySelector('input[name="predictor"]:checked').value;
  a.download = `image.bmp[${predictor}].pre`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);

  statusEl.textContent = 'Downloaded .pre file!';
});

// ==================== SHOW ERROR MATRIX ====================
showErrorBtn.addEventListener('click', () => {
  if (!currentError) {
    statusEl.textContent = 'Predict first';
    return;
  }
  drawError(errorCtx, currentError, currentScale);
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

  if (source === 'original') hist = currentHist.original;
  else if (source === 'error') hist = currentHist.error;

  if (hist.length === 0) {
    statusEl.textContent = 'No histogram data available';
    return;
  }

  drawHistogram(histCtx, hist);
});

// ==================== DRAW FUNCTIONS ====================
function drawGrayscale(ctx, matrix) {
  const imgData = ctx.createImageData(256, 256);
  const d = imgData.data;
  let i = 0;
  for (let y = 0; y < 256; y++) {
    for (let x = 0; x < 256; x++) {
      const v = Math.max(0, Math.min(255, matrix[y][x]));
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
  ctx.fillStyle = '#3498db';
  for (let i = 0; i < 256; i++) {
    const h = (hist[i] / max) * 200;
    ctx.fillRect(i * 2, 200 - h, 2, h);
  }
}

// Initial scale
scaleValue.textContent = currentScale.toFixed(1);
