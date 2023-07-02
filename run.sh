#!/bin/bash

GRAPH_NAME="rw_latency_"
FIO_OUT_DIR="results/"
BRANCHES=$(git for-each-ref --format='%(refname:short)' refs/heads/)

GREEN='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

if [[ $EUID != 0 ]];
then
    echo "${RED}Please run as root${NC}"
    exit
fi

for branch in $BRANCHES;
do

  git checkout $branch

	pushd rust
	make; make unload &>/dev/null; make load
	popd

	echo -e "${GREEN}rust scull module loaded\n${NC}"

	pushd c
	make; make unload &>/dev/null; make load
	popd

	echo -e "${GREEN}c scull module loaded\n${NC}"

	fio jf.fio --output-format=json --output=$FIO_OUT_DIR$GRAPH_NAME$branch --minimal
	echo -e "${GREEN}tested ${NC}" $GRAPH_NAME$branch
	python3 fio-parser-plotter.py $GRAPH_NAME$branch
 	echo -e "${GREEN}plotted${NC}"
done

