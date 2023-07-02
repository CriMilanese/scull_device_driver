#!/bin/bash

GRAPH_NAME="rw_latency_"
BRANCH=$(git branch --show-current)

if [[ $EUID != 0 ]];
then
    echo "Please run as root"
    exit
fi


pushd rust
make; make unload &>/dev/null; make load
popd

echo -e "rust scull module loaded\n"

pushd c
make; make unload &>/dev/null; make load
popd

echo -e "c scull module loaded\n"

echo $GRAPH_NAME$BRANCH
fio jf.fio --output-format=json --output=$GRAPH_NAME$BRANCH
python3 fio-parser-plotter.py $GRAPH_NAME$BRANCH
