# VibeSentinel Agents

> Instructions for AI agents and subagents working on this project.

---

## Agent Roles

### 1. `explore` — Codebase Navigator

Use for: searching files, understanding structure, finding patterns.

**Instructions:**
- Search by naming conventions (e.g., `*train*`, `*model*`, `*imu*`)
- Grep for function/struct definitions
- Map crate boundaries and dependencies
- Return file paths + line numbers

### 2. `general` — Task Executor

Use for: multi-step development tasks, bug fixes, feature implementation.

**Instructions before coding:**
1. Read the relevant files completely
2. Check `vibesentinel_context.md` for constants and conventions
3. Check `design.md` for architecture rationale
4. Look at existing patterns in neighboring files
5. Run tests before and after changes

### 3. `customize-opencode` — Config Editor

Use for: editing opencode.json, opencode.jsonc, or .opencode/ files.

**Never use for:** application code changes.

---

## Workflow Rules

### Before Writing Code

1. Read the file you're about to edit
2. Check neighboring files for conventions
3. Review `design.md` for relevant decisions
4. Run `cargo check` to verify compilation

### Rust Conventions

- `no_std` crates: never import `std`, `Vec`, `String`, `Box`, `alloc`
- Use `libm` for math: `libm::sqrtf`, `libm::powf`, `libm::sinf`
- Fixed-size arrays `[f32; N]` instead of `Vec<f32>`
- `weights.rs` is auto-generated — edit the trainer, not the weights
- Golden vector tests verify x86 ↔ Xtensa floating-point parity

### Commit Rules

- One logical change per commit
- Prefix: `feat:`, `fix:`, `train:`, `docs:`, `refactor:`, `test:`, `chore:`
- Run `cargo test -p vibesentinel-features -p vibesentinel-model` before commit
- Never commit `weights.rs` changes without running golden vector tests

### File Creation Rules

- Never create documentation files (`.md`, README) unless explicitly requested
- Never add emojis to files unless asked
- Keep code comments to zero — code should be self-documenting

---

## Phase-Specific Guidance

### Phase 2 (Connectivity)
- WiFi: use `esp_idf_svc::wifi` with `embedded-svc` traits
- MQTT: use `esp_idf_svc::mqtt::client`
- OTA: use `esp_ota_ops` partition handling
- SD card: use `esp_idf_svc::sdmmc` or `vfs` with SPI SD card

### Phase 3 (Robustness)
- Multi-model: store multiple `weights.rs` variants, switch at runtime
- Adaptive threshold: use exponential moving average of recent MSE
- Multi-sensor: use `embedded-hal` traits, not direct I2C

### Phase 4 (Advanced ML)
- INT8: implement `q_matmul.rs` with `i8` dot product
- Fine-tuning: gradient descent with SGD (tiny steps) on-device
- Sensor fusion: expand FEATURE_DIM, add temperature/current features

### Phase 5 (Product)
- Dashboard: Next.js + Chart.js for real-time visualization
- Notifications: Telegram Bot API (HTTP POST from ESP32)
- CI/CD: GitHub Actions with `esp-rs/rust-build` action
