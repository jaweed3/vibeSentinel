# Panduan Anak Magang — VibeSentinel

> lo magang, lo disuruh ngurusin project ini, lo bingung? tenang, gw jelasin dari nol.
> baca pelan-pelan, sambil nyeruput kopi. semua yang lo perlu tau ada di sini.

---

## Daftar Isi

1. [Project ini apaan sih?](#1-project-ini-apaan-sih)
2. [Arsitektur: Siapa ngapain](#2-arsitektur-siapa-ngapain)
3. [Cara jalanin dari nol](#3-cara-jalanin-dari-nol)
4. [Alur data: dari getaran sampe LED nyala](#4-alur-data-dari-getaran-sampe-led-nyala)
5. [Testing: jangan sampe nge-push bug](#5-testing-jangan-sampe-nge-push-bug)
6. [Ngoprek & ngembangin](#6-ngoprek--ngembangin)
7. [Troubleshooting umum](#7-troubleshooting-umum)
8. [FAQ anak magang](#8-faq-anak-magang)

---

## 1. Project Ini Apaan Sih?

### Masalah nyata

Lo punya pabrik. Di dalemnya ada ratusan motor listrik, conveyor belt, pompa, kipas angin raksasa. Kalo salah satu mesin tiba-tiba rusak:

- **Produksi berhenti** — 1 jam downtime bisa rugi puluhan juta
- **Maintenance darurat** — manggil teknisi tengah malem, mahal
- **Part rusak tambah parah** — bearing yg harusnya cuma Rp 500rb malah ngerusak poros yg harganya Rp 50jt

Cara tradisional: maintenance terjadwal (ganti part tiap 6 bulan, padahal masi bagus — buang duit) ATAU maintenance reaktif (tunggu rusak dulu — udah telat).

### Solusi: VibeSentinel

VibeSentinel adalah **sensor getaran pintar** yang ditempel di mesin. Dia belajar kayak apa suara "normal" mesin lo, dan langsung teriak kalo ada yg aneh — **sebelum** mesinnya jebol.

Analoginya gini: lo tau kan suara motor sehat vs motor yg mulai ada gesekan aneh? Nah ini versi AI-nya.

### Kenapa harus VibeSentinel, bukan solusi cloud?

| Solusi Cloud (AWS IoT, dll) | VibeSentinel |
|---|---|
| Butuh internet stabil | Jalan offline |
| Bayar subscription tiap bulan | Beli hardware 1x (~Rp 200rb) |
| Latency 100-500ms (network) | Latency < 5ms (lokal) |
| Data keluar pabrik (security risk) | Data gak kemana-mana |
| Butuh orang ML buat maintain pipeline | Training 1x di laptop, upload, beres |

### Use case konkret

1. **Monitor motor pabrik** — tempel di housing motor, deteksi bearing aus dari perubahan getaran
2. **Conveyor belt peternakan** — deteksi anomali belt conveyor pakan ayam
3. **HVAC gedung** — deteksi kipas gedung mulai rusak sebelum anak kantor ngeluh panas
4. **Produk retrofit** — jual hardware Rp 250rb + firmware gratis, charge SaaS buat dashboard cloud

---

## 2. Arsitektur: Siapa Ngapain

Project ini dibagi jadi 4 crate (Rust-speak untuk "modul"):

```
vibesentinel/
├── crates/
│   ├── vibesentinel-features/   ← DSP, ngolah getaran jadi angka
│   ├── vibesentinel-model/      ← Otaknya, autoencoder neural network
│   ├── vibesentinel-trainer/    ← Training di laptop, hasilnya export
│   └── vibesentinel-firmware/   ← Program yg jalan di ESP32-S3
├── data/                        ← CSV data getaran
├── docs/                        ← Dokumentasi (lo baca ini)
├── scripts/                     ← Bootstrap & build scripts
└── build-firmware.sh            ← Pipeline lengkap: train → build
```

### Crate 1: `vibesentinel-features` (no_std)

**Tugasnya:** Ngolah 128 sampel getaran mentah jadi 20 fitur yg bermakna.

Kenapa 20 fitur, bukan langsung 128 sampel? Karena ngasih 128 angka mentah ke neural network itu boros — model jadi gede, lambat, dan susah belajar. 20 fitur ini adalah "rangkuman cerdas" dari getaran.

**20 fitur itu isinya:**
- **Per axis (X, Y, Z) × 6 fitur = 18 fitur:**
  - **RMS** — energi total getaran
  - **Peak** — amplitudo maksimum
  - **Kurtosis** — impulsifitas (tiba-tiba nyentak?), indikator bearing rusak
  - **Crest Factor** — peak/RMS, deteksi early wear sebelum RMS berubah
  - **FFT Bin 0** — komponen DC (beban statis)
  - **FFT Bin 1** — frekuensi dominan (kecepatan rotasi)
- **2 fitur cross-axis:**
  - **Axial/Radial ratio** — indikator misalignment
  - **Total RMS** — keparahan getaran keseluruhan (standar ISO 10816)

**File penting:**
| File | Isinya |
|---|---|
| `stats.rs` | RMS, Peak, Kurtosis, Crest Factor, Variance |
| `fft.rs` | FFT wrapper pake `microfft` |
| `extractor.rs` | Si "otak" yg manggil semua stats + FFT, output 20 fitur |
| `window.rs` | Sliding window buffer (128 sampel per axis) |

### Crate 2: `vibesentinel-model` (no_std)

**Tugasnya:** Neural network autoencoder yg bisa jalan tanpa heap, tanpa allocator, tanpa OS — murni stack-allocated arrays.

**Arsitektur Autoencoder: 20 → 10 → 4 → 10 → 20**

```
Input (20 fitur)
  → Encoder Layer 1: 20→10 (ReLU)
  → Encoder Layer 2: 10→4  (ReLU)     ← ini "bottleneck", cuma 4 angka!
  → Decoder Layer 1: 4→10  (ReLU)
  → Decoder Layer 2: 10→20 (Sigmoid)
  → Output (20 fitur rekonstruksi)
```

Konsepnya gini: model dilatih **hanya** pake data normal. Dia belajar "ngompres" 20 fitur jadi 4 angka (latent space), lalu "dekompres" balik. Kalo inputnya normal → rekonstruksi mirip (error kecil). Kalo inputnya anomali → rekonstruksi jelek (error besar).

**524 parameter total × 4 bytes = ~2KB memory.** Bener-bener kecil.

**File penting:**
| File | Isinya |
|---|---|
| `arch.rs` | Forward pass, reconstruction error, normalisasi fitur |
| `activations.rs` | ReLU dan Sigmoid (no_std, pake `libm`) |
| `matmul.rs` | Matrix multiply fixed-size, zero-allocation |
| `weights.rs` | **AUTO-GENERATED** — isinya weights, bias, threshold, mean, std |

### Crate 3: `vibesentinel-trainer` (Desktop only)

**Tugasnya:** Training autoencoder pake **Burn** (Rust deep learning framework), lalu export weights ke `weights.rs`.

Ini cuma jalan di laptop/desktop lo — bukan di ESP32. Outputnya file `weights.rs` yg nanti di-compile bareng firmware.

**Alur training:**
1. Load data dari CSV (atau generate synthetic)
2. Hitung mean & std per fitur (buat normalisasi)
3. Normalisasi semua sampel (z-score, clip ke [-3, 3])
4. Split 85% train / 15% validation
5. Training loop 200 epoch pake Adam optimizer
6. Hitung reconstruction error semua validation samples
7. **Threshold = mean(error) + 3.0 × std(error)** — 3-sigma rule
8. Export weights, bias, threshold, mean, std → `weights.rs`
9. Export golden features buat cross-architecture parity test

**File penting:**
| File | Isinya |
|---|---|
| `model.rs` | Definisi model Burn (harus match sama `arch.rs`) |
| `train.rs` | Training loop + kalibrasi threshold |
| `dataset.rs` | CSV loader & synthetic data generator |
| `export.rs` | Write `weights.rs` — outputnya valid Rust code |
| `main.rs` | CLI entry point |

### Crate 4: `vibesentinel-firmware` (ESP32-S3)

**Tugasnya:** Program utama yg jalan di ESP32-S3. Baca sensor → ekstrak fitur → inference → alert.

**Safety features (hasil fix 10 GitHub issues):**
- ✅ Precise timer-based sampling (200Hz akurat)
- ✅ Watchdog timer — kalo hang, auto-reboot
- ✅ I2C error recovery — kalo bus I2C ngadat, reset otomatis
- ✅ Frozen sensor detection — deteksi kalo sensor macet
- ✅ Log only on state change — gak spam UART
- ✅ Configurable G-range (default 8G)

**File penting:**
| File | Isinya |
|---|---|
| `main.rs` | Main loop: sampling → windowing → features → inference → alert |
| `imu.rs` | LSM6DS3 driver via I2C |
| `alert.rs` | GPIO LED control |
| `config.rs` | Konstanta: sample rate, interval, dll |

---

## 3. Cara Jalanin dari Nol

### Prasyarat

Elo butuh:
- **Mac/Linux** (Windows kurang recommended)
- **Rust nightly** — `rustup toolchain install nightly`
- **Python 3.11** (buat ESP-IDF build system)
- **ESP32-S3** + sensor LSM6DS3 (buat firmware)

### Step 1: Clone & Bootstrap

```bash
git clone <repo-url> vibesentinel
cd vibesentinel

# Setup ESP-IDF environment (cuma perlu sekali)
chmod +x scripts/*.sh
./scripts/bootstrap.sh
```

### Step 2: Verify — Tes Dulu!

```bash
# Test fitur extraction
cargo test -p vibesentinel-features
# Harus: 10 tests passed

# Test model inference
cargo test -p vibesentinel-model
# Harus: 8 tests passed
```

### Step 3: Training Model

Elo bisa pake data synthetic (default) atau data CSV asli:

```bash
# Pake synthetic data (buat development)
cargo run --release -p vibesentinel-trainer

# Pake data asli dari CSV
cargo run --release -p vibesentinel-trainer -- \
    --data data/normal_vibration.csv \
    --epochs 200 \
    --lr 0.001 \
    --sigma 3.0
```

**Output yg diharapkan:**
```
=== VibeSentinel Calibration Report ===
  Validation samples:    150
  Mean recon error:      0.018432
  Std  recon error:      0.004821
  Threshold (k=3.0):     0.032895

  Percentile distribution:
    P50:   0.0172
    P90:   0.0241
    P95:   0.0268
    P99:   0.0305
    P99.9: 0.0329
=======================================
```

Abis itu `weights.rs` ke-update otomatis.

### Step 4: Build Firmware

```bash
# Pake build script
./build-firmware.sh

# Atau manual:
cd crates/vibesentinel-firmware
cargo +esp build --release --target xtensa-esp32s3-espidf
```

### Step 5: Flash ke ESP32

```bash
espflash flash target/xtensa-esp32s3-espidf/release/vibesentinel-firmware --monitor
```

**Output serial yg diharapkan:**
```
VibeSentinel online. Range: 8G | Threshold: 1.869323 | 5ms period
NORMAL: MSE: 0.012345 (returned to normal)
!!! ANOMALY DETECTED !!! MSE: 0.045678 > Threshold: 1.869323
```

### Step 6: Tes Anomali — Pukul Mesinnya

Sambil firmware jalan, coba ketok sensornya atau getarin kuat-kuat. LED harus nyala dan serial keluar `!!! ANOMALY !!!`.

### Ngumpulin Data Sendiri

Sebelum training beneran, lo perlu ngumpulin data getaran "normal" dari mesin beneran:

1. Flash firmware data collector:
   ```rust
   // ganti main.rs dengan ini:
   loop {
       FreeRtos::delay_ms(5);
       let (x, y, z) = imu.read_accel().unwrap();
       println!("{:.6},{:.6},{:.6}", x, y, z);
   }
   ```

2. Capture ke CSV:
   ```bash
   espflash monitor --baud 115200 | grep -E "^-?[0-9]" | tee data/normal_vibration.csv
   ```

3. Rekam minimal 10 menit (120,000 sampel) operasi normal

---

## 4. Alur Data: dari Getaran Sampe LED Nyala

```
╔══════════════════════════════════════════════════════════════╗
║                    FIRMWARE LOOP (ESP32-S3)                   ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  ┌──────────┐    raw_accel     ┌────────────────┐           ║
║  │  IMU     │─────────────────▶│ Sliding Window  │           ║
║  │ LSM6DS3  │   [f32; 3]       │ 128 samples     │           ║
║  │ @200Hz   │   @5ms interval  │ per axis (X,Y,Z)│           ║
║  │ ±8G      │                  └───────┬──────────┘           ║
║  └──────────┘                          │                      ║
║                                        ▼                      ║
║                               ┌────────────────┐             ║
║                               │ Feature         │             ║
║                               │ Extraction      │             ║
║                               │                 │             ║
║                               │ RMS, Peak,      │             ║
║                               │ Kurtosis, Crest,│             ║
║                               │ FFT[0..1]       │             ║
║                               │ → [f32; 20]     │             ║
║                               └───────┬──────────┘             ║
║                                       │                        ║
║                          ┌────────────▼────────────┐          ║
║                          │ Sensor Health Check      │          ║
║                          │ Variance > 0.0001 ?      │          ║
║                          │ YES → lanjut             │          ║
║                          │ NO  → SENSOR_FAILURE     │          ║
║                          └────────────┬────────────┘          ║
║                                       │                        ║
║                                       ▼                        ║
║                          ┌────────────────────────┐           ║
║                          │ Normalize (Z-score)     │           ║
║                          │ (x - mean) / std        │           ║
║                          │ clip to [-3, 3]         │           ║
║                          └───────────┬────────────┘           ║
║                                      │                         ║
║                                      ▼                         ║
║                          ┌────────────────────────┐           ║
║                          │ Autoencoder Forward     │           ║
║                          │ 20→10→4→10→20          │           ║
║                          │ (524 params, ~2KB)      │           ║
║                          └───────────┬────────────┘           ║
║                                      │                         ║
║                                      ▼                         ║
║                          ┌────────────────────────┐           ║
║                          │ MSE(input, output)      │           ║
║                          │ > ANOMALY_THRESHOLD ?   │           ║
║                          └───┬──────────────┬─────┘           ║
║                              │ NO           │ YES              ║
║                              ▼              ▼                  ║
║                         LED OFF        LED ON                  ║
║                         Log "NORMAL"   Log "ANOMALY!"          ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```

---

## 5. Testing: Jangan Sampe Nge-Push Bug

### Run semua tests

```bash
# Unit tests (no_std crates)
cargo test -p vibesentinel-features
cargo test -p vibesentinel-model

# Trainer (butuh lebih lama, compile Burn)
cargo check -p vibesentinel-trainer
```

### Apa yg di-test?

**`vibesentinel-features` (10 tests):**
- RMS constant signal = 1.0
- RMS empty input = 0 (gak panic)
- Peak detection akurat
- Variance = 0 buat constant signal
- Variance ≠ 0 buat signal bervariasi
- Crest factor zero input = 0
- Crest factor input normal > 0
- Kurtosis sinyal gaussian ≈ 3.0
- Kurtosis constant signal = 0 (gak NaN)
- FFT 50Hz dominan di bin 32

**`vibesentinel-model` (8 tests):**
- Forward pass output di range [0, 1]
- Reconstruction error ≈ 0 kalo input = output
- Reconstruction error > 0 kalo input ≠ output
- Normalisasi clip di [-3, 3]
- Normalisasi nilai mean ≈ 0
- **Cross-architecture parity test** — bandingin output no_std vs output trainer

### Nambah test baru

Kalo lo nambah fitur, WAJIB nambah test. Contoh:

```rust
#[test]
fn test_fitur_baru_gue() {
    let samples = [1.0f32; 128];
    let result = fitur_baru(&samples);
    assert!((result - 1.0).abs() < 1e-5);
}
```

---

## 6. Ngoprek & Ngembangin

### Guide: Nambah Fitur Baru ke Ekstraksi

Misal lo mau nambah fitur statistik baru (misalnya skewness):

1. **Tambah di `stats.rs`:**
   ```rust
   pub fn skewness(samples: &[f32]) -> f32 {
       // rumus skewness...
   }
   ```

2. **Update dimensi di `lib.rs`:**
   ```rust
   pub const FEATURE_DIM: usize = 21; // tadinya 20
   ```

3. **Update extractor** — tambahin fitur baru di output array

4. **Update arsitektur model** — ubah dimensi di `arch.rs` dan trainer

5. **Re-train** — jalanin trainer ulang

6. **Test!**

### Guide: Ganti Jenis Sensor

Misal lo mau pake MPU-6050 instead of LSM6DS3:

1. Update `imu.rs` — ganti register addresses
2. Sesuaikan I2C address (MPU-6050 = 0x68)
3. Update scale factor
4. Test

### Guide: Nambah Output (WiFi, MQTT, SD Card)

Ini fitur advanced. Kerangkanya udah ada:

```rust
// Di main.rs, setelah deteksi anomali:
if is_anomaly {
    // TODO: kirim HTTP POST ke webhook
    // pake esp_idf_svc::http::client
}
```

### Roadmap Pengembangan

#### Fase 1: Connectivity (paling deket)
- [ ] **WiFi webhook** — POST data anomali ke server
- [ ] **MQTT telemetry** — stream MSE tiap window
- [ ] **SD card logging** — simpan history ke microSD

#### Fase 2: Robustness
- [ ] **Multi-model** — beda model buat idle vs loaded vs startup
- [ ] **Adaptive threshold** — threshold menyesuaikan drift suhu/musim
- [ ] **OTA update** — update model tanpa reflash firmware

#### Fase 3: Advanced ML
- [ ] **Quantization INT8** — model 4× lebih kecil & cepat
- [ ] **On-device fine-tuning** — model belajar dari data baru
- [ ] **Multi-sensor fusion** — suhu + getaran + arus listrik

#### Fase 4: Product
- [ ] **Web dashboard** (React/Next.js) — visualisasi real-time
- [ ] **Alerting** — Telegram/WhatsApp notification
- [ ] **Multi-device management** — pantau puluhan sensor dari 1 dashboard

### Konvensi Koding

- **Jangan pake allocator di crate no_std** — semua musti stack-allocated arrays
- **Math pake `libm`**, bukan `f32::sqrt()` — ESP32 gak selalu punya std math
- **Jangan clone buffer gede** — di embedded, setiap byte berharga
- **Test dulu sebelum commit** — `cargo test -p vibesentinel-features -p vibesentinel-model`
- **Gak usah komen yg obvious** — kode yang bagus itu self-documenting

---

## 7. Troubleshooting Umum

### "microfft::rfft_128 not found"

Pastiin `microfft = "0.6"` di Cargo.toml features. API-nya `rfft_128`, bukan `rfft_128` (nama fungsi).

### "linker error: __rust_begin_short_backtrace"

Lo compile no_std crate sebagai binary? Pastiin `#![no_std]` di lib.rs dan crate ini cuma library.

### "Burn compilation error"

Burn berat banget dependencies-nya. Pastiin lo pake `--release` buat trainer. Dan sabar — first compile bisa 5-10 menit.

### "ESP-IDF Python version mismatch"

ESP-IDF maunya Python 3.11. Kalo lo pake 3.12/3.13, bakal error. Pake `pyenv` buat switch:
```bash
pyenv local 3.11.9
```

### "I2C timeout / sensor not responding"

- Cek wiring: SDA ke GPIO8, SCL ke GPIO9
- Cek power: LSM6DS3 butuh 3.3V
- Cek address: LSM6DS3 = 0x6A (bukan 0x68, itu MPU-6050)
- Kalo error persistent > 5x, firmware bakal auto-recovery

### "LED nyala terus / false alarm"

Threshold terlalu rendah. Coba naikin:
- Re-train dengan `--sigma 4.0` (default 3.0)
- Atau edit manual `ANOMALY_THRESHOLD` di `weights.rs`

### "Trainer error: csv::Error"

Format CSV musti: `timestamp_ms,accel_x,accel_y,accel_z`
Gak boleh ada header tambahan, gak boleh ada kolom lain.

---

## 8. FAQ Anak Magang

**Q: Gw gak ngerti ML/AI, bisa ngoprek ini?**
A: Bisa. Lo gak perlu ngerti backpropagation. Fokus aja di feature extraction (`features/`) atau firmware (`firmware/`). Modelnya udah jadi, tinggal pake.

**Q: Bedanya `no_std` sama `std` apaan?**
A: `no_std` = gak pake Rust standard library, cocok buat embedded (gak ada OS, gak ada heap). `std` = full Rust, pake Vec, String, File I/O, dll. Di project ini, features & model = `no_std`, trainer & firmware = `std`.

**Q: Kenapa pake Burn, bukan PyTorch/TensorFlow?**
A: Zero Python. Zero C. 100% Rust. Burn generate Rust code yg bisa langsung di-export. PyTorch output-nya Python pickle, gak bisa dipake embedded.

**Q: Kenapa arsitekturnya 20→10→4→10→20? Kenapa gak lebih gede?**
A: Lebih gede = lebih banyak parameter = lebih lambat di ESP32. 524 parameter udah cukup buat deteksi anomali getaran. Ini deliberately small.

**Q: ESP32 cukup kuat gak sih?**
A: ESP32-S3 240MHz dual-core, 8MB PSRAM. Forward pass 524 params itu < 1ms. Lebih dari cukup. Budget kita microseconds, bukan milliseconds.

**Q: Bisa gak pake ESP32 biasa (bukan S3)?**
A: Bisa, tapi S3 lebih recommended karena PSRAM-nya. ESP32 biasa cuma 520KB SRAM. Masih muat sih, tapi mepet.

**Q: Gimana cara nambah sensor selain getaran? (suhu, arus, dll)**
A: Bisa. Tambahin sensor di firmware, tambahin fitur di extractor (FEATURE_DIM lo naikin), re-train model. Pastiin arsitektur model di-update sesuai.

**Q: Golden features test itu apaan?**
A: Test yg mastiin inference di laptop (Burn) outputnya SAMA PERSIS dengan inference di ESP32 (no_std). Ini penting karena arsitektur CPU beda (x86 vs Xtensa) bisa bikin floating-point beda hasil.

**Q: Kok threshold-nya 1.86 (gede banget)? Kok bukan 0.01?**
A: Itu threshold buat FEATURE MEAN (nilai mentah fitur sebelum normalisasi). Setelah normalisasi, MSE bakal kecil. Threshold dihitung otomatis dari validation set — jangan diotak-atik manual.

---

## Referensi Tambahan

- [`vibesentinel_context.md`](../vibesentinel_context.md) — Konteks teknis lengkap (arsitektur, dependencies, algoritma)
- [`docs/architecture.md`](architecture.md) — Diagram dan alur data
- [`docs/getting-started.md`](getting-started.md) — Setup & deployment singkat
- [`docs/overview.md`](overview.md) — Gambaran besar project
- [`docs/current-state-future-implementation.md`](current-state-future-implementation.md) — Roadmap

---

*Ditulis pake semangat anak magang, buat anak magang. Kalo ada yg kurang jelas, tanya aja — gak ada pertanyaan bodoh.*
