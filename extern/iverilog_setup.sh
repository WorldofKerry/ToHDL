#!/bin/bash -xe

# From https://github.com/steveicarus/iverilog/actions/runs/6154732831/workflow
cd extern/iverilog/
sudo apt update -qq
sudo apt install -y make g++ git bison flex gperf libreadline-dev autoconf python3-sphinx python3-docopt
autoconf
./configure
make -j$(nproc) check
sudo make install
