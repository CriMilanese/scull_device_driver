#!/bin/bash

GRAPH_NAME="rw_latency_"
RES_DIR="results"
BRANCH=$(git branch --show-current)

BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

if [[ $EUID != 0 ]];
then
    echo "${RED}Please run as root${NC}"
    exit
fi

pushd rust
make; make unload &>/dev/null; make load
popd

echo -e ${BLUE}"rust scull module loaded\n"${NC}

pushd c
make; make unload &>/dev/null; make load
popd

echo -e ${BLUE}"c scull module loaded\n"${NC}

fio jf.fio --output-format=json --output=${RES_DIR}"/"${GRAPH_NAME}${BRANCH}".json"
echo -e ${BLUE}"tested"${NC} ${RES_DIR}"/"${GRAPH_NAME}${BRANCH}
python3 fio-parser-plotter.py ${GRAPH_NAME}${BRANCH} ${RES_DIR}
echo -e ${BLUE}"plotted"${NC}

