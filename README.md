# rescript-zed

ReScript support for [Zed](https://zed.dev) editor.

This extension plugs in the following projects:

- [tree-sitter-rescript](https://github.com/rescript-lang/tree-sitter-rescript) parser
- [@rescript/language-server](https://github.com/rescript-lang/rescript-vscode) LSP
- [@jvlk/rescript-lint](https://github.com/jderochervlk/rescript-lint) diagnostics

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
        "version": "0.1.0-beta.1"
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

Until `@jvlk/rescript-lint` is published, use the local binary override in the
[contribution guide](CONTRIBUTING.md#using-a-local-language-server-build) when
testing the linter integration.

## Developing

See [CONTRIBUTING.md](CONTRIBUTING.md) for instructions on how to develop this extension locally.

## Acknowledgements

This project was originally created by [humaans](https://github.com/humaans/). We're grateful for their initial work in bringing ReScript support to Zed.
