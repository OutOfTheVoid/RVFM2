pushd tests/rust
echo "===================================="
echo "cleaning tests/rust"
echo "===================================="
make clean
popd

pushd tests/hello_triangle
echo "===================================="
echo "cleaning tests/hello_triangle"
echo "===================================="
make clean
popd

pushd tests/alloc_sgfx
echo "===================================="
echo "cleaning tests/alloc_sgfx"
echo "===================================="
make clean
popd
