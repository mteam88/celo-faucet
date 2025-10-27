interface FaucetResponse {
  txHash: string;
}

interface ErrorResponse {
  error: string;
}

const form = document.getElementById("faucetForm") as HTMLFormElement;
const addressInput = document.getElementById("address") as HTMLInputElement;
const submitBtn = document.getElementById("submitBtn") as HTMLButtonElement;
const messageDiv = document.getElementById("message") as HTMLDivElement;

form.addEventListener("submit", async (e) => {
  e.preventDefault();

  const address = addressInput.value.trim();

  // Basic validation
  if (!address.match(/^0x[a-fA-F0-9]{40}$/)) {
    showError("Please enter a valid Ethereum address");
    return;
  }

  // Disable form during request
  submitBtn.disabled = true;
  submitBtn.innerHTML = '<span class="spinner"></span>Sending...';
  messageDiv.style.display = "none";

  try {
    const response = await fetch("/faucet", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
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
    <div class="tx-hash">Transaction: <a href="https://celo.blockscout.com/tx/${txHash}" target="_blank">${txHash}</a></div>
  `;
  messageDiv.style.display = "block";
}

function showError(message: string) {
  messageDiv.className = "message error";
  messageDiv.innerHTML = `<strong>✗ ${message}</strong>`;
  messageDiv.style.display = "block";
}
