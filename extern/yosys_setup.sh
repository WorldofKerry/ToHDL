#!/bin/bash -xe

cd extern/
mkdir -p yosys && cd yosys
wget -nv https://github.com/YosysHQ/oss-cad-suite-build/releases/download/2023-08-02/oss-cad-suite-linux-x64-20230802.tgz
tar -xzf oss-cad-suite-linux-x64-20230802.tgz
