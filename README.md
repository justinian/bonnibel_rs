# Bonnibel

Bonnibel (`pb` for short) builds [Ninja][] build files for a set of modules.
Bonnibel is a thin wrapper around [Tera][] templates defining both _modules_
to be built and the _targets_ to build them for. This allows expressive power
in using Ninja's simple but otherwise very powerful build definition language.

For example, Bonnibel was created for [jsix][], where I needed the ability
to build tools for my local native environment, use those tools to make output
files used by other build stages, and build the bootloader, kernel and
user-space applications all with different compiler and linker options.
Complicating things even more, several libraries need to be built and used by
applications on multiple target environments.

[Ninja]: https://ninja-build.org
[Tera]: https://tera.netlify.com/
[jsix]: https://github.com/justinian/jsix

## Install

If you have Rust and cargo installed, then you can `cargo install bonnibel`.
Otherwise, please see the latest [prebuilt release][releases].

[releases]: https://github.com/justinian/bonnibel/releases

## Usage

```Bonnibel 2.0.0
Justin C. Miller <justin@devjustinian.com>
Bonnibel, the jsix OS build tool

USAGE:
    pb [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help         Prints help information
    -V, --version      Prints version information
    -v, --verbosity    Pass many times for more log output

OPTIONS:
    -d, --dir <build_dir>       The build directory to use (default "build")
    -f, --file <config_file>    The modules file to read from (default "modules.yaml")

SUBCOMMANDS:
    build       Run the build via Ninja
    generate    Regenerate the build files
    help        Prints this message or the help of the given subcommand(s)
    init        Initialize the build directory and options
    sync        Synchronize external packages
```
