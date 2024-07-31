pushd tests/c
echo "===================================="
echo "making tests/c"
echo "===================================="
make clean
make
popd

pushd tests/basic_gpu
echo "===================================="
echo "making tests/basic_gpu"
echo "===================================="
make clean
make
popd

pushd tests/interrupts
echo "===================================="
echo "making tests/interrupts"
echo "===================================="
make clean
make
popd

pushd tests/texture_upload
echo "===================================="
echo "making tests/texture_upload"
echo "===================================="
make clean
make
popd

pushd tests/multi_hart
echo "===================================="
echo "making tests/multi_hart"
echo "===================================="
make clean
make
popd

pushd tests/direct_blit
echo "===================================="
echo "making tests/direct_blit"
echo "===================================="
make clean
make
popd

pushd tests/cutout_blit
echo "===================================="
echo "making tests/cutout_blit"
echo "===================================="
make clean
make
popd

pushd tests/basic_spu
echo "===================================="
echo "making tests/basic_spu"
echo "===================================="
make clean
make
popd

pushd tests/rust
echo "===================================="
echo "making tests/rust"
echo "===================================="
pwd
make clean
make
popd

pushd tests/hello_triangle
echo "===================================="
echo "making tests/hello_triangle"
echo "===================================="
pwd
make clean
make
popd
