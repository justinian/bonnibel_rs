# Bonnibel, the jsix OS build tool

Bonnibel (formerly _Popcorn Build_) is a tool for automating several of the
tasks common to jsix's development:

## Ninja build file generation

Command: `pb init`

Bonnibel's main purpose is [Ninja][] build file generation. It takes a fairly
hands-off approach, with the project specifying most of the templates for the
build files. The two main things that Bonnibel attempts to help with are:

* Modularization - Code is grouped into a set of modules, either libraries or
  executables, and may expose include directories. Other modules depending on a
  module will automatically use the exported include directories and link the
  produced libraries.

* Multiple target platforms - Modules may or may not explicitly target a
  specific platform. Platforms can have their own set of build configuration.
  For a given platform, Bonnibel will build all modules explicitly targeting
  that platform plus all their dependencies.

Bonnibel takes a YAML file describing a project, and generates the necessary
Ninja build files to build all the described modules for their necessary target
platforms.

For example, when building the jsix kernel, the build machine needs to build a
`makerd` tool to build the initial ramdisk. `makerd` depends on the `initrd`
library that defines the ramdisk structure. The kernel also depends on the same
library, but in a completely different build environment. Thus the `initrd`
library module will be built for both the kernel's target platform, and for the
build machine's native platform.

[Ninja]: https://ninja-build.org/


## External dependency syncing

Command: `pb sync`, `pb override`

Bonnibel can sync prebuilt versions of external dependencies. These need not be
libraries, code, or any other type of file. Bonnibel thinks of them merely as
project directory _overlays_. For example, while it is possible to manually
build the custom LLVM sysroot for building the jsix kernel, the build process
requires large source downloads and a lengthy build process. Since this rarely
changes, the jsix project default is to sync a pre-built package. Bonnibel
eases this process by automatically syncing and caching these dependencies for
you.

It is also possible to override this behavior and tell bonnibel that you want
to use your own version of any of these dependencies for doing local
development or testing on those items.


## Running the generated build

Command: `pb build`

This is a convenience function which will run the `init` and `sync` steps if
needed, then call out to Ninja to run the build.


## Configuration file

Bonnibel reads a YAML file called `bonnibel.yaml` in the project root directory.
