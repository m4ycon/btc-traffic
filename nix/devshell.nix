{ inputs, ... }:
{
  perSystem = { config, self', pkgs, lib, ... }: {
    devShells.default = pkgs.mkShell {
      name = "btc-traffic-shell";
      inputsFrom = [
        self'.devShells.rust
        config.pre-commit.devShell # See ./nix/pre-commit.nix
      ] ++ config.dependencies;
      shellHook = config.hook;
      packages = with pkgs; [
        just
        nixd # Nix language server
        bacon
      ] ++ config.dependencies;
    };
  };
}
