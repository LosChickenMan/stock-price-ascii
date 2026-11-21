#!/bin/sh
set -eu

INSTALL_DIR="$HOME/.local/bin"
PROJECT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"

mkdir -p "$INSTALL_DIR"

cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml"

cat > "$INSTALL_DIR/stonks" <<SCRIPT
#!/bin/sh
cd "$PROJECT_DIR"
exec "$PROJECT_DIR/target/release/hardware-candlestick" "\$@"
SCRIPT

chmod +x "$INSTALL_DIR/stonks"

echo "Stonks installed! Run: stonks"
echo "Make sure $INSTALL_DIR is in your PATH."
