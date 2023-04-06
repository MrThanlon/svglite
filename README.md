# SVGLite

Rendering SVG with VGLite hardware.

## Build

### For K230

```shell
cargo build --target riscv64gc-unknown-linux-gnu --config target.riscv64gc-unknown-linux-gnu.linker=\"/path/to/T-Head_Xuantie_Toolchains/bin/riscv64-unknown-linux-gnu-gcc\"
```

## Compatibility

- Static SVG, no event or script
- No animation
- No stroke due to VGLite API (planed)
