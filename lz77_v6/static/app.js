let encodeFileData = null;
let decodeFileData = null;
let encodedData = null;
let decodedData = null;
let lastTokens = null;

const encodeFileInput = document.getElementById("encode-file");
const encodeFileInfo = document.getElementById("encode-file-info");
const offsetBitsInput = document.getElementById("offset-bits");
const lengthBitsInput = document.getElementById("length-bits");
const displayTokensCheckbox = document.getElementById("display-tokens");
const encodeBtn = document.getElementById("encode-btn");
const encodeResult = document.getElementById("encode-result");
const downloadEncodedBtn = document.getElementById("download-encoded");
const tokensDisplay = document.getElementById("tokens-display");
const tokensList = document.getElementById("tokens-list");

const decodeFileInput = document.getElementById("decode-file");
const decodeFileInfo = document.getElementById("decode-file-info");
const decodeBtn = document.getElementById("decode-btn");
const decodeResult = document.getElementById("decode-result");
const downloadDecodedBtn = document.getElementById("download-decoded");

const loading = document.getElementById("loading");
const errorDiv = document.getElementById("error");

encodeFileInput.addEventListener("change", handleEncodeFileSelect);
encodeBtn.addEventListener("click", handleEncode);
downloadEncodedBtn.addEventListener("click", handleDownloadEncoded);

decodeFileInput.addEventListener("change", handleDecodeFileSelect);
decodeBtn.addEventListener("click", handleDecode);
downloadDecodedBtn.addEventListener("click", handleDownloadDecoded);

displayTokensCheckbox.addEventListener("change", handleDisplayTokensToggle);

function handleEncodeFileSelect(event) {
  const file = event.target.files[0];
  if (file) {
    encodeFileInfo.textContent = `Selected: ${file.name} (${formatBytes(
      file.size
    )})`;

    const reader = new FileReader();
    reader.onload = (e) => {
      encodeFileData = {
        filename: file.name,
        data: new Uint8Array(e.target.result),
      };
      encodeBtn.disabled = false;
    };
    reader.readAsArrayBuffer(file);
  }
}

function handleDecodeFileSelect(event) {
  const file = event.target.files[0];
  if (file) {
    if (!file.name.endsWith(".lz77")) {
      showError("Please select a file with .lz77 extension");
      return;
    }

    decodeFileInfo.textContent = `Selected: ${file.name} (${formatBytes(
      file.size
    )})`;

    const reader = new FileReader();
    reader.onload = (e) => {
      decodeFileData = {
        filename: file.name,
        data: new Uint8Array(e.target.result),
      };
      decodeBtn.disabled = false;
    };
    reader.readAsArrayBuffer(file);
  }
}

async function handleEncode() {
  if (!encodeFileData) return;

  const offsetBits = parseInt(offsetBitsInput.value);
  const lengthBits = parseInt(lengthBitsInput.value);

  if (offsetBits < 2 || offsetBits > 15) {
    showError("Offset bits must be between 2 and 15");
    return;
  }

  if (lengthBits < 2 || lengthBits > 7) {
    showError("Length bits must be between 2 and 7");
    return;
  }

  showLoading();

  try {
    const response = await fetch("/api/encode", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        filename: encodeFileData.filename,
        file_data: Array.from(encodeFileData.data),
        offset_bits: offsetBits,
        length_bits: lengthBits,
        display_tokens: displayTokensCheckbox.checked,
      }),
    });

    if (!response.ok) {
      throw new Error("Encoding failed");
    }

    const result = await response.json();
    displayEncodeResult(result);
    encodedData = {
      filename: result.encoded_filename,
      data: new Uint8Array(result.encoded_data),
    };
  } catch (error) {
    showError("Error during encoding: " + error.message);
  } finally {
    hideLoading();
  }
}

async function handleDecode() {
  if (!decodeFileData) return;

  showLoading();

  try {
    const response = await fetch("/api/decode", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        filename: decodeFileData.filename,
        file_data: Array.from(decodeFileData.data),
      }),
    });

    if (!response.ok) {
      throw new Error("Decoding failed");
    }

    const result = await response.json();
    displayDecodeResult(result);
    decodedData = {
      filename: result.decoded_filename,
      data: new Uint8Array(result.decoded_data),
    };
  } catch (error) {
    showError("Error during decoding: " + error.message);
  } finally {
    hideLoading();
  }
}

function displayEncodeResult(result) {
  document.getElementById("encoded-filename").textContent =
    result.encoded_filename;
  document.getElementById("original-size").textContent = formatBytes(
    result.original_size
  );
  document.getElementById("compressed-size").textContent = formatBytes(
    result.compressed_size
  );
  document.getElementById("compression-ratio").textContent =
    result.compression_ratio.toFixed(2);

  lastTokens = result.tokens && result.tokens.length > 0 ? result.tokens : null;
  updateTokensDisplay();

  encodeResult.classList.remove("hidden");
}

function updateTokensDisplay() {
  if (lastTokens && displayTokensCheckbox.checked) {
    tokensList.innerHTML = "";
    lastTokens.forEach((token, index) => {
      const tokenItem = document.createElement("div");
      tokenItem.className = "token-item";
      tokenItem.textContent = `Token ${index + 1}: (offset=${
        token.offset
      }, length=${token.match_length}, char=${token.next_char})`;
      tokensList.appendChild(tokenItem);
    });
    tokensDisplay.classList.remove("hidden");
  } else {
    tokensDisplay.classList.add("hidden");
  }
}

function handleDisplayTokensToggle() {
  updateTokensDisplay();
}

function displayDecodeResult(result) {
  document.getElementById("decoded-filename").textContent =
    result.decoded_filename;
  document.getElementById("original-compressed-size").textContent = formatBytes(
    result.original_compressed_size
  );
  document.getElementById("decompressed-size").textContent = formatBytes(
    result.decompressed_size
  );

  decodeResult.classList.remove("hidden");
}

function handleDownloadEncoded() {
  if (!encodedData) return;

  const blob = new Blob([encodedData.data], {
    type: "application/octet-stream",
  });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = encodedData.filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

function handleDownloadDecoded() {
  if (!decodedData) return;

  const blob = new Blob([decodedData.data], {
    type: "application/octet-stream",
  });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = decodedData.filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

function formatBytes(bytes) {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
}

function showLoading() {
  loading.classList.remove("hidden");
}

function hideLoading() {
  loading.classList.add("hidden");
}

function showError(message) {
  errorDiv.textContent = message;
  errorDiv.classList.remove("hidden");
  setTimeout(() => {
    errorDiv.classList.add("hidden");
  }, 5000);
}
