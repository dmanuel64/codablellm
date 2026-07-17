[windows]
set shell := ["powershell.exe", "-NoLogo", "-Command"]
set no-cd

cargo := if os_family() == "windows" { "& " + quote(require("cargo")) } else { quote(require("cargo")) }
uv := if os_family() == "windows" { "& " + quote(require("uv")) } else { quote(require("uv")) }
docker := if os_family() == "windows" { "& " + quote(require("docker")) } else { quote(require("docker")) }
