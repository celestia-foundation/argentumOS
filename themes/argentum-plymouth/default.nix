{ stdenv, lib }:

stdenv.mkDerivation {
  pname = "argentum-plymouth";
  version = "0.1.0";

  src = ./assets;

  dontBuild = true;

  installPhase = ''
    runHook preInstall
    install -dm755 $out/share/plymouth/themes/argentum
    cp -r $src/. $out/share/plymouth/themes/argentum/
    runHook postInstall
  '';

  meta = with lib; {
    description = "Plymouth boot splash theme for argentumOS";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}
