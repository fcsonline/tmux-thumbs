#!/usr/bin/env bash
set -Eeu -o pipefail

# Removing the binary to make this script idempotent
rm -rf target/release/thumbs

clear

cat << EOF

  █████                                                       █████    █████                                 █████
 ░░███                                                       ░░███    ░░███                                 ░░███
 ███████   █████████████   █████ ████ █████ █████            ███████   ░███████   █████ ████ █████████████   ░███████   █████
░░░███░   ░░███░░███░░███ ░░███ ░███ ░░███ ░░███  ██████████░░░███░    ░███░░███ ░░███ ░███ ░░███░░███░░███  ░███░░███ ███░░
  ░███     ░███ ░███ ░███  ░███ ░███  ░░░█████░  ░░░░░░░░░░   ░███     ░███ ░███  ░███ ░███  ░███ ░███ ░███  ░███ ░███░░█████
  ░███ ███ ░███ ░███ ░███  ░███ ░███   ███░░░███              ░███ ███ ░███ ░███  ░███ ░███  ░███ ░███ ░███  ░███ ░███ ░░░░███
  ░░█████  █████░███ █████ ░░████████ █████ █████             ░░█████  ████ █████ ░░████████ █████░███ █████ ████████  ██████
   ░░░░░  ░░░░░ ░░░ ░░░░░   ░░░░░░░░ ░░░░░ ░░░░░               ░░░░░  ░░░░ ░░░░░   ░░░░░░░░ ░░░░░ ░░░ ░░░░░ ░░░░░░░░  ░░░░░░



It looks like this is the first time you are executing tmux-thumbs
because the binary is not present.

We are going to proceed with the installation. If you have Rust preinstalled, we will try to 
compile the binary from source. Otherwise, a prebuild binary for your platform will be used.

Do you want to continue?

Press any key to continue...
EOF

read -s -n 1

if ! [ -x "$(command -v cargo)" ]; then
  system=$(uname -s)

  echo "Rust is not installed! Trying to install ${system} binary..."

  sources=$(curl -s "https://api.github.com/repos/fcsonline/tmux-thumbs/releases/latest" | grep browser_download_url)

  case $system in
    Darwin )
      url=$(echo $sources | grep -o 'https://.*darwin.zip' | uniq)
      curl -L ${url} | bsdtar -xf - thumbs

      break;;
    Linux )
      url=$(echo $sources | grep -o 'https://.*linux-musl.tar.gz' | uniq)
      curl -L ${url} | tar -zxf - thumbs

      break;;
    *)
      echo "Unknown system"
      exit 1
      break;;
  esac

  chmod +x thumbs
  mkdir -p target/release
  mv thumbs target/release/thumbs
else
  echo 'Compiling tmux-thumbs, be patient:'
  cargo build --release --target-dir=target
fi

cat << EOF
Installation complete! 💯

Press any key to close this pane...
EOF

read -s -n 1