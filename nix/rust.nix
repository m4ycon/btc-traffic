{ inputs, ... }:
{
  imports = [
    inputs.rust-flake.flakeModules.default
    inputs.rust-flake.flakeModules.nixpkgs
  ];
  perSystem = { config, self', pkgs, lib, ... }: {
    rust-project.crates."btc-traffic".crane.args = {
      preBuild = config.hook;
      doCheck = false;
      buildInputs = config.dependencies;
    };
    packages.default = self'.packages.btc-traffic;
  };
}
