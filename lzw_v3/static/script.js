function formatBytes(bytes) {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

function downloadFile(data, filename) {
  const bytes = base64ToArrayBuffer(data);
  const blob = new Blob([bytes], { type: "application/octet-stream" });
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  window.URL.revokeObjectURL(url);
  document.body.removeChild(a);
}

function base64ToArrayBuffer(base64) {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}


const encodeFileInput = document.getElementById("encodeFile");
const encodeFileNameDiv = document.getElementById("encodeFileName");
const encodeBtn = document.getElementById("encodeBtn");
const autoUpdateRadio = document.getElementById("autoUpdate");
const manualUpdateRadio = document.getElementById("manualUpdate");
const manualOptions = document.getElementById("manualOptions");
const indexBitsSelect = document.getElementById("indexBits");
const freezeModeRadio = document.getElementById("freezeMode");
const emptyModeRadio = document.getElementById("emptyMode");
const showEmittedCodesCheckbox = document.getElementById("showEmittedCodes");
const encodeLoading = document.getElementById("encodeLoading");
const encodeStats = document.getElementById("encodeStats");
const encodeCodesOutput = document.getElementById("encodeCodesOutput");
const encodeError = document.getElementById("encodeError");

autoUpdateRadio.addEventListener("change", () => {
  if (autoUpdateRadio.checked) {
    manualOptions.classList.add("disabled-section");
  }
});

manualUpdateRadio.addEventListener("change", () => {
  if (manualUpdateRadio.checked) {
    manualOptions.classList.remove("disabled-section");
  }
});

encodeFileInput.addEventListener("change", (e) => {
  const file = e.target.files[0];
  if (file) {
    encodeFileNameDiv.textContent = `📄 ${file.name} (${formatBytes(
      file.size
    )})`;
    encodeBtn.disabled = false;
  } else {
    encodeFileNameDiv.textContent = "No file selected";
    encodeBtn.disabled = true;
  }
});

encodeBtn.addEventListener("click", async () => {
  const file = encodeFileInput.files[0];
  if (!file) return;

  encodeBtn.disabled = true;
  encodeLoading.classList.add("show");
  encodeStats.classList.remove("show");
  encodeCodesOutput.classList.remove("show");
  encodeError.classList.remove("show");

  try {
    const formData = new FormData();
    formData.append("file", file);

    const autoUpdate = autoUpdateRadio.checked;
    const params = new URLSearchParams({
      auto_update_index: autoUpdate,
      show_emitted_codes: showEmittedCodesCheckbox.checked,
    });

    if (!autoUpdate) {
      params.append("manual_index_bits", indexBitsSelect.value);
    }

    const response = await fetch(`/api/encode?${params.toString()}`, {
      method: "POST",
      body: formData,
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(errorData.error || "Encoding failed");
    }

    const result = await response.json();

    document.getElementById("encodeOriginalSize").textContent = formatBytes(
      result.original_size
    );
    document.getElementById("encodeCompressedSize").textContent = formatBytes(
      result.compressed_size
    );
    document.getElementById(
      "encodeHeaderSize"
    ).textContent = `${result.header_size} bits`;
    document.getElementById("encodeCompressionRatio").textContent =
      result.compression_ratio.toFixed(2);
    document.getElementById("encodeSpaceSaved").textContent = formatBytes(
      result.space_saved
    );
    document.getElementById(
      "encodePercentageSaved"
    ).textContent = `${result.percentage_saved.toFixed(2)}%`;
    document.getElementById("encodeOutputFile").textContent = result.filename;

    encodeStats.classList.add("show");

    if (result.codes && result.codes.length > 0) {
      encodeCodesOutput.innerHTML = `<strong>Emitted Codes (${result.codes.length} total):</strong><br>`;
      const codesToShow = result.codes.slice(0, 500); 
      encodeCodesOutput.innerHTML += codesToShow.join(", ");
      if (result.codes.length > 500) {
        encodeCodesOutput.innerHTML += `<br><em>... and ${
          result.codes.length - 500
        } more codes</em>`;
      }
      encodeCodesOutput.classList.add("show");
    }

    downloadFile(result.file_data, result.filename);
  } catch (error) {
    console.error("Encoding error:", error);
    encodeError.textContent = `Error: ${error.message}`;
    encodeError.classList.add("show");
  } finally {
    encodeLoading.classList.remove("show");
    encodeBtn.disabled = false;
  }
});


const decodeFileInput = document.getElementById("decodeFile");
const decodeFileNameDiv = document.getElementById("decodeFileName");
const decodeBtn = document.getElementById("decodeBtn");
const showDecodedCodesCheckbox = document.getElementById("showDecodedCodes");
const decodeLoading = document.getElementById("decodeLoading");
const decodeStats = document.getElementById("decodeStats");
const decodeCodesOutput = document.getElementById("decodeCodesOutput");
const decodeError = document.getElementById("decodeError");

decodeFileInput.addEventListener("change", (e) => {
  const file = e.target.files[0];
  if (file) {
    if (!file.name.toLowerCase().endsWith(".lzw")) {
      decodeError.textContent = "Error: File must have .LZW extension";
      decodeError.classList.add("show");
      decodeFileNameDiv.textContent = "No file selected";
      decodeBtn.disabled = true;
      return;
    }

    decodeError.classList.remove("show");
    decodeFileNameDiv.textContent = `📄 ${file.name} (${formatBytes(
      file.size
    )})`;
    decodeBtn.disabled = false;
  } else {
    decodeFileNameDiv.textContent = "No file selected";
    decodeBtn.disabled = true;
  }
});

decodeBtn.addEventListener("click", async () => {
  const file = decodeFileInput.files[0];
  if (!file) return;

  decodeBtn.disabled = true;
  decodeLoading.classList.add("show");
  decodeStats.classList.remove("show");
  decodeCodesOutput.classList.remove("show");
  decodeError.classList.remove("show");

  try {
    const formData = new FormData();
    formData.append("file", file);

    const params = new URLSearchParams({
      show_codes: showDecodedCodesCheckbox.checked,
    });

    const response = await fetch(`/api/decode?${params.toString()}`, {
      method: "POST",
      body: formData,
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(errorData.error || "Decoding failed");
    }

    const result = await response.json();

    document.getElementById("decodeCompressedSize").textContent = formatBytes(
      result.original_size
    );
    document.getElementById("decodeDecompressedSize").textContent = formatBytes(
      result.decompressed_size
    );
    document.getElementById("decodeOutputFile").textContent = result.filename;

    decodeStats.classList.add("show");

    if (result.codes && result.codes.length > 0) {
      decodeCodesOutput.innerHTML = `<strong>Decoded Codes (${result.codes.length} total):</strong><br>`;
      const codesToShow = result.codes.slice(0, 100); 
      codesToShow.forEach((codeInfo) => {
        decodeCodesOutput.innerHTML += `[${codeInfo[0]}]: ${codeInfo[1]}<br>`;
      });
      if (result.codes.length > 100) {
        decodeCodesOutput.innerHTML += `<em>... and ${
          result.codes.length - 100
        } more codes</em>`;
      }
      decodeCodesOutput.classList.add("show");
    }

    downloadFile(result.file_data, result.filename);
  } catch (error) {
    console.error("Decoding error:", error);
    decodeError.textContent = `Error: ${error.message}`;
    decodeError.classList.add("show");
  } finally {
    decodeLoading.classList.remove("show");
    decodeBtn.disabled = false;
  }
});
