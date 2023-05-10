wget https://github.com/mirumirumi/ro-soku/releases/latest/download/ro-soku_x86_64_linux -P /tmp/
sudo install -Dp -m0755 /tmp/ro-soku_x86_64_linux /usr/local/bin/ro-soku
rm -rf /tmp/ro-soku_x86_64_linux
echo "ro-soku has been installed!"
