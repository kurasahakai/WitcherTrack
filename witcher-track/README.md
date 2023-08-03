#

```powershell
# Install cargo-vcpkg
cargo install cargo-vcpkg
# Build vcpkg dependencies: tesseract and leptonica
cargo vcpkg -v build
# Build artifacts
cargo build --release
```

If `cargo vcpkg -v build` fails because www.nasm.us is down, navigate to `$env:VCPKG_ROOT` and run:
```
iwr https://github.com/microsoft/vcpkg/files/12073957/nasm-2.16.01-win64.zip -OutFile downloads/nasm-2.16.01-win64.zip
```
