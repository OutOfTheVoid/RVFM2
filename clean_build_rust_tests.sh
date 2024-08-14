pushd tests/rust
echo "===================================="
echo "making tests/rust"
echo "===================================="
make clean
make
popd


pushd tests/hello_triangle
echo "===================================="
echo "making tests/hello_triangle"
echo "===================================="
make clean
make
popd

pushd tests/alloc_sgfx
echo "===================================="
echo "making tests/alloc_sgfx"
echo "===================================="
make clean
make
popd
