pushd tests/hello_triangle
echo "===================================="
echo "making tests/hello_triangle"
echo "===================================="
make 
popd

pushd tests/alloc_sgfx
echo "===================================="
echo "making tests/alloc_sgfx"
echo "===================================="
make 
popd
