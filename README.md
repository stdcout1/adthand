# Adthand
Simple dameon and client for prayer tracking. Multiple consumers can use dameon to acess prayer time information and edit system adthan settings.

Consists of a dameon, client, and some prayer util functions. They may be extracted into a external library in the future.

## Building and development:

Handled with nix. Run `nix build` to build and `nix develop` to get into a development shell. Furthermore, you can include this repo as an input and acess the package like that.

## Todo:
- Fix todo tags in code
- Add interface in client to get the next prayer & all prayer list (helpful in my waybar script)
- Add audio? 
