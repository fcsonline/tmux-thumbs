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

We are going to proceed with the installation. Remember that Rust is
a prerequisite to being able to build tmux-thumbs.

Do you want to continue?

Press any key to continue...
EOF

echo "Which format do you prefer for installation?"
echo "  1) Compile: will use cargo to compile tmux-thumbs."
echo "     If cargo isn't installed will use rustup to install it."
echo "  2) Download: will download a precompiled binary for your system."

select opt in "Compile" "Download"; do
  case $opt in
    Compile ) 
      if ! [ -x "$(command -v cargo)" ]; then
        echo 'Rust is not installed! ❌' >&2
        echo 'Press any key to install it' >&2

        read -s -n 1

        # This installation es provided by the official https://rustup.rs documentation
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
      fi

      echo 'Compiling tmux-thumbs, be patient:'
      cargo build --release --target-dir=target

      break;;
    Download ) 
      system=$(uname -s)
      case $system in
        Darwin )
          curl -L https://github.com/fcsonline/tmux-thumbs/releases/download/0.7.0/tmux-thumbs_0.7.0_x86_64-apple-darwin.zip | bsdtar -xf - thumbs

          break;;
        *)
          echo "Unknown system"

          break;;
      esac
      
      chmod +x thumbs
      mkdir -p target/release
      mv thumbs target/release/thumbs

      break;;
  esac
done

cat << EOF
Installation complete! 💯

Press any key to close this pane...
EOF

read -s -n 1
