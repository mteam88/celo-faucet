interface FaucetResponse {
  txHash: string;
}
interface ErrorResponse {
  error: string;
}

// ----- Interactive background using Mark.svg rendered to an offscreen canvas -----
const backgroundCanvas = document.createElement("canvas");
backgroundCanvas.className = "bg-canvas";
document.body.appendChild(backgroundCanvas);
const ctx = backgroundCanvas.getContext("2d", { alpha: true });

let dpi = window.devicePixelRatio || 1;
let width = 0;
let height = 0;
let t = 0;
let mouseX = 0;
let mouseY = 0;

const markImg = new Image();
markImg.src = "/assets/Mark.svg"; // corrected asset path

function resize() {
  width = window.innerWidth;
  height = window.innerHeight;
  backgroundCanvas.style.width = `${width}px`;
  backgroundCanvas.style.height = `${height}px`;
  backgroundCanvas.width = Math.floor(width * dpi);
  backgroundCanvas.height = Math.floor(height * dpi);
  ctx?.setTransform(dpi, 0, 0, dpi, 0, 0);
}

window.addEventListener("resize", resize);
window.addEventListener("pointermove", (e) => {
  mouseX = e.clientX;
  mouseY = e.clientY;
});

function draw() {
  if (!ctx) return;
  t += 0.006;
  ctx.clearRect(0, 0, width, height);

  // Parallax background gradient
  const gx = Math.sin(t) * 80 + (mouseX - width / 2) * 0.03;
  const gy = Math.cos(t * 0.7) * 80 + (mouseY - height / 2) * 0.03;
  const grad = ctx.createLinearGradient(0, 0, width, height);
  grad.addColorStop(0, "#050607");
  grad.addColorStop(1, "#0f1216");
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, width, height);

  if (markImg.complete) {
    const tile = 220;
    const offset = (t * 120) % tile;
    // cover the rotated corners fully to avoid edge pop-in/out
    const pad = Math.ceil(Math.hypot(width, height));

    ctx.save();
    // rotate canvas so the pattern scrolls top-left to bottom-right
    const angle = -Math.PI / 4;
    ctx.translate(width / 2, height / 2);
    ctx.rotate(angle);
    ctx.translate(-width / 2, -height / 2);

    const parallaxX = (mouseX - width / 2) * 0.02;
    const parallaxY = (mouseY - height / 2) * 0.02;
    const startX = -pad - tile - offset + parallaxX;
    const startY = -pad - tile - offset + parallaxY;
    for (let x = startX; x < width + pad + tile; x += tile) {
      for (let y = startY; y < height + pad + tile; y += tile) {
        // subtle scale pulse
        const pulse = 1 + Math.sin((x + y) * 0.01 + t * 4) * 0.04;
        const size = 160 * pulse;
        ctx.globalAlpha = 0.1 + (Math.sin(x * y * 0.00004 + t * 2) + 1) * 0.06;
        ctx.drawImage(markImg, x, y, size, size);
      }
    }
    ctx.restore();

    // Faint highlight bloom in the center
    const r = Math.max(width, height) * 0.6;
    const radial = ctx.createRadialGradient(
      width / 2,
      height / 2,
      0,
      width / 2,
      height / 2,
      r
    );
    radial.addColorStop(0, "rgba(57,195,255,0.10)");
    radial.addColorStop(1, "rgba(57,195,255,0)");
    ctx.fillStyle = radial;
    ctx.fillRect(0, 0, width, height);
  }

  requestAnimationFrame(draw);
}

resize();
draw();

// ----- Faucet form wiring -----
const form = document.getElementById("faucetForm") as HTMLFormElement;
const addressInput = document.getElementById("address") as HTMLInputElement;
const submitBtn = document.getElementById("submitBtn") as HTMLButtonElement;
const messageDiv = document.getElementById("message") as HTMLDivElement;

form.addEventListener("submit", async (e) => {
  e.preventDefault();
  const address = addressInput.value.trim();
  if (!address.match(/^0x[a-fA-F0-9]{40}$/)) {
    showError("Please enter a valid Ethereum address");
    return;
  }
  submitBtn.disabled = true;
  submitBtn.innerHTML = '<span class="spinner"></span>Sending...';
  messageDiv.style.display = "none";

  try {
    const response = await fetch("/faucet", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ address }),
    });
    const data = await response.json();
    if (response.ok) {
      const result = data as FaucetResponse;
      showSuccess(`Tokens sent successfully!`, result.txHash);
      addressInput.value = "";
    } else {
      const error = data as ErrorResponse;
      if (error.error === "already_sent") {
        showError("This address has already received tokens from the faucet.");
      } else {
        showError(error.error || "Failed to send tokens. Please try again.");
      }
    }
  } catch (error) {
    showError("Network error. Please check your connection and try again.");
    console.error("Faucet error:", error);
  } finally {
    submitBtn.disabled = false;
    submitBtn.innerHTML = "Request Tokens";
  }
});

function showSuccess(message: string, txHash: string) {
  messageDiv.className = "message success";
  messageDiv.innerHTML = `
    <strong>✓ ${message}</strong>
    <div class="tx-hash">Transaction: <a href="https://celo.blockscout.com/tx/${txHash}" target="_blank" rel="noopener noreferrer">${txHash}</a></div>
  `;
  messageDiv.style.display = "block";
}

function showError(message: string) {
  messageDiv.className = "message error";
  messageDiv.innerHTML = `<strong>✗ ${message}</strong>`;
  messageDiv.style.display = "block";
}
