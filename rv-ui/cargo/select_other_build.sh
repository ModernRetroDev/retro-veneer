#!/usr/bin/env bash

cargo_src=`cat Cargo.SOURCE`

if [ "$cargo_src" = 'other' ]; then
	echo 'OTHER build already selected; exiting now...'
	exit 0
fi
echo 'Switching build process over to OTHER build...'

cp cargo/other/* .