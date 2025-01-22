#!/usr/bin/env bash

cargo_src=`cat Cargo.SOURCE`

if [ "$cargo_src" = 'stable' ]; then
	echo 'Stable build already selected; exiting now...'
	exit 0
fi
echo 'Switching build process over to stable build...'

cp cargo/stable/* .