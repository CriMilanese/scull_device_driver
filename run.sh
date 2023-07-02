#!/bin/bash

GRAPH_NAME="rw_latency_"
RES_DIR="results"
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
  # skip devel branch
<<<<<<< Updated upstream
  if [[ $branch -ne "nocursor" ]];
=======
  if ! [ $branch == "nocursor" ];
>>>>>>> Stashed changes
	then
	  echo $branch
		continue
  fi

  echo $branch
  git checkout $branch

	pushd rust
	make; make unload &>/dev/null; make load
	popd

	echo -e "${GREEN}rust scull module loaded\n${NC}"

	pushd c
	make; make unload &>/dev/null; make load
	popd

	echo -e "${GREEN}c scull module loaded\n${NC}"

<<<<<<< Updated upstream
	fio jf.fio --output-format=json --output=$RES_DIR"/"$GRAPH_NAME$branch --minimal
	echo -e "${GREEN}tested ${NC}" $RES_DIR"/"$GRAPH_NAME$branch
	python3 fio-parser-plotter.py $GRAPH_NAME$branch $RES_DIR
=======
	fio jf.fio --output-format=json --output=$RES_DIR"/"$GRAPH_NAME$branch".json"
	echo -e "${GREEN}tested ${NC}" $RES_DIR"/"$GRAPH_NAME$branch
	python3 fio-parser-plotter.py $GRAPH_NAME$branch".json" $RES_DIR
>>>>>>> Stashed changes
 	echo -e "${GREEN}plotted${NC}"
 	exit
done

