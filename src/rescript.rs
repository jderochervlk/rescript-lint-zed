use std::{borrow::Cow, env, fs};
use zed_extension_api::serde_json::Value;
use zed_extension_api::settings::LspSettings;
use zed_extension_api::{self as zed, Result};

const RESCRIPT_SERVER_ID: &str = "rescript-language-server";
const RESCRIPT_SERVER_PATH: &str = "node_modules/@rescript/language-server/out/cli.js";
const RESCRIPT_PACKAGE_NAME: &str = "@rescript/language-server";
const LINTER_SERVER_ID: &str = "rescript-lint";
const LINTER_SERVER_PATH: &str = "node_modules/@jvlk/rescript-lint/bin/rescript-lint.cjs";
const LINTER_PACKAGE_NAME: &str = "@jvlk/rescript-lint";
const LINTER_DEFAULT_VERSION: &str = "0.1.0-alpha.1";
const LINTER_BINARY_NAME: &str = "rescript-lint";

struct ReScriptExtension {
    did_find_rescript_server: bool,
    did_find_linter_server: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ServerKind {
    ReScript,
    Linter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VersionStrategy {
    Latest,
    Default(&'static str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PackageSpec {
    name: &'static str,
    script_path: &'static str,
    version_strategy: VersionStrategy,
}

#[derive(Debug, Default, Eq, PartialEq)]
struct Settings {
    version: Option<String>,
}

fn parse_settings(settings_value: Value) -> Settings {
    let version = settings_value
        .as_object()
        .and_then(|object| object.get("version"))
        .and_then(Value::as_str)
        .map(str::to_owned);

    Settings { version }
}

fn server_kind(server_id: &str) -> Result<ServerKind> {
    match server_id {
        RESCRIPT_SERVER_ID => Ok(ServerKind::ReScript),
        LINTER_SERVER_ID => Ok(ServerKind::Linter),
        _ => Err(format!("unknown language server '{server_id}'")),
    }
}

fn package_spec(kind: ServerKind) -> PackageSpec {
    match kind {
        ServerKind::ReScript => PackageSpec {
            name: RESCRIPT_PACKAGE_NAME,
            script_path: RESCRIPT_SERVER_PATH,
            version_strategy: VersionStrategy::Latest,
        },
        ServerKind::Linter => PackageSpec {
            name: LINTER_PACKAGE_NAME,
            script_path: LINTER_SERVER_PATH,
            version_strategy: VersionStrategy::Default(LINTER_DEFAULT_VERSION),
        },
    }
}

fn script_arguments(kind: ServerKind, script_path: String) -> Vec<String> {
    match kind {
        ServerKind::ReScript => vec![script_path, "--stdio".to_string()],
        ServerKind::Linter => vec![script_path, "lsp".to_string(), "--stdio".to_string()],
    }
}

fn direct_arguments(kind: ServerKind) -> Vec<String> {
    match kind {
        ServerKind::ReScript => vec!["--stdio".to_string()],
        ServerKind::Linter => vec!["lsp".to_string(), "--stdio".to_string()],
    }
}

fn initialization_options(kind: ServerKind) -> Option<Value> {
    match kind {
        ServerKind::ReScript => Some(zed::serde_json::json!({
            "extensionConfiguration": {
                "inlayHints": {
                    "enable": true
                },
                "codeLens": true,
                "signatureHelp": {
                    "enabled": true
                }
            }
        })),
        ServerKind::Linter => None,
    }
}

fn default_version(spec: PackageSpec) -> Result<String> {
    match spec.version_strategy {
        VersionStrategy::Latest => zed::npm_package_latest_version(spec.name),
        VersionStrategy::Default(version) => Ok(version.to_string()),
    }
}

fn configured_version(spec: PackageSpec, settings: Settings) -> Result<String> {
    match settings.version {
        Some(version) => Ok(version),
        None => default_version(spec),
    }
}

impl ReScriptExtension {
    fn server_exists(spec: PackageSpec) -> bool {
        fs::metadata(spec.script_path).is_ok_and(|stat| stat.is_file())
    }

    fn did_find_server(&self, kind: ServerKind) -> bool {
        match kind {
            ServerKind::ReScript => self.did_find_rescript_server,
            ServerKind::Linter => self.did_find_linter_server,
        }
    }

    fn mark_server_found(&mut self, kind: ServerKind) {
        match kind {
            ServerKind::ReScript => self.did_find_rescript_server = true,
            ServerKind::Linter => self.did_find_linter_server = true,
        }
    }

    fn get_lsp_settings_for_worktree(
        &mut self,
        server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Settings> {
        let settings = zed::settings::LspSettings::for_worktree(server_id.as_ref(), worktree);
        match settings {
            Err(_) => Ok(Settings::default()),
            Ok(LspSettings { settings: None, .. }) => Ok(Settings::default()),
            Ok(LspSettings {
                settings: Some(settings_value),
                ..
            }) => Ok(parse_settings(settings_value)),
        }
    }

    fn package_script_path(
        &mut self,
        kind: ServerKind,
        server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Cow<'static, str>> {
        let spec = package_spec(kind);
        let server_exists = Self::server_exists(spec);

        if self.did_find_server(kind) && server_exists {
            return Ok(spec.script_path.into());
        }

        zed::set_language_server_installation_status(
            server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let settings = self.get_lsp_settings_for_worktree(server_id, worktree)?;
        let version = configured_version(spec, settings)?;

        if !server_exists
            || zed::npm_package_installed_version(spec.name)?.as_ref() != Some(&version)
        {
            zed::set_language_server_installation_status(
                server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            let result = zed::npm_install_package(spec.name, &version);

            match result {
                Ok(()) => {
                    if !Self::server_exists(spec) {
                        Err(format!(
                            "installed package '{}' did not contain expected path '{}'",
                            spec.name, spec.script_path
                        ))?;
                    }
                }
                Err(error) => {
                    if !Self::server_exists(spec) {
                        Err(error)?;
                    }
                }
            }
        }

        self.mark_server_found(kind);

        Ok(spec.script_path.into())
    }
}

impl zed::Extension for ReScriptExtension {
    fn new() -> Self {
        Self {
            did_find_rescript_server: false,
            did_find_linter_server: false,
        }
    }

    fn language_server_command(
        &mut self,
        server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let kind = server_kind(server_id.as_ref())?;

        if kind == ServerKind::Linter {
            if let Some(command) = worktree.which(LINTER_BINARY_NAME) {
                return Ok(zed::Command {
                    command,
                    args: direct_arguments(kind),
                    env: Default::default(),
                });
            }
        }

        let server_path = self.package_script_path(kind, server_id, worktree)?;

        let current_dir =
            env::current_dir().map_err(|e| format!("failed to get current directory: {e}"))?;

        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: script_arguments(
                kind,
                current_dir
                    .join(server_path.as_ref())
                    .to_string_lossy()
                    .to_string(),
            ),
            env: Default::default(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        server_kind(server_id.as_ref()).map(initialization_options)
    }
}

zed::register_extension!(ReScriptExtension);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_an_independent_package_version() {
        let settings = parse_settings(zed::serde_json::json!({
            "version": "0.1.0-beta.2"
        }));

        assert_eq!(
            settings,
            Settings {
                version: Some("0.1.0-beta.2".to_string())
            }
        );
    }

    #[test]
    fn ignores_invalid_package_settings() {
        assert_eq!(
            parse_settings(zed::serde_json::json!({ "version": 1 })),
            Settings::default()
        );
        assert_eq!(
            parse_settings(zed::serde_json::json!(null)),
            Settings::default()
        );
    }

    #[test]
    fn selects_each_registered_server() {
        assert_eq!(server_kind(RESCRIPT_SERVER_ID), Ok(ServerKind::ReScript));
        assert_eq!(server_kind(LINTER_SERVER_ID), Ok(ServerKind::Linter));
        assert_eq!(
            server_kind("unknown"),
            Err("unknown language server 'unknown'".to_string())
        );
    }

    #[test]
    fn keeps_server_packages_and_versions_independent() {
        assert_eq!(
            package_spec(ServerKind::ReScript),
            PackageSpec {
                name: RESCRIPT_PACKAGE_NAME,
                script_path: RESCRIPT_SERVER_PATH,
                version_strategy: VersionStrategy::Latest,
            }
        );
        assert_eq!(
            package_spec(ServerKind::Linter),
            PackageSpec {
                name: LINTER_PACKAGE_NAME,
                script_path: LINTER_SERVER_PATH,
                version_strategy: VersionStrategy::Default(LINTER_DEFAULT_VERSION),
            }
        );
    }

    #[test]
    fn uses_the_pinned_linter_version_unless_configured() {
        let spec = package_spec(ServerKind::Linter);

        assert_eq!(
            configured_version(spec, Settings::default()),
            Ok(LINTER_DEFAULT_VERSION.to_string())
        );
        assert_eq!(
            configured_version(
                spec,
                Settings {
                    version: Some("0.1.0-beta.2".to_string())
                }
            ),
            Ok("0.1.0-beta.2".to_string())
        );
    }

    #[test]
    fn builds_server_specific_arguments() {
        assert_eq!(
            script_arguments(ServerKind::ReScript, "server.js".to_string()),
            vec!["server.js", "--stdio"]
        );
        assert_eq!(
            script_arguments(ServerKind::Linter, "linter.mjs".to_string()),
            vec!["linter.mjs", "lsp", "--stdio"]
        );
        assert_eq!(direct_arguments(ServerKind::Linter), vec!["lsp", "--stdio"]);
    }

    #[test]
    fn only_the_compiler_server_receives_initialization_options() {
        assert!(initialization_options(ServerKind::ReScript).is_some());
        assert_eq!(initialization_options(ServerKind::Linter), None);
    }
}
