let encodeFile = null;
let encodeFileData = null;
let encodeFileName = "";

const encodeFileInput = document.getElementById("encode-file-input");
const encodeFileNameDisplay = document.getElementById("encode-file-name");
const encodeButton = document.getElementById("encode-button");
const encodeShowCodes = document.getElementById("encode-show-codes");
const encodeTwoBytes = document.getElementById("encode-two-bytes");
const encodeResults = document.getElementById("encode-results");
const encodeCodesContainer = document.getElementById("encode-codes-container");

encodeFileInput.addEventListener("change", (e) => {
  const file = e.target.files[0];
  if (file) {
    encodeFile = file;
    encodeFileName = file.name;
    encodeFileNameDisplay.textContent = file.name;
    encodeButton.disabled = false;

    encodeResults.style.display = "none";
    encodeCodesContainer.style.display = "none";
  }
});

encodeButton.addEventListener("click", async () => {
  if (!encodeFile) return;

  showLoading(true);

  const formData = new FormData();
  formData.append("file", encodeFile);

  const showCodes = encodeShowCodes.checked;
  const twoBytes = encodeTwoBytes.checked;

  try {
    const response = await fetch(
      `/api/encode?show_codes=${showCodes}&two_bytes=${twoBytes}`,
      {
        method: "POST",
        body: formData,
      }
    );

    const result = await response.json();

    if (result.success) {
      document.getElementById("encode-original-size").textContent = formatBytes(
        result.original_size
      );
      document.getElementById("encode-compressed-size").textContent =
        formatBytes(result.compressed_size);
      document.getElementById("encode-header-size").textContent = formatBytes(
        result.header_size
      );
      document.getElementById("encode-data-size").textContent = formatBytes(
        result.compressed_data_size
      );
      document.getElementById("encode-ratio").textContent =
        result.compression_ratio.toFixed(2) + "%";
      document.getElementById("encode-saved").textContent = `${formatBytes(
        result.space_saved
      )} (${result.percentage_saved.toFixed(2)}%)`;

      encodeResults.style.display = "block";

      encodeFileData = base64ToBytes(result.file_data);
      encodeFileName = result.filename;

      if (showCodes && result.codes) {
        displayCodes("encode-codes", result.codes);
        encodeCodesContainer.style.display = "block";
      } else {
        encodeCodesContainer.style.display = "none";
      }
    } else {
      alert("Error: " + (result.error || result.message));
    }
  } catch (error) {
    alert("Error encoding file: " + error.message);
    console.error(error);
  } finally {
    showLoading(false);
  }
});

document.getElementById("download-encoded").addEventListener("click", () => {
  if (encodeFileData) {
    downloadFile(encodeFileData, encodeFileName);
  }
});

let decodeFile = null;
let decodeFileData = null;
let decodeFileName = "";

const decodeFileInput = document.getElementById("decode-file-input");
const decodeFileNameDisplay = document.getElementById("decode-file-name");
const decodeButton = document.getElementById("decode-button");
const decodeShowCodes = document.getElementById("decode-show-codes");
const decodeResults = document.getElementById("decode-results");
const decodeCodesContainer = document.getElementById("decode-codes-container");

decodeFileInput.addEventListener("change", (e) => {
  const file = e.target.files[0];
  if (file) {
    decodeFile = file;
    decodeFileNameDisplay.textContent = file.name;
    decodeButton.disabled = false;

    decodeResults.style.display = "none";
    decodeCodesContainer.style.display = "none";
  }
});

decodeButton.addEventListener("click", async () => {
  if (!decodeFile) return;

  showLoading(true);

  const formData = new FormData();
  formData.append("file", decodeFile);

  const showCodes = decodeShowCodes.checked;

  try {
    const response = await fetch(`/api/decode?show_codes=${showCodes}`, {
      method: "POST",
      body: formData,
    });

    const result = await response.json();

    if (result.success) {
      document.getElementById("decode-original-size").textContent = formatBytes(
        result.original_size
      );
      document.getElementById("decode-decompressed-size").textContent =
        formatBytes(result.decompressed_size);

      decodeResults.style.display = "block";

      decodeFileData = base64ToBytes(result.file_data);
      decodeFileName = result.filename;

      if (showCodes && result.codes) {
        displayCodes("decode-codes", result.codes);
        decodeCodesContainer.style.display = "block";
      } else {
        decodeCodesContainer.style.display = "none";
      }
    } else {
      alert("Error: " + (result.error || result.message));
    }
  } catch (error) {
    alert("Error decoding file: " + error.message);
    console.error(error);
  } finally {
    showLoading(false);
  }
});

document.getElementById("download-decoded").addEventListener("click", () => {
  if (decodeFileData) {
    downloadFile(decodeFileData, decodeFileName);
  }
});

function showLoading(show) {
  document.getElementById("loading-overlay").style.display = show
    ? "flex"
    : "none";
}

function formatBytes(bytes) {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
}

function base64ToBytes(base64) {
  const binaryString = atob(base64);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes;
}

function downloadFile(data, filename) {
  const blob = new Blob([data]);
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

function displayCodes(elementId, codes) {
  const container = document.getElementById(elementId);
  container.innerHTML = "";

  codes.forEach(([symbol, code]) => {
    const entry = document.createElement("div");
    entry.className = "code-entry";

    const symbolSpan = document.createElement("span");
    symbolSpan.className = "code-symbol";
    symbolSpan.textContent = symbol;

    const codeSpan = document.createElement("span");
    codeSpan.className = "code-value";
    codeSpan.textContent = code;

    entry.appendChild(symbolSpan);
    entry.appendChild(codeSpan);
    container.appendChild(entry);
  });
}
