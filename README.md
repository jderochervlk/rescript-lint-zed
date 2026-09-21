# rescript-zed

ReScript support for [Zed](https://zed.dev) editor.

This extension plugs in the following projects:

- [tree-sitter-rescript](https://github.com/rescript-lang/tree-sitter-rescript) parser
- [@rescript/language-server](https://github.com/rescript-lang/rescript-vscode) LSP
- [@jvlk/rescript-lint](https://github.com/jderochervlk/rescript-lint) diagnostics

## Installation (Development Extension)

This extension is not published in Zed's extension registry yet. For now, install
it from this repository using Zed's **Install Dev Extension** command.

1. Install [Zed](https://zed.dev) and [Rust via rustup](https://rustup.rs/).
2. Clone this repository:

   ```sh
   git clone https://github.com/jderochervlk/rescript-lint-zed.git
   ```

3. In Zed, run `zed: install dev extension` from the command palette, or open the
   Extensions page and click **Install Dev Extension**.
4. Select the cloned `rescript-lint-zed` directory. Zed builds the extension
   automatically.
5. Open a ReScript project to use the extension. Zed downloads
   `@jvlk/rescript-lint@0.1.0-alpha.1` automatically when no local
   `rescript-lint` executable is available.

This dev extension currently uses the same ID as the official ReScript extension,
so installing it overrides that extension. To update your development installation,
run `git pull` in the cloned repository, then click **Reinstall** on the extension
in Zed.

See Zed's [development extension documentation](https://zed.dev/docs/extensions/developing-extensions)
for more details.

## Settings

```json
{
  "lsp": {
    "rescript-language-server": {
      "initialization_options": {
        "extensionConfiguration": {
          "askToStartBuild": false
        }
      },
      "settings": {
        "version": "1.71.0-next-441959d.0"
      }
    },
    "rescript-lint": {
      "settings": {
        "version": "0.1.0-alpha.1"
      }
    }
  }
}
```

`initialization_options` are passed to the language server when it is started. They can be used to configure the language server. See [extensionConfiguration](https://github.com/rescript-lang/rescript-vscode/blob/441959d1feeaaffc1a589687758b1fbe1f649e72/server/src/config.ts#L5-L29)

`settings` are specific to the Zed extension.
With `version` you can point to a specific npm version of the [@rescript/language-server](https://www.npmjs.com/package/@rescript/language-server?activeTab=versions).
The `rescript-lint` server has an independent `version` setting for the
[@jvlk/rescript-lint](https://www.npmjs.com/package/@jvlk/rescript-lint) package.
The extension uses a `rescript-lint` executable from the worktree's `PATH` when
available and otherwise installs the configured npm version.

The default linter package is `@jvlk/rescript-lint@0.1.0-alpha.1`. See the
[contribution guide](CONTRIBUTING.md#using-a-local-language-server-build) to
use a local linter build instead.

This alpha requires Node.js 24 or newer and currently supports Linux glibc x64
and ARM64 only. See the [linter package documentation](https://www.npmjs.com/package/@jvlk/rescript-lint)
for current platform support.

## Developing

See [CONTRIBUTING.md](CONTRIBUTING.md) for instructions on how to develop this extension locally.

## Acknowledgements

This project was originally created by [humaans](https://github.com/humaans/). We're grateful for their initial work in bringing ReScript support to Zed.
