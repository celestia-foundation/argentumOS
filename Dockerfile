FROM nixos/nix:latest

RUN mkdir -p /etc/nix \
 && echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf \
 && echo "sandbox = false" >> /etc/nix/nix.conf

# Base image already ships git-minimal, bash, coreutils — installing the
# full git package collides on git-merge-file. Only add what's missing.
RUN nix-env --install --attr nixpkgs.qemu nixpkgs.OVMF

WORKDIR /workspace
COPY . /workspace

RUN chmod +x /workspace/scripts/*.sh || true

ENTRYPOINT ["/workspace/scripts/build.sh"]
CMD ["toplevel"]
