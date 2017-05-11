echo "This is for building pbr-demo-linux and pbr-demo-windows.exe";
echo "If you just want to build + run the project on your current platform, use: \"cargo run --release -- [arguments]\"";
echo "";

echo "Building for windows...";
cargo build --release --target=x86_64-pc-windows-gnu;
cp ./target/x86_64-pc-windows-gnu/release/pbr-demo.exe ./pbr-demo-windows.exe;
echo "Building for linux...";
cargo build --release --target=x86_64-unknown-linux-gnu;
cp ./target/x86_64-unknown-linux-gnu/release/pbr-demo ./pbr-demo-linux;