# Alex' key toolkit

An open source tool for Windows 95, 98 and XP licensing, inspired by the [UMSKT project](https://github.com/UMSKT/UMSKT/).

For research and experimentation use only.

## Features
- Supports `keys.json` from UMSKT
- Read `%SYSTEMROOT%\System32\pidgen.dll`, extract and parse the contained BINK resources, in order to identify the correct keys for your installation
- Generate confirmation IDs using hyperelliptic curve math
- Generate PIDGEN v3 compatible license keys using elliptic curve math (used in Windows XP, ME)
- Generate PIDGEN v2 compatible license keys with a matching modulo 7 check digit (used in Windows 95, 98, Office 2000)
- (coming soon) Automatically complete the activation flow in Windows XP:
    - reading the product ID and BINK resource(s) from the OS
    - generating a license key and installing it
    - asking the OS to generate an installation ID
    - generating a matching confirmation ID and installing it
