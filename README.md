# Kısayolluk

**Kısayolluk** is a tiny, Rust‑based Windows utility that opens a dark, middle‑click‑activated radial menu for file‑based actions. The menu scales automatically, uses the Slint UI framework, and runs with minimal RAM on Windows 10/11.

---

## Features
- **Middle‑click activation** – Open the radial menu with a single mouse‑wheel click on a selected file.
- **Dynamic layout** – Menu slices are positioned symmetrically and automatically adjust when new actions are added.
- **Low resource usage** – Small binary (~19 MB) and minimal RAM consumption.
- **Built with Rust & Slint** – Fast, safe, and modern UI.
- **Windows 10/11 support** – Uses native `WH_MOUSE_LL` hook and embedded WinRT resources for proper Task‑Manager identification.
- **Preserves existing functionality** – All existing conversion and event‑handling logic remains unchanged.

---

## Installation
1. **Prerequisites**
   - Windows 10 or 11 (64‑bit).
   - Rust toolchain (stable) – install via https://rustup.rs.
2. **Clone the repository**
   ```bash
   git clone https://github.com/<your‑username>/Kısayolluk.git
   cd Kısayolluk
   ```
3. **Build the release binary**
   ```bash
   cargo build --release
   ```
   The executable will be located at `target\release\kısayolluk.exe`.

---

## Usage
1. **Select a file** in Windows Explorer (or on the Desktop) with a normal left‑click.
2. **Press the middle mouse button** (mouse wheel click). The radial menu appears centered on the cursor.
3. **Hover** over a slice to highlight it (orange by default) and **click** to execute the associated action.
4. **Close** the menu by clicking anywhere outside the ring or pressing `Esc`.

> **Note:** The traditional `Ctrl + Right‑Click` shortcut has been removed; only middle‑click opens the menu.

---

## Building & Development
- The project uses the `winres` crate to embed Windows resource information (product name, description, etc.). This ensures the process shows up as **Kısayolluk** in Task Manager.
- UI is defined with Slint (`*.slint` files) and compiled at build time.
- Run the test suite:
  ```bash
  cargo test
  ```
- To modify the UI layout or add new menu actions, edit the relevant Slint files under `src/ui/` and update the event handling in `src/core/event_manager.rs`.

---

## Contributing
Contributions are welcome! Please:
1. Fork the repository.
2. Create a feature branch.
3. Ensure all tests pass (`cargo test`).
4. Submit a pull request with a clear description of the changes.

---

## License

[MIT License](LICENSE).

 *Created by Arif S.*
