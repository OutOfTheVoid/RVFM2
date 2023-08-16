pushd test_binaries/c
make clean
make
popd

pushd test_binaries/basic_gpu
make clean
make
popd

pushd test_binaries/interrupts
make clean
make
popd

pushd test_binaries/texture_upload
make clean
make
popd

pushd test_binaries/multi_hart
make clean
make
popd

pushd test_binaries/direct_blit
make clean
make
popd

pushd test_binaries/cutout_blit
make clean
make
popd
