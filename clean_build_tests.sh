pushd test_binaries/c
echo "===================================="
echo "making test/binaries/c"
echo "===================================="
make clean
make
popd

pushd test_binaries/basic_gpu
echo "===================================="
echo "making test/binaries/basic_gpu"
echo "===================================="
make clean
make
popd

pushd test_binaries/interrupts
echo "===================================="
echo "making test/binaries/interrupts"
echo "===================================="
make clean
make
popd

pushd test_binaries/texture_upload
echo "===================================="
echo "making test/binaries/texture_upload"
echo "===================================="
make clean
make
popd

pushd test_binaries/multi_hart
echo "===================================="
echo "making test/binaries/multi_hart"
echo "===================================="
make clean
make
popd

pushd test_binaries/direct_blit
echo "===================================="
echo "making test/binaries/direct_blit"
echo "===================================="
make clean
make
popd

pushd test_binaries/cutout_blit
echo "===================================="
echo "making test/binaries/cutout_blit"
echo "===================================="
make clean
make
popd

pushd test_binaries/basic_spu
echo "===================================="
echo "making test/binaries/basic_spu"
echo "===================================="
make clean
make
popd

pushd test_binaries/rust
echo "===================================="
echo "making test/binaries/rust"
echo "===================================="
pwd
make clean
make
popd

