#!/usr/bin/env bash

ThisArch=`uname -m`

DesktopFile='./AppDir/installer.desktop'
AppName='installer'
AppVersion='0.0.1'

if [ "$ThisArch" = 'aarch64' ]; then
	bash cargo/select_aarch64_build.sh
else
	bash cargo/select_other_build.sh
fi

#------------------------------------------------------------------------------#
# Compile a release version of the code                                        #
#------------------------------------------------------------------------------#
cargo build --release

#------------------------------------------------------------------------------#
# Prepare `AppDir` directory for packaging                                     #
#------------------------------------------------------------------------------#
rm -rf ./AppDir || true
mkdir -p ./AppDir/usr/bin
cp target/release/installer ./AppDir/usr/bin
cp ../retroveneer.png ./AppDir

#------------------------------------------------------------------------------#
# Create a .desktop file                                                       #
#------------------------------------------------------------------------------#
echo '[Desktop Entry]'                  >  $DesktopFile
echo "X-AppImage-Arch=${ThisArch}"      >> $DesktopFile
echo "X-AppImage-Version=${AppVersion}" >> $DesktopFile
echo "X-AppImage-Name=${AppName}"       >> $DesktopFile
echo "Name=${AppName}"                  >> $DesktopFile
echo 'Path=/usr/bin'                    >> $DesktopFile
echo 'Exec=installer'                   >> $DesktopFile
echo 'Icon=retroveneer'                 >> $DesktopFile
echo 'Type=Application'                 >> $DesktopFile
echo 'Terminal=false'                   >> $DesktopFile
echo 'Categories=Utility;'              >> $DesktopFile
echo 'Comment=Installs/updates a RetroVeneer instance.' >> $DesktopFile

#------------------------------------------------------------------------------#
# Create an AppRun script                                                      #
#------------------------------------------------------------------------------#
linuxdeploy --appdir=./AppDir \
	--executable=target/release/installer \
	--desktop-file=$DesktopFile \
	--icon-file=../hosted/retroveneer.png \
	--output=appimage
