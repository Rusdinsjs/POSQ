<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { cartStore } from '$lib/stores/cart.svelte';
  import { showToast } from '$lib/toast.svelte';
  import type { PaymentMethod, PaymentLine } from '$lib/types';

  interface Props {
    isOpen: boolean;
    estimatedTotal: number;
    onClose: () => void;
    onSuccess?: (orderId: string) => void;
  }

  let { isOpen, estimatedTotal, onClose, onSuccess }: Props = $props();

  // Svelte 5 State Machine
  type PaymentStep = 'select_method' | 'input_amount' | 'processing' | 'success' | 'failed';
  let step = $state<PaymentStep>('select_method');

  // Selected Payment Method & Payments Array
  let selectedMethod = $state<PaymentMethod | null>(null);
  let paymentLines = $state<PaymentLine[]>([]);
  let inputAmount = $state<string>('');
  let lastOrderId = $state<string | null>(null);
  let errorMessage = $state<string>('');
  let amountInputEl = $state<HTMLInputElement | null>(null);

  // Derived Calculations
  let paidTotal = $derived(paymentLines.reduce((sum, p) => sum + p.amount, 0));
  let remainingBalance = $derived(Math.max(0, estimatedTotal - paidTotal));
  let changeTotal = $derived(Math.max(0, paidTotal - estimatedTotal));
  let canComplete = $derived(remainingBalance === 0 && paymentLines.length > 0 && step !== 'processing');

  // Helper Formatter
  function formatIDR(val: number): string {
    return new Intl.NumberFormat('id-ID', {
      style: 'currency',
      currency: 'IDR',
      minimumFractionDigits: 0
    }).format(val);
  }

  // Keyboard Escape Handler
  $effect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (isOpen && e.key === 'Escape' && step !== 'processing') {
        onClose();
      }
    }
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  });

  // Autofocus when entering input_amount step
  $effect(() => {
    if (isOpen && step === 'input_amount') {
      setTimeout(() => {
        amountInputEl?.focus();
      }, 50);
    }
  });

  // Reset modal state whenever opened
  $effect(() => {
    if (isOpen) {
      step = 'select_method';
      paymentLines = [];
      selectedMethod = null;
      inputAmount = '';
      lastOrderId = null;
      errorMessage = '';
    }
  });

  function selectMethod(method: PaymentMethod) {
    selectedMethod = method;
    if (method === 'cash') {
      inputAmount = remainingBalance > 0 ? remainingBalance.toString() : '';
      step = 'input_amount';
    } else {
      if (remainingBalance <= 0) {
        showToast('Tagihan sudah lunas', 'info');
        return;
      }
      paymentLines.push({ method, amount: remainingBalance });
      selectedMethod = null;
      step = 'select_method';
    }
  }

  function handleKeypadPress(num: string) {
    if (inputAmount === '0') {
      inputAmount = num;
    } else {
      inputAmount += num;
    }
  }

  function handleKeypadClear() {
    inputAmount = '';
  }

  function handleKeypadBackspace() {
    inputAmount = inputAmount.slice(0, -1);
  }

  function setQuickAmount(amount: number) {
    inputAmount = amount.toString();
  }

  function confirmAddCashPayment() {
    const parsed = parseInt(inputAmount, 10);
    if (isNaN(parsed) || parsed <= 0) {
      showToast('Masukkan nominal pembayaran yang valid', 'warning');
      return;
    }

    if (!selectedMethod) return;

    paymentLines.push({ method: selectedMethod, amount: parsed });
    selectedMethod = null;
    inputAmount = '';
    step = 'select_method';
  }

  function removePaymentLine(index: number) {
    paymentLines.splice(index, 1);
  }

  async function handleCompleteCheckout() {
    if (!canComplete) return;

    step = 'processing';
    errorMessage = '';

    try {
      const cart_payload = cartStore.prepareCheckoutPayload();
      const orderId = await invoke<string>('process_checkout', {
        cart_payload,
        payments: paymentLines
      });

      lastOrderId = orderId;
      step = 'success';
      showToast('Transaksi berhasil diproses!', 'success');
      if (onSuccess) onSuccess(orderId);
    } catch (err: any) {
      console.error('Checkout error:', err);
      const msg = typeof err === 'string' ? err : err?.message || 'Gagal memproses transaksi';
      errorMessage = msg;
      step = 'select_method';
      showToast(msg, 'error');
    }
  }

  async function handlePrintReceipt() {
    if (!lastOrderId) return;
    try {
      const receiptData = await invoke('get_receipt', { orderId: lastOrderId });
      await invoke('print_receipt', { data: receiptData });
      showToast('Struk berhasil dicetak', 'success');
    } catch (err: any) {
      showToast('Gagal mencetak struk: ' + (err?.message || err), 'error');
    }
  }

  function handleNewTransaction() {
    cartStore.clearCart();
    step = 'select_method';
    paymentLines = [];
    selectedMethod = null;
    inputAmount = '';
    lastOrderId = null;
    onClose();
  }
</script>

{#if isOpen}
  <!-- Modal Overlay -->
  <div
    class="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-md flex items-center justify-center p-4 overflow-y-auto"
    role="dialog"
    aria-modal="true"
    aria-labelledby="payment-modal-title"
  >
    <!-- Modal Card Container -->
    <div
      class="w-full max-w-xl bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh] transition-all"
    >
      <!-- Modal Header -->
      <div class="p-5 border-b border-slate-800 flex items-center justify-between bg-slate-900/80 shrink-0">
        <div>
          <h3 id="payment-modal-title" class="text-lg font-bold text-white tracking-tight">
            Pembayaran Transaksi
          </h3>
          <p class="text-xs text-slate-400 mt-0.5">Pilih metode dan selesaikan transaksi kasir</p>
        </div>

        {#if step !== 'processing'}
          <button
            type="button"
            onclick={onClose}
            class="w-9 h-9 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-400 hover:text-white flex items-center justify-center transition-colors cursor-pointer"
            aria-label="Tutup modal pembayaran"
          >
            ✕
          </button>
        {/if}
      </div>

      <!-- Modal Body -->
      <div class="p-6 overflow-y-auto flex-1 custom-scrollbar space-y-5">
        <!-- Summary Header -->
        <div class="bg-slate-800/60 border border-slate-700/60 rounded-2xl p-4 space-y-2">
          <div class="flex justify-between items-center text-sm">
            <span class="text-slate-400">Total Tagihan (Estimasi):</span>
            <span class="font-extrabold text-white font-mono text-base">{formatIDR(estimatedTotal)}</span>
          </div>

          <div class="flex justify-between items-center text-sm">
            <span class="text-slate-400">Total Dibayar:</span>
            <span class="font-bold text-emerald-400 font-mono">{formatIDR(paidTotal)}</span>
          </div>

          <div class="border-t border-slate-700/60 pt-2 flex justify-between items-center text-sm">
            <span class="font-semibold text-slate-300">Sisa yang harus dibayar:</span>
            <span
              class="font-black font-mono text-base {remainingBalance === 0
                ? 'text-emerald-400'
                : 'text-amber-400'}"
            >
              {formatIDR(remainingBalance)}
            </span>
          </div>

          {#if changeTotal > 0}
            <div class="flex justify-between items-center text-sm text-blue-400 pt-1 border-t border-slate-700/40">
              <span class="font-semibold">Kembalian:</span>
              <span class="font-bold font-mono text-base">{formatIDR(changeTotal)}</span>
            </div>
          {/if}
        </div>

        <!-- Split Payment Lines List -->
        {#if paymentLines.length > 0}
          <div class="space-y-2">
            <h4 class="text-xs font-bold text-slate-400 uppercase tracking-wider">Daftar Pembayaran (Split Payment):</h4>
            <div class="space-y-1.5">
              {#each paymentLines as p, i}
                <div
                  class="flex justify-between items-center bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-xs"
                >
                  <span class="font-semibold text-slate-200 uppercase">{p.method}</span>
                  <div class="flex items-center gap-2">
                    <span class="font-mono text-emerald-400 font-bold">{formatIDR(p.amount)}</span>
                    {#if step !== 'processing' && step !== 'success'}
                      <button
                        type="button"
                        onclick={() => removePaymentLine(i)}
                        class="text-rose-400 hover:text-rose-300 font-bold px-1.5 py-0.5 rounded hover:bg-rose-500/10 cursor-pointer"
                        aria-label="Hapus metode pembayaran"
                      >
                        ✕
                      </button>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- STEP 1: SELECT METHOD -->
        {#if step === 'select_method'}
          <div class="space-y-3">
            <h4 class="text-xs font-bold text-slate-400 uppercase tracking-wider">Pilih Metode Pembayaran:</h4>
            <div class="grid grid-cols-2 gap-3">
              <button
                type="button"
                onclick={() => selectMethod('cash')}
                class="p-4 rounded-2xl bg-slate-800 hover:bg-slate-700/80 border border-slate-700 hover:border-blue-500/50 flex flex-col items-center justify-center gap-2 text-white font-bold transition-all active:scale-95 cursor-pointer shadow-sm group"
                aria-label="Metode Pembayaran Tunai"
              >
                <span class="text-3xl group-hover:scale-110 transition-transform">💵</span>
                <span>Tunai (Cash)</span>
              </button>

              <button
                type="button"
                onclick={() => selectMethod('card')}
                class="p-4 rounded-2xl bg-slate-800 hover:bg-slate-700/80 border border-slate-700 hover:border-blue-500/50 flex flex-col items-center justify-center gap-2 text-white font-bold transition-all active:scale-95 cursor-pointer shadow-sm group"
                aria-label="Metode Pembayaran Kartu EDC"
              >
                <span class="text-3xl group-hover:scale-110 transition-transform">💳</span>
                <span>Kartu (Card)</span>
              </button>

              <button
                type="button"
                onclick={() => selectMethod('qris')}
                class="p-4 rounded-2xl bg-slate-800 hover:bg-slate-700/80 border border-slate-700 hover:border-blue-500/50 flex flex-col items-center justify-center gap-2 text-white font-bold transition-all active:scale-95 cursor-pointer shadow-sm group"
                aria-label="Metode Pembayaran QRIS"
              >
                <span class="text-3xl group-hover:scale-110 transition-transform">📱</span>
                <span>QRIS</span>
              </button>

              <button
                type="button"
                onclick={() => selectMethod('ewallet')}
                class="p-4 rounded-2xl bg-slate-800 hover:bg-slate-700/80 border border-slate-700 hover:border-blue-500/50 flex flex-col items-center justify-center gap-2 text-white font-bold transition-all active:scale-95 cursor-pointer shadow-sm group"
                aria-label="Metode Pembayaran E-Wallet"
              >
                <span class="text-3xl group-hover:scale-110 transition-transform">👛</span>
                <span>E-Wallet</span>
              </button>
            </div>
          </div>
        {/if}

        <!-- STEP 2: INPUT AMOUNT (CASH KEYPAD) -->
        {#if step === 'input_amount'}
          <div class="space-y-4">
            <div class="flex justify-between items-center">
              <h4 class="text-xs font-bold text-slate-400 uppercase tracking-wider">Input Nominal Tunai:</h4>
              <button
                type="button"
                onclick={() => {
                  step = 'select_method';
                  selectedMethod = null;
                }}
                class="text-xs text-slate-400 hover:text-white underline cursor-pointer"
              >
                ← Ganti Metode
              </button>
            </div>

            <!-- Amount Input Display -->
            <div class="relative">
              <input
                bind:this={amountInputEl}
                type="text"
                bind:value={inputAmount}
                placeholder="0"
                class="w-full bg-slate-950 border border-slate-700 rounded-2xl py-3.5 px-4 text-right font-mono font-black text-2xl text-emerald-400 focus:outline-none focus:border-blue-500"
                aria-label="Input nominal pembayaran tunai"
              />
            </div>

            <!-- Quick Nominal Buttons -->
            <div class="grid grid-cols-4 gap-2">
              <button
                type="button"
                onclick={() => setQuickAmount(remainingBalance)}
                class="py-2.5 px-1 rounded-xl bg-blue-600/20 hover:bg-blue-600/30 border border-blue-500/40 text-blue-300 font-bold text-xs transition-all active:scale-95 cursor-pointer"
              >
                Uang Pas
              </button>
              <button
                type="button"
                onclick={() => setQuickAmount(50000)}
                class="py-2.5 px-1 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-200 font-bold text-xs transition-all active:scale-95 cursor-pointer font-mono"
              >
                50.000
              </button>
              <button
                type="button"
                onclick={() => setQuickAmount(100000)}
                class="py-2.5 px-1 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-200 font-bold text-xs transition-all active:scale-95 cursor-pointer font-mono"
              >
                100.000
              </button>
              <button
                type="button"
                onclick={() => setQuickAmount(200000)}
                class="py-2.5 px-1 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-200 font-bold text-xs transition-all active:scale-95 cursor-pointer font-mono"
              >
                200.000
              </button>
            </div>

            <!-- Virtual 3x4 Keypad -->
            <div class="grid grid-cols-3 gap-2 pt-1">
              {#each ['1', '2', '3', '4', '5', '6', '7', '8', '9'] as num}
                <button
                  type="button"
                  onclick={() => handleKeypadPress(num)}
                  class="h-12 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700 text-white font-bold text-base transition-all active:scale-95 cursor-pointer font-mono"
                  aria-label="Nomor {num}"
                >
                  {num}
                </button>
              {/each}
              <button
                type="button"
                onclick={handleKeypadClear}
                class="h-12 rounded-xl bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/30 text-rose-400 font-bold text-sm transition-all active:scale-95 cursor-pointer"
                aria-label="Bersihkan nominal"
              >
                C
              </button>
              <button
                type="button"
                onclick={() => handleKeypadPress('0')}
                class="h-12 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700 text-white font-bold text-base transition-all active:scale-95 cursor-pointer font-mono"
                aria-label="Nomor 0"
              >
                0
              </button>
              <button
                type="button"
                onclick={handleKeypadBackspace}
                class="h-12 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-300 font-bold text-base transition-all active:scale-95 cursor-pointer"
                aria-label="Hapus digit terakhir"
              >
                ⌫
              </button>
            </div>

            <button
              type="button"
              onclick={confirmAddCashPayment}
              class="w-full py-3.5 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-sm transition-all active:scale-95 cursor-pointer shadow-md"
            >
              Tambahkan Pembayaran Tunai
            </button>
          </div>
        {/if}

        <!-- STEP 3: SUCCESS VIEW -->
        {#if step === 'success'}
          <div class="py-6 flex flex-col items-center justify-center text-center space-y-4">
            <div
              class="w-16 h-16 rounded-full bg-emerald-500/20 border border-emerald-500/40 text-emerald-400 flex items-center justify-center text-3xl shadow-lg shadow-emerald-500/20"
            >
              ✓
            </div>
            <div>
              <h4 class="text-xl font-black text-white tracking-tight">Transaksi Berhasil!</h4>
              <p class="text-xs text-slate-400 mt-1 font-mono">ID Orde: {lastOrderId}</p>
            </div>

            <div class="flex flex-col w-full gap-2.5 pt-4">
              <button
                type="button"
                onclick={handlePrintReceipt}
                class="w-full py-3.5 rounded-xl bg-blue-600 hover:bg-blue-500 text-white font-extrabold text-sm transition-all active:scale-95 cursor-pointer flex items-center justify-center gap-2 shadow-lg shadow-blue-600/30"
              >
                <span>🖨️</span>
                <span>Cetak Struk</span>
              </button>

              <button
                type="button"
                onclick={handleNewTransaction}
                class="w-full py-3.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 font-bold text-sm border border-slate-700 transition-all active:scale-95 cursor-pointer"
              >
                Transaksi Baru
              </button>
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Footer (Complete Checkout Action) -->
      {#if step !== 'success'}
        <div class="p-5 border-t border-slate-800 bg-slate-900/90 flex justify-end gap-3 shrink-0">
          {#if step !== 'processing'}
            <button
              type="button"
              onclick={onClose}
              class="py-3 px-5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 font-bold text-sm transition-all active:scale-95 cursor-pointer"
            >
              Batal
            </button>
          {/if}

          <button
            type="button"
            onclick={handleCompleteCheckout}
            disabled={!canComplete}
            class="py-3.5 px-6 rounded-xl bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 disabled:pointer-events-none text-white font-extrabold text-sm transition-all active:scale-95 cursor-pointer shadow-lg shadow-emerald-600/30 flex items-center gap-2 {step ===
            'processing'
              ? 'animate-pulse'
              : ''}"
            aria-label="Selesaikan Pembayaran"
          >
            {#if step === 'processing'}
              <span>Memproses Transaksi...</span>
            {:else}
              <span>SELESAIKAN PEMBAYARAN</span>
            {/if}
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}
