# build the release zip
./release.sh

version=$(sed 's/version = "\([0-9.]\{1,\}\)"/\1/;t;d' Cargo.toml | head -1)

# deploy on dystroy.org
rm -rf ~/dev/www/dystroy/backdown/download/*
cp -r build/* ~/dev/www/dystroy/backdown/download/
cp "backdown_$version.zip"  ~/dev/www/dystroy/backdown/download/
~/dev/www/dystroy/deploy.sh
