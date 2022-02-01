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

EOF


if [ "${1:-install}" == "update" ]; then

cat << EOF
  ⚠️  UPDATE! ⚠️

  It looks like you got a new version of tmux-thumbs repository but
  the binary version is not in sync.

  We are going to proceed with the new installation.

  Do you want to continue?

  Press any key to continue...
EOF

else

cat << EOF
  It looks like this is the first time you are executing tmux-thumbs
  because the binary is not present.

  We are going to proceed with the installation. If you have Rust preinstalled, we will try to
  compile the binary from source. Otherwise, a prebuild binary for your platform will be used.

  Do you want to continue?

  Press any key to continue...
EOF

fi

read -rs -n 1

if ! [ -x "$(command -v cargo)" ]; then
  platform="$(uname -s) $(uname -m)"

  echo "  Rust is not installed! Trying to install ${platform} binary..."

  sources=$(curl -s "https://api.github.com/repos/fcsonline/tmux-thumbs/releases/latest" | grep browser_download_url)

  case $platform in
    "Darwin x86_64")
      url=$(echo "${sources}" | grep -o 'https://.*darwin.zip' | uniq)
      curl -sL "${url}" | bsdtar -xf - thumbs tmux-thumbs

      ;;
    "Linux x86_64")
      url=$(echo "${sources}" | grep -o 'https://.*linux-musl.tar.gz' | uniq)
      curl -sL "${url}" | tar -zxf - thumbs tmux-thumbs

      ;;
    *)
      echo "Unknown platform: $platform"
      exit 1
      ;;
  esac

  chmod +x thumbs tmux-thumbs
  mkdir -p target/release
  mv thumbs tmux-thumbs target/release
else
  echo '  Compiling tmux-thumbs, be patient:'
  cargo build --release --target-dir=target
fi

cat << EOF
  Installation complete! 💯

  Press any key to close this pane...
EOF

read -rs -n 1
