Enciclopedia of liquid assets
===

Enciclopedia is a liquid asset repository web-client side, written in Rust.


A demo version is public available [here](https://enciclopedia.lvaccaro.com).

The list of full assets is available at [Esplora](https://blockstream.info/liquid/assets).

### Build
Update registry assets
```bash
curl https://github.com/Blockstream/asset_registry_db/raw/master/icons.json  -L > assets/liquid_icons.json
curl https://github.com/Blockstream/asset_registry_db/raw/master/index.json  -L > assets/liquid_assets.json
curl https://github.com/Blockstream/asset_registry_db/raw/master/index.minimal.json  -L > assets/liquid_assets_minimal.json
```
Generate wasm/html files
```bash
trunk build --release
```
Run on test environment
```bash
trunk serve
```
