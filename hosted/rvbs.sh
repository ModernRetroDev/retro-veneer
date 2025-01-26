#!/usr/bin/env bash
#------------------------------------------------------------------------------#
# RetroVeneer -- Install Bootstrap Script                                      #
#------------------------------------------------------------------------------#
http_base_loc='https://raw.githubusercontent.com/ModernRetroDev/retro-veneer/refs/heads/master/hosted'

function yes_or_no {
    while true; do
        read -p "$* [y/n]: " yn
        case $yn in
            [Yy]*) return 0  ;;  
            [Nn]*) return 1 ;;
        esac
    done
}

is_update='FALSE'
if [ "$1" = '--update' ]; then
    is_update='TRUE'
fi

if [ "$is_update" = 'FALSE' ]; then
    #==========================================================================#
cat << EOF
================================================================================
  I N S T A L L I N G    R E T R O    V E N E E R . . . . . . . . . . . .
================================================================================
This script will bootstrap the process of installing RetroVeneer on your system.

RetroVeneer is a set of scripts and/or programs which simplify the installation,
management, and updates for modern retro system emualtors on Raspberry Pi
systems.

To proceed with the installation you must first accept the software license.

EOF
read -p "Press Enter review the software license:" > /dev/null

clear
cat << EOF
================================================================================
Copyright (c) 2025 <Mike AKA: ModernRetroDev>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
================================================================================

Do you accept the RetroVeneer software license?
EOF

abort='TRUE'
yes_or_no "$message" && abort='FALSE'

if [ "$abort" = 'TRUE' ]; then
	echo 'Installation Aborted.'
	exit 1
fi

#------------------------------------------------------------------------------#
# Check if the system is likely running Raspberry Pi OS... if not; abort.      #
#------------------------------------------------------------------------------#
if [ ! -f '/etc/apt/sources.list.d/raspi.list' ]; then
    if [ ! "$YOLO" = '1' ]; then
        clear
        echo '#==============================================================================#'
        echo '#                                W A R N I N G                                 #'
        echo '#==============================================================================#'
        echo '#                                                                              #'
        echo '# This set of scripts/codes are designed and tested only to work on Raspberry  #'
        echo '# Pi Systems running Raspberry Pi OS.                                          #'
        echo '#                                                                              #'
        echo '#------------------------------------------------------------------------------#'
        echo '# Continuing with installation on an unsupported system will likely have some  #'
        echo '# undesired consequences and as such is discouraged.                           #'
        echo '#                                                                              #'
        echo '# If you still wish to continue in spite of this, you can proceed by setting   #'
        echo "# the environment variable 'YOLO' equal to '1', then re-run the command which  #"
        echo '# started this installer. If you are unfamiliar, this can be done by running a #'
        echo '# command like the following:                                                  #'
        echo '#                                                                              #'
        echo '#    export YOLO=1                                                             #'
        echo '#                                                                              #'
        echo '# I wish you the best of of luck...                                            #'
        echo '#==============================================================================#'
        exit 1
    fi
fi

    #==========================================================================#
fi

if [ "$is_update" = 'TRUE' ]; then
    rv_running="${HOME}/retroveneer/.temp/rv_is_running"
    until [ ! -f "$rv_running" ]; do
        echo "Waiting for RetroVeneer-UI to stop..."
        sleep 1
    done
fi

#------------------------------------------------------------------------------#
# Get and launch the retroveneer installer                                     #
#------------------------------------------------------------------------------#
thisArch=`uname -m`
supportedArch='FALSE'
if [ "$thisArch" = 'x86_64' ]; then
    supportedArch='TRUE'
fi
if [ "$thisArch" = 'aarch64' ]; then
    supportedArch='TRUE'
fi

if [ "$supportedArch" = 'FALSE' ]; then
        echo '#==============================================================================#'
        echo '#                                  E R R O R                                   #'
        echo '#==============================================================================#'
        echo '#                                                                              #'
        echo '# This is currently an unsupported architecture. Aborting installation...      #'
        echo '#                                                                              #'
        echo '#==============================================================================#'
        exit 1
fi

#------------------------------------------------------------------------------#
# Get and check the state of `install_freeze` before continuing.               #
#------------------------------------------------------------------------------#
tempdir=`mktemp -d`
remotepath="${http_base_loc}/install_freeze"
freezepath="${tempdir}/install_freeze"
wget -P "$tempdir" -q "$remotepath"
inst_frozen=`grep TRUE ${freezepath} | wc -l`
if [ "$inst_frozen" = '1' ]; then
        echo '#==============================================================================#'
        echo '#                                  E R R O R                                   #'
        echo '#==============================================================================#'
        echo '#                                                                              #'
        echo '# Installation Process Currently Frozen: Please try again later.               #'
        echo '#                                                                              #'
        echo '# This likely means that updates are being actively made to the install        #'
        echo '# process of RetroVeneer. Wait an hour or so and try to install again.         #'
        echo '#                                                                              #'
        echo '#==============================================================================#'
        exit 1
fi

appimagename="installer-${thisArch}.AppImage"
appimagepath="${tempdir}/${appimagename}"
remotepath="${http_base_loc}/${appimagename}"
echo "Downloading installer from \`${remotepath}\`..."

wget -P "$tempdir" -q "$remotepath" && \
	chmod +x $appimagepath && \
	$appimagepath

rm -f $appimagepath
rmdir $tempdir

#------------------------------------------------------------------------------#
# Launch RetroVeneer #
#------------------------------------------------------------------------------#
gio launch "${HOME}/.config/autostart/retroveneer.desktop"
