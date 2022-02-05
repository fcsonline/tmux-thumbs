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
  because the binary is not present. We are going to proceed with the
  installation.

  Do you want to continue?

  Press any key to continue...
EOF

fi

read -rs -n 1

cat << EOF

  Which format do you prefer for installation?

  1) Compile: will use cargo to compile tmux-thumbs. It requires Rust.
  2) Download: will download a precompiled binary for your system.

EOF

select opt in "Compile" "Download"; do
  case $opt in
    Compile|1)

      if ! [ -x "$(command -v cargo)" ]; then
        echo '❌ Rust is not installed!'
        exit 1
      fi

      echo '  Compiling tmux-thumbs, be patient:'
      cargo build --release --target-dir=target

      break;;
    Download|2)
      platform="$(uname -s)_$(uname -m)"

      echo "  Downloading ${platform} binary..."

      sources=$(curl -s "https://api.github.com/repos/fcsonline/tmux-thumbs/releases/latest" | grep browser_download_url)

      case $platform in
        Darwin_x86_64)
          url=$(echo "${sources}" | grep -o 'https://.*darwin.zip' | uniq)
          curl -sL "${url}" | bsdtar -xf - thumbs tmux-thumbs

          ;;
        Linux_x86_64)
          url=$(echo "${sources}" | grep -o 'https://.*linux-musl.tar.gz' | uniq)
          curl -sL "${url}" | tar -zxf - thumbs tmux-thumbs

          ;;
        *)
          echo "❌ Unknown platform: ${platform}"
          read -rs -n 1
          echo "  Press any key to close this pane..."
          exit 1
          ;;
      esac

      chmod +x thumbs tmux-thumbs
      mkdir -p target/release
      mv thumbs tmux-thumbs target/release

      break;;
    *)
      echo "❌ Ouh? Choose an available option."
  esac
done

cat << EOF
  Installation complete! 💯

  Press any key to close this pane...
EOF

read -rs -n 1
