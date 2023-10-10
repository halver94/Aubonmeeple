#! /bin/sh

sudo apt-get -y install imagemagick

cd img
ls -1 *.png | xargs -n 1 bash -c 'convert "$0" "${0%.*}.jpg"'

cd ..
sudo -u postgres psql -a -f helpers/rename_image.sql