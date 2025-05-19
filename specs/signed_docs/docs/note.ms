sudo nixos-rebuild switch --upgrade

substituters = https://cache.nixos.org https://your-preferred-mirror
Replace https://your-preferred-mirror with a reliable mirror.



max-download-speed = 0
timeout = 300


curl -O https://cache.nixos.org/nar/1ag9x4bzzijqchbwcm3z00azbnn0slpyiya3kz5cv03ghqzw17na.nar.xz
sudo nix-store --add-fixed sha256 1ag9x4bzzijqchbwcm3z00azbnn0slpyiya3kz5cv03ghqzw17na.nar.xz

df -h

sudo nixos-rebuild switch --upgrade --show-trace