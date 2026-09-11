# Launching a BepInEx-modded Unity game with an isolated profile directory

**Research note — 2026-09-10**
**Subject:** Void Crew (Steam AppID 1063420), BepInEx 5.4.x + UnityDoorstop 4.5.0, replicating what Thunderstore Mod Manager / r2modman does.

---

## Answer up front — the recipe

The whole trick is one sentence: **BepInEx derives its entire directory tree from the location of the preloader DLL that Doorstop was told to invoke.** Point Doorstop at `<profile>/BepInEx/core/BepInEx.Preloader.dll` and `plugins`, `config`, `patchers`, `cache` and `LogOutput.log` all move to `<profile>/BepInEx/` automatically. There is no "plugins path" setting anywhere.

**Windows (the main case):**

1. Copy three files from the BepInEx pack into the **game folder** (unavoidable — Doorstop is a DLL proxy, it must sit next to the exe): `winhttp.dll`, `doorstop_config.ini`, `.doorstop_version`.
2. Keep everything else (`BepInEx/core`, `BepInEx/plugins/<Author>-<Mod>/`, `BepInEx/config`) in **your profile dir**.
3. Launch:
   ```
   "C:\Program Files (x86)\Steam\Steam.exe" -applaunch 1063420 "--doorstop-enabled" "true" "--doorstop-target-assembly" "<PROFILE>\BepInEx\core\BepInEx.Preloader.dll"
   ```
   (or run `"<GAME>\Void Crew.exe"` directly with the same args and `cwd = <GAME>`).
4. To launch vanilla: `--doorstop-enable false` — note r2modman uses the **Doorstop 3** spelling here, which is a bug on Doorstop 4; the correct v4 spelling is `--doorstop-enabled false`.

**Linux / Proton** (Void Crew is Windows-only on Steam, so this is always Proton, never the native `run_bepinex.sh` path):

5. Same three files in the game folder, same args, plus Wine must be told to prefer the game-local `winhttp.dll`:
   `WINEDLLOVERRIDES="winhttp=n,b"` in the environment (r2modman *also* writes `"winhttp"="native,builtin"` into `<compatdata>/pfx/user.reg` under `[Software\\Wine\\DllOverrides]`).
6. Paths passed to Doorstop must be Wine-visible — r2modman prefixes them with `Z:` (e.g. `Z:/home/you/profiles/default/BepInEx/core/BepInEx.Preloader.dll`).

**Two things that will surprise you:** on Windows Doorstop reads **no `DOORSTOP_*` environment variables for configuration** — only the ini and CLI args (env-var config is the *nix build only). And if `doorstop_config.ini` is absent, Doorstop's built-in default is `enabled = FALSE`, so args-only works but ini-less-and-args-less silently does nothing.

---

## 1. UnityDoorstop

### What it is

Doorstop is the injector; BepInEx is only the thing Doorstop invokes. Repo: <https://github.com/NeighTools/UnityDoorstop>.

> Doorstop is a tool to execute managed .NET assemblies inside Unity as early as possible.
>
> — [`README.md`, NeighTools/UnityDoorstop](https://github.com/NeighTools/UnityDoorstop/blob/master/README.md)

The managed entry point Doorstop 4 calls is always `static void Doorstop.Entrypoint.Start()` in the target assembly.

### The files

| File | Platform | Role |
| --- | --- | --- |
| `winhttp.dll` | Windows | The Doorstop payload itself, built as a **proxy DLL** for `winhttp.dll`. Windows resolves it from the exe's directory before `System32`, so putting it next to the game exe gets Doorstop loaded into the process. Must live in the game folder. |
| `doorstop_config.ini` | Windows | Doorstop's config file. Read from the **game exe's directory** (see CWD note below). If it does not exist, Doorstop keeps its compiled-in defaults. |
| `libdoorstop.so` / `.dylib` + `run.sh` (`run_bepinex.sh` in BepInEx dists) | Linux / macOS | Native builds use `LD_PRELOAD` / `DYLD_INSERT_LIBRARIES` instead of a proxy DLL; the shell script parses CLI args, exports them as `DOORSTOP_*` env vars, and `exec`s the game. |
| `.doorstop_version` | all | A plain text file containing the Doorstop version, shipped in the Doorstop release zip and copied into BepInEx dists by `build.cake`. **Doorstop itself never reads it** — it exists so tools (r2modman) can tell v3 from v4. |

`.doorstop_version` in BepInExPack 5.4.2305 contains exactly:

```
4.5.0
```

### Where the ini is read from — CWD is normalized first

`src/windows/entrypoint.c` sets the working directory to the exe's folder *before* config load, so both `doorstop_config.ini` and any relative `target_assembly` resolve against the game exe directory, not whatever CWD the launcher had:

```c
bool_t fix_cwd() {
    char_t *app_path = program_path();
    char_t *app_dir = get_folder_name(app_path);
    bool_t fixed_cwd = FALSE;
    char_t *working_dir = get_working_dir();

    if (strcmpi(app_dir, working_dir) != 0) {
        fixed_cwd = TRUE;
        SetCurrentDirectory(app_dir);
    }
    ...
}
```
*(`src/windows/entrypoint.c`, NeighTools/UnityDoorstop @ master)*

### Complete Doorstop 4 configuration surface

The authoritative list is the `Config` struct in [`src/config/config.h`](https://github.com/NeighTools/UnityDoorstop/blob/master/src/config/config.h): `enabled`, `redirect_output_log`, `ignore_disabled_env`, `target_assembly`, `boot_config_override`, `mono_dll_search_path_override`, `mono_debug_enabled`, `mono_debug_suspend`, `mono_debug_address`, `clr_runtime_coreclr_path`, `clr_corlib_dir`. That is all of them — eleven.

The shipped ini template ([`assets/windows/doorstop_config.ini`](https://github.com/NeighTools/UnityDoorstop/blob/master/assets/windows/doorstop_config.ini)):

```ini
# General options for Unity Doorstop
[General]

# Enable Doorstop?
enabled=true

# Path to the assembly to load and execute
# NOTE: The entrypoint must be of format `static void Doorstop.Entrypoint.Start()`
target_assembly=Doorstop.dll

# If true, Unity's output log is redirected to <current folder>\output_log.txt
redirect_output_log=false

# Overrides the default boot.config file path
boot_config_override=

# If enabled, DOORSTOP_DISABLE env var value is ignored
# USE THIS ONLY WHEN ASKED TO OR YOU KNOW WHAT THIS MEANS
ignore_disable_switch=false


# Options specific to running under Unity Mono runtime
[UnityMono]

# Overrides default Mono DLL search path
# ...
# To specify multiple paths, separate them with semicolons (;)
dll_search_path_override=

# If true, Mono debugger server will be enabled
debug_enabled=false

# When debug_enabled is true, specifies the address to use for the debugger server
debug_address=127.0.0.1:10000

# If true and debug_enabled is true, Mono debugger server will suspend the game execution until a debugger is attached
debug_suspend=false

# Options sepcific to running under Il2Cpp runtime
[Il2Cpp]

# Path to coreclr.dll that contains the CoreCLR runtime
coreclr_path=

# Path to the directory containing the managed core libraries for CoreCLR (mscorlib, System, etc.)
corlib_dir=
```

And the exact ini keys the Windows binary looks up, from [`src/windows/config.c`](https://github.com/NeighTools/UnityDoorstop/blob/master/src/windows/config.c):

```c
static inline void init_config_file() {
    if (!file_exists(CONFIG_NAME))
        return;

    char_t *config_path = get_full_path(CONFIG_NAME);

    load_bool_file(config_path, TEXT("General"), TEXT("enabled"), TEXT("true"),
                   &config.enabled);
    load_bool_file(config_path, TEXT("General"), TEXT("ignore_disable_switch"),
                   TEXT("false"), &config.ignore_disabled_env);
    load_bool_file(config_path, TEXT("General"), TEXT("redirect_output_log"),
                   TEXT("false"), &config.redirect_output_log);
    load_path_file(config_path, TEXT("General"), TEXT("target_assembly"),
                   DEFAULT_TARGET_ASSEMBLY, &config.target_assembly);
    load_path_file(config_path, TEXT("General"), TEXT("boot_config_override"),
                   NULL, &config.boot_config_override);

    load_str_file(config_path, TEXT("UnityMono"),
                  TEXT("dll_search_path_override"), TEXT(""),
                  &config.mono_dll_search_path_override);
    load_bool_file(config_path, TEXT("UnityMono"), TEXT("debug_enabled"),
                   TEXT("false"), &config.mono_debug_enabled);
    load_bool_file(config_path, TEXT("UnityMono"), TEXT("debug_suspend"),
                   TEXT("false"), &config.mono_debug_suspend);
    load_str_file(config_path, TEXT("UnityMono"), TEXT("debug_address"),
                  TEXT("127.0.0.1:10000"), &config.mono_debug_address);

    load_path_file(config_path, TEXT("Il2Cpp"), TEXT("coreclr_path"), NULL,
                   &config.clr_runtime_coreclr_path);
    load_path_file(config_path, TEXT("Il2Cpp"), TEXT("corlib_dir"), NULL,
                   &config.clr_corlib_dir);

    free(config_path);
}
```

Note the guard on line 1: **no ini file → no ini values at all**, and the compiled default (from `src/config/common.c`) is `config.enabled = FALSE`.

### Doorstop 4 vs Doorstop 3 — the rename table

Straight from [`CHANGES.md`](https://github.com/NeighTools/UnityDoorstop/blob/master/CHANGES.md):

| UnityDoorstop 3.x | Doorstop 4 |
| --- | --- |
| `UnityDoorstop.enabled` | `General.enabled` |
| `UnityDoorstop.targetAssembly` | `General.target_assembly` |
| `UnityDoorstop.redirectOutputLog` | `General.redirect_output_log` |
| `UnityDoorstop.ignoreDisableSwitch` | `General.ignore_disable_switch` |
| `UnityDoorstop.dllSearchPathOverride` | `UnityMono.dll_search_path_override` |
| `MonoBackend.runtimeLib` | removed |
| `MonoBackend.configDir` | removed |
| `MonoBackend.corlibDir` | removed |
| `MonoBackend.debugEnabled` | `UnityMono.debug_enabled` |
| `MonoBackend.debugSuspend` | `UnityMono.debug_suspend` |
| `MonoBackend.debugAddress` | `UnityMono.debug_address` |

CLI arguments:

| UnityDoorstop 3.x | Doorstop 4 |
| --- | --- |
| `--doorstop-enable` | `--doorstop-enabled` |
| `--redirect-output-log` | `--doorstop-redirect-output-log` |
| `--doorstop-target` | `--doorstop-target-assembly` |
| `--doorstop-dll-search-override` | `--doorstop-mono-dll-search-path-override` |
| `--mono-runtime-lib` | removed |
| `--mono-config-dir` | removed |
| `--mono-corlib-dir` | removed |
| `--mono-debug-enabled` | `--doorstop-mono-debug-enabled` |
| `--mono-debug-suspend` | `--doorstop-mono-debug-suspend` |
| `--mono-debug-address` | `--doorstop-mono-debug-address` |

Also from `CHANGES.md`: the IL2CPP backend switched from embedded Mono to CoreCLR, adding `Il2Cpp.coreclr_path` and `Il2Cpp.corlib_dir` and deleting the whole `[MonoBackend]` section; and the entry point is now fixed at `Doorstop.Entrypoint.Start()` instead of being auto-discovered.

For reference, the Doorstop 3 ini (from tag `v3.4.0.0`, `docs/doorstop_config.ini`) used the flat `[UnityDoorstop]` / `[MonoBackend]` sections and camelCase keys shown in the left column above.

The Doorstop 3 Windows arg parser, verbatim from `Proxy/config.c` @ `v3.4.0.0`:

```c
PARSE_ARG(L"--doorstop-enable", config.enabled, load_bool_argv);
PARSE_ARG(L"--redirect-output-log", config.redirect_output_log, load_bool_argv);
PARSE_ARG(L"--doorstop-target", config.target_assembly, load_path_argv);
PARSE_ARG(L"--doorstop-dll-search-override", config.mono_dll_search_path_override, load_path_argv);
PARSE_ARG(L"--mono-runtime-lib", config.mono_lib_dir, load_path_argv);
PARSE_ARG(L"--mono-config-dir", config.mono_config_dir, load_path_argv);
PARSE_ARG(L"--mono-corlib-dir", config.mono_config_dir, load_path_argv);
PARSE_ARG(L"--mono-debug-enabled", config.mono_debug_enabled, load_bool_argv);
PARSE_ARG(L"--mono-debug-suspend", config.mono_debug_suspend, load_bool_argv);
PARSE_ARG(L"--mono-debug-address", config.mono_debug_address, load_string_argv);
```

### Which BepInEx ships which Doorstop

- **BepInEx 5.4.23.x (current v5-lts) ships Doorstop 4.5.0.** From `build.cake` in the `v5-lts` branch: `const string DOORSTOP_VER = "4.5.0";`, and the README credits *"[NeighTools/UnityDoorstop](https://github.com/NeighTools/UnityDoorstop) - 4.5.0 ([33dab9a](https://github.com/NeighTools/UnityDoorstop/commit/33dab9a6733862eb81869ff08431d9478b28784b))"*.
- **Confirmed empirically for Void Crew's actual pack:** `BepInEx-BepInExPack-5.4.2305` contains `BepInExPack/.doorstop_version` = `4.5.0`.
- Older BepInEx 5.4.x releases shipped Doorstop 3.x. The reliable runtime check is exactly what r2modman does — read `.doorstop_version` and branch on the major version. **Unconfirmed:** the precise BepInEx 5.4.x release at which the v3→v4 switch happened; I did not walk the full `build.cake` history.

---

## 2. Environment variables

**This is the most important correction to the premise of the question.** Doorstop's env-var *configuration* support is a property of the ***nix build only**.

### Doorstop 4, Linux/macOS — env vars ARE the config mechanism

[`src/nix/config.c`](https://github.com/NeighTools/UnityDoorstop/blob/master/src/nix/config.c), verbatim:

```c
void load_config() {
    get_env_bool("DOORSTOP_ENABLED", &config.enabled);
    get_env_bool("DOORSTOP_REDIRECT_OUTPUT_LOG", &config.redirect_output_log);
    get_env_bool("DOORSTOP_IGNORE_DISABLED_ENV", &config.ignore_disabled_env);
    get_env_bool("DOORSTOP_MONO_DEBUG_ENABLED", &config.mono_debug_enabled);
    get_env_bool("DOORSTOP_MONO_DEBUG_SUSPEND", &config.mono_debug_suspend);
    try_get_env("DOORSTOP_MONO_DEBUG_ADDRESS", TEXT("127.0.0.1:10000"),
                &config.mono_debug_address);
    get_env_path("DOORSTOP_TARGET_ASSEMBLY", &config.target_assembly);
    get_env_path("DOORSTOP_BOOT_CONFIG_OVERRIDE", &config.boot_config_override);
    try_get_env("DOORSTOP_MONO_DLL_SEARCH_PATH_OVERRIDE", TEXT(""),
                &config.mono_dll_search_path_override);
    get_env_path("DOORSTOP_CLR_RUNTIME_CORECLR_PATH",
                 &config.clr_runtime_coreclr_path);
    get_env_path("DOORSTOP_CLR_CORLIB_DIR", &config.clr_corlib_dir);
    ...
}
```

That is the complete, current list of eleven config env vars. Booleans are `"1"` / `"0"` — `get_env_bool` compares against those literals exactly, nothing else. Note the name is `DOORSTOP_ENABLED` (with the `D`), not `DOORSTOP_ENABLE`.

### Doorstop 4, Windows — env vars are NOT read for config

The *only* env var the Windows build reads is the kill switch. Whole function, from `src/windows/config.c`:

```c
static inline void init_env_vars() {
    char_t *disable_env = getenv(TEXT("DOORSTOP_DISABLE"));
    if (!config.ignore_disabled_env && disable_env != 0) {
        LOG("DOORSTOP_DISABLE is set! Disabling Doorstop!");
        config.enabled = FALSE;
    }
    shutenv(disable_env);
}
```

So on Windows: **setting `DOORSTOP_TARGET_ASSEMBLY` does nothing.** Use CLI args (or rewrite the ini). Doorstop 3 was the same — it only checked `DOORSTOP_DISABLE` and `DOORSTOP_INITIALIZED` (`Proxy/config.c:125`, `Proxy/main.c:47` @ `v3.4.0.0`).

### Env vars Doorstop *exports* (all platforms)

These are outputs, set by Doorstop for the injected assembly to read. From the README:

| Environment variable | Description |
| --- | --- |
| `DOORSTOP_INITIALIZED` | Always set to `TRUE`. Use to determine if your code is run via Doorstop. |
| `DOORSTOP_INVOKE_DLL_PATH` | Path to the assembly executed by Doorstop relative to the current working directory. |
| `DOORSTOP_PROCESS_PATH` | Path to the application executable where the injected assembly is run. |
| `DOORSTOP_MANAGED_FOLDER_DIR` | *UnityMono*: Path to the game's `Managed` folder. *Il2Cpp*: Path to CoreCLR's base class library folder. |
| `DOORSTOP_DLL_SEARCH_DIRS` | Paths where the runtime searchs assemblies from by default, separated by OS-specific separator (`;` on windows and `:` on *nix). |
| `DOORSTOP_MONO_LIB_PATH` | *Only on UnityMono*: Full path to the mono runtime library. |

Cross-checked against `src/bootstrap.c` (lines 19, 47–56, 213, 286–290) and `src/windows/entrypoint.c:138` — the README table is accurate and complete.

**These six are the actual interface between Doorstop and BepInEx.** `DOORSTOP_INVOKE_DLL_PATH` is what BepInEx uses to find its own root (see §4).

---

## 3. CLI arguments

Verbatim from `init_cmd_args()` in [`src/windows/config.c`](https://github.com/NeighTools/UnityDoorstop/blob/master/src/windows/config.c):

```c
    for (int i = 0; i < argc; i++) {
        PARSE_ARG(TEXT("--doorstop-enabled"), config.enabled, load_bool_argv);
        PARSE_ARG(TEXT("--doorstop-redirect-output-log"),
                  config.redirect_output_log, load_bool_argv);
        PARSE_ARG(TEXT("--doorstop-target-assembly"), config.target_assembly,
                  load_path_argv);
        PARSE_ARG(TEXT("--doorstop-boot-config-override"),
                  config.boot_config_override, load_path_argv);

        PARSE_ARG(TEXT("--doorstop-mono-dll-search-path-override"),
                  config.mono_dll_search_path_override, load_path_argv);
        PARSE_ARG(TEXT("--doorstop-mono-debug-enabled"),
                  config.mono_debug_enabled, load_bool_argv);
        PARSE_ARG(TEXT("--doorstop-mono-debug-suspend"),
                  config.mono_debug_suspend, load_bool_argv);
        PARSE_ARG(TEXT("--doorstop-mono-debug-address"),
                  config.mono_debug_address, load_str_argv);

        PARSE_ARG(TEXT("--doorstop-clr-corlib-dir"), config.clr_corlib_dir,
                  load_path_argv);
        PARSE_ARG(TEXT("--doorstop-clr-runtime-coreclr-path"),
                  config.clr_runtime_coreclr_path, load_path_argv);
    }
```

Ten arguments, all `--doorstop-*`, all taking exactly one following value. Booleans accept the literal strings `true` / `false` only (`load_bool_argv` does `STR_EQUAL(par, TEXT("true"))` / `TEXT("false")`, case-insensitively via `lstrcmpi`); anything else leaves the previous value untouched rather than erroring.

`load_path_argv` runs the value through `get_full_path()`, so relative paths are resolved **against the game exe directory** (because of `fix_cwd()`). Absolute paths are safest.

> ⚠️ **README bug:** the argument table in the Doorstop README has the descriptions for `--doorstop-clr-corlib-dir` and `--doorstop-clr-runtime-coreclr-path` swapped. The source above is authoritative: `--doorstop-clr-corlib-dir` → corlib **directory**, `--doorstop-clr-runtime-coreclr-path` → path to the coreclr **library**.

The *nix `run.sh` / `run_bepinex.sh` accepts the same arg names (plus legacy aliases `--doorstop_enabled` and `--doorstop_target_assembly` with underscores) and translates them into the `DOORSTOP_*` env vars listed in §2 before `exec`ing the game.

### Precedence

From `load_config()` in `src/windows/config.c`:

```c
void load_config() {
    init_config_defaults();
    init_config_file();
    init_cmd_args();
    init_env_vars();
}
```

Windows order, later wins: **compiled defaults → `doorstop_config.ini` → CLI args → `DOORSTOP_DISABLE`.**

So on Windows, **CLI args override the ini**, and `DOORSTOP_DISABLE` overrides everything (unless `ignore_disable_switch` / `--doorstop-...` is set — note there is *no* CLI arg for `ignore_disable_switch`; it is ini-only).

On *nix there is no ini at all in the native build — `run_bepinex.sh` parses args → exports env vars → `libdoorstop.so` reads env vars. One layer, no precedence question.

**Answering "does Doorstop 4 still read the ini when env vars are set?"** — On Windows: it reads the ini and env vars are irrelevant to config, so yes, always (if the file exists). On Linux/macOS: the native library reads env vars *only*; it never opens an ini. The two mechanisms never coexist in one binary.

### Steam and argument passthrough

r2modman launches via `Steam.exe -applaunch <appid> <args...>` and this demonstrably works — that is the entirety of its Windows launch path (see §5). Valve's own documentation of `-applaunch` argument forwarding is not something I could locate as a first-party page; the behaviour is documented only in community sources **[unverified — secondary source]**, e.g. [howtogeek](https://www.howtogeek.com/825209/add-command-line-arguments-to-steam-gog-epic-games-store/). The strongest primary evidence is r2modmanPlus's own source relying on it in production.

The other route is the game's **Steam launch options** with `%command%`, which r2modman's Help screen tells users to paste manually when they want to launch outside the manager (see §5).

---

## 4. How BepInEx decides where its folders live

### BepInEx 5 (Mono) — the crux

`BepInEx.Preloader/Entrypoint.cs`, `v5-lts`:

```cs
public static void PreloaderPreMain()
{
    PlatformUtils.SetPlatform();

    string bepinPath = Utility.ParentDirectory(Path.GetFullPath(EnvVars.DOORSTOP_INVOKE_DLL_PATH), 2);

    Paths.SetExecutablePath(EnvVars.DOORSTOP_PROCESS_PATH, bepinPath, EnvVars.DOORSTOP_MANAGED_FOLDER_DIR, EnvVars.DOORSTOP_DLL_SEARCH_DIRS);
    ...
}
```

`bepinPath` = **two directories up from the preloader DLL**. `<X>/BepInEx/core/BepInEx.Preloader.dll` → `<X>/BepInEx`. Everything else falls out of that, in `BepInEx/Paths.cs`:

```cs
internal static void SetExecutablePath(string executablePath, string bepinRootPath = null, string managedPath = null, string[] dllSearchPath = null)
{
    ExecutablePath = executablePath;
    ProcessName = Path.GetFileNameWithoutExtension(executablePath);

    GameRootPath = PlatformHelper.Is(Platform.MacOS)
        ? Utility.ParentDirectory(executablePath, 4)
        : Path.GetDirectoryName(executablePath);

    ManagedPath = managedPath ?? Utility.CombinePaths(GameRootPath, $"{ProcessName}_Data", "Managed");
    BepInExRootPath = bepinRootPath ?? Path.Combine(GameRootPath, "BepInEx");
    ConfigPath = Path.Combine(BepInExRootPath, "config");
    BepInExConfigPath = Path.Combine(ConfigPath, "BepInEx.cfg");
    PluginPath = Path.Combine(BepInExRootPath, "plugins");
    PatcherPluginPath = Path.Combine(BepInExRootPath, "patchers");
    BepInExAssemblyDirectory = Path.Combine(BepInExRootPath, "core");
    BepInExAssemblyPath = Path.Combine(BepInExAssemblyDirectory, $"{Assembly.GetExecutingAssembly().GetName().Name}.dll");
    CachePath = Path.Combine(BepInExRootPath, "cache");
    DllSearchPaths = (dllSearchPath ?? new string[0]).Concat(new[] { ManagedPath }).Distinct().ToArray();
}
```

Key consequences:

- `plugins`, `config`, `patchers`, `cache`, `core` are **all** unconditionally `BepInExRootPath + name`. There is no way to split them.
- `GameRootPath` comes from `DOORSTOP_PROCESS_PATH` — the game exe — and is completely independent of `BepInExRootPath`. This is exactly why a profile in `%APPDATA%` works while the game stays in `steamapps`.
- `LogOutput.log` is written into `BepInExRootPath`: `Logger.Listeners.Add(new DiskLogListener("LogOutput.log", ...))` in `BepInEx/Bootstrap/Chainloader.cs:128`, with rollover to `LogOutput.log.<n>` (`DiskLogListener.cs:59`).

### BepInEx 6 (master) — identical mechanism

`Runtimes/Unity/BepInEx.Unity.Mono.Preloader/UnityPreloaderRunner.cs:49`:

```cs
var bepinPath = Utility.ParentDirectory(Path.GetFullPath(EnvVars.DOORSTOP_INVOKE_DLL_PATH), 2);

Paths.SetExecutablePath(EnvVars.DOORSTOP_PROCESS_PATH,
                        bepinPath,
                        EnvVars.DOORSTOP_MANAGED_FOLDER_DIR,
                        true,
                        EnvVars.DOORSTOP_DLL_SEARCH_DIRS);
```

Same "parent directory, 2 levels" rule. `BepInEx.Core/Paths.cs` differs only by adding `GameDataPath` (with `Void Crew_Data` → `Data` fallback probing) and making `SetExecutablePath` public. The IL2CPP variant (`Runtimes/Unity/BepInEx.Unity.IL2CPP/UnityPreloadRunner.cs:18`) does the same thing.

### The `BEPINEX_*` env vars — what actually exists

I grepped both `v5-lts` and `master` for `BEPINEX_` and for `bepinex_target` / `--bepinex`. **There is no `bepinex_target` setting, no `--bepinex-*` CLI argument, and no `BEPINEX_PLUGINS` / `BEPINEX_CONFIGS` / `BEPINEX_PATCHERS` env var — in either BepInEx 5 or BepInEx 6.** Do not go looking for them; the mechanism is the preloader DLL path and nothing else.

The only `BEPINEX_*` env vars that exist, all in BepInEx 6 and all IL2CPP/CoreCLR-specific:

| Var | Where | Purpose |
| --- | --- | --- |
| `BEPINEX_GAME_ASSEMBLY_PATH` | `Runtimes/Unity/BepInEx.Unity.IL2CPP/Il2CppInteropManager.cs:121` | override path to `GameAssembly.dll` |
| `BEPINEX_PRELOADER_LOG` | `Runtimes/Unity/BepInEx.Unity.IL2CPP/DoorstopEntrypoint.cs:22` | silent-exception log filename |
| `BEPINEX_FAIL_FAST` | `Runtimes/Unity/BepInEx.Unity.IL2CPP/DoorstopEntrypoint.cs:55` | if unset, swallow preloader exceptions |

The `DOORSTOP_*` vars BepInEx consumes are declared in `BepInEx.Preloader/EnvVars.cs` (v5) / `BepInEx.Preloader.Core/EnvVars.cs` (v6): `DOORSTOP_INVOKE_DLL_PATH`, `DOORSTOP_MANAGED_FOLDER_DIR`, `DOORSTOP_PROCESS_PATH`, `DOORSTOP_DLL_SEARCH_DIRS` (v6 adds `DOORSTOP_MONO_LIB_PATH`).

### `BepInEx.cfg`

Lives at `<BepInExRootPath>/config/BepInEx.cfg`. It configures *behaviour*, never *paths*. The shipped BepInExPack 5.4.2305 file has these sections: `[Caching]` (`EnableAssemblyCache`), `[Chainloader]` (`HideManagerGameObject`), `[Harmony.Logger]` (`LogChannels`), `[Logging]` (`UnityLogListening`, `LogConsoleToUnityLog`), `[Logging.Console]` (`Enabled`, `PreventClose`, `ShiftJisEncoding`, `StandardOutType`, `LogLevels`), `[Logging.Disk]` (`WriteUnityLog`, `AppendLog`, `Enabled`, `LogLevels`), `[Preloader]` (`ApplyRuntimePatches`, `HarmonyBackend`, `DumpAssemblies`, `LoadDumpedAssemblies`, `BreakBeforeLoadAssemblies`), `[Preloader.Entrypoint]` (`Assembly`, `Type`, `Method`).

Notably the pack ships with `[Logging.Console] Enabled = true` and `[Logging.Disk] LogLevels = All`, both non-default — that's why a modded game shows a console window.

Since it lives inside the profile's `BepInEx/config/`, each profile gets its own console/logging settings for free.

### 5.x vs 6.x for Void Crew

Void Crew is Unity **Mono** and the community targets **BepInEx 5**. See §8.

---

## 5. How r2modman / Thunderstore Mod Manager actually does it

Source: <https://github.com/ebkr/r2modmanPlus> (`develop` @ `b0b9537`, 2026-09-09). TMM is the same codebase.

### The argument builder

`src/r2mm/launching/instructions/instructions/loader/BepInExGameInstructions.ts` — this is the whole thing:

```ts
public async generate(game: Game, profile: Profile): Promise<GameInstruction> {
    const doorstopVersion = await getUnityDoorstopVersion(profile);
    switch (doorstopVersion) {
        case 4: return this.genDoorstopV4(game, profile);
        default: return this.genDoorstopV3(game, profile);
    }
}

private async genDoorstopV3(game: Game, profile: Profile): Promise<GameInstruction> {

    const launchArgs: string[] = [
        '--doorstop-enable',
        'true',
        '--doorstop-target',
        DynamicGameInstruction.BEPINEX_PRELOADER_PATH
    ];

    if (["linux", "darwin"].includes(appWindow.getPlatform().toLowerCase())) {

        launchArgs.push(
            '--r2profile',
            DynamicGameInstruction.PROFILE_NAME
        );
        if (game.instanceType === GameInstanceType.SERVER) {
            launchArgs.push('--server');
        }
        if (await FsProvider.instance.exists(Profile.getActiveProfile().joinToProfilePath("unstripped_corlib"))) {
            launchArgs.push(
                '--doorstop-dll-search-override',
                DynamicGameInstruction.BEPINEX_CORLIBS
            );
        }
    }
    return {
        moddedParameterList: launchArgs,
        vanillaParameterList: ['--doorstop-enable', 'false']
    };
}

private async genDoorstopV4(game: Game, profile: Profile): Promise<GameInstruction> {

    const launchArgs: string[] = [
        '--doorstop-enabled',
        'true',
        '--doorstop-target-assembly',
        DynamicGameInstruction.BEPINEX_PRELOADER_PATH
    ];

    if (["linux", "darwin"].includes(appWindow.getPlatform().toLowerCase())) {
        launchArgs.push(
            '--r2profile',
            DynamicGameInstruction.PROFILE_NAME
        );
        if (game.instanceType === GameInstanceType.SERVER) {
            launchArgs.push('--server');
        }
        if (await FsProvider.instance.exists(Profile.getActiveProfile().joinToProfilePath("unstripped_corlib"))) {
            launchArgs.push(
                '--doorstop-mono-dll-search-path-override',
                DynamicGameInstruction.BEPINEX_CORLIBS
            );
        }
    }
    return {
        moddedParameterList: launchArgs,
        vanillaParameterList: ['--doorstop-enable', 'false']
    };
}
```

Observations:

- **On Windows the modded arg list is exactly four tokens.** That's it. No env vars, no ini rewriting.
- `--r2profile <name>` and `--server` are **not Doorstop arguments**; they are consumed and stripped by r2modman's own `linux_wrapper.sh` (see below). Doorstop ignores unrecognized args.
- The vanilla list uses `--doorstop-enable false` (Doorstop **3** spelling) in both branches — on Doorstop 4 this is silently ignored, so "start vanilla" relies on the game's own `doorstop_config.ini` still saying `enabled=true` but... actually it relies on the fact that the profile's preloader is never named, so Doorstop falls back to the ini's `target_assembly=BepInEx\core\BepInEx.Preloader.dll` **relative to the game folder**, where no `BepInEx/core` exists (the linker only copies root files, §6). Worth not replicating.

Version detection, `src/utils/UnityDoorstopUtils.ts`:

```ts
export async function getUnityDoorstopVersion(profile: Profile): Promise<number> {
    if (await FsProvider.instance.exists(profile.joinToProfilePath(".doorstop_version"))) {
        const dvContent = (await FsProvider.instance.readFile(profile.joinToProfilePath(".doorstop_version"))).toString();
        const majorVersion = Number(dvContent.split(".")[0]);
        if (majorVersion && majorVersion > 3) {
            return majorVersion;
        }
    }
    return 3;
}
```

### The `@bepInExPreloaderPath` placeholder

`src/r2mm/launching/instructions/DynamicGameInstruction.ts` defines `BEPINEX_PRELOADER_PATH = "@bepInExPreloaderPath"`; `GameInstructionParser.ts` resolves it:

```ts
private static async bepInExPreloaderPathResolver(game: Game, profile: Profile): Promise<string | R2Error> {
    try {
        if (["linux"].includes(appWindow.getPlatform().toLowerCase())) {
            const settings = await ManagerSettings.getSingleton(game);
            const isProton = await GameInstructionParser.isProton(game);
            const corePath = await FsProvider.instance.realpath(profile.joinToProfilePath("BepInEx", "core"));
            const preloaderPath = path.join(corePath,
                (await FsProvider.instance.readdir(corePath))
                    .filter((x: string) => ["BepInEx.Unity.Mono.Preloader.dll", "BepInEx.Unity.IL2CPP.dll", "BepInEx.Preloader.dll", "BepInEx.IL2CPP.dll", "BepInEx.NET.CoreCLR.dll"].includes(x))[0]!);
            return `${isProton ? 'Z:' : ''}${preloaderPath}`;
        } else {
            const corePath = profile.joinToProfilePath("BepInEx", "core");
            return path.join(corePath,
                (await FsProvider.instance.readdir(corePath))
                    .filter((x: string) => ["BepInEx.Unity.Mono.Preloader.dll", "BepInEx.Unity.IL2CPP.dll", "BepInEx.Preloader.dll", "BepInEx.IL2CPP.dll", "BepInEx.NET.CoreCLR.dll"].includes(x))[0]!);
        }
    } catch (e) { ... }
}
```

So the resolved path is `<profile>/BepInEx/core/<first matching preloader name>`, and under Proton it is prefixed with `Z:` so Wine can see the Linux path. The preloader-name candidate list, in priority order as filtered, is: `BepInEx.Unity.Mono.Preloader.dll` (BepInEx 6 Mono), `BepInEx.Unity.IL2CPP.dll` (BepInEx 6 IL2CPP), `BepInEx.Preloader.dll` (**BepInEx 5 — the Void Crew case**), `BepInEx.IL2CPP.dll` (older BepInEx 6), `BepInEx.NET.CoreCLR.dll`.

### Windows launch

`src/r2mm/launching/runners/windows/SteamGameRunner_Windows.ts`:

```ts
const mappedArgs = args.map(value => `"${value}"`).join(' ');

ChildProcess.exec(`"${steamDir}/Steam.exe" -applaunch ${game.activePlatform.storeIdentifier} ${mappedArgs} ${settings.getContext().gameSpecific.launchParameters}`, undefined, (err => { ... }));
```

Every argument token is individually double-quoted. For Void Crew (`storeIdentifier` = `1063420`) the resulting command is:

```
"C:/Program Files (x86)/Steam/Steam.exe" -applaunch 1063420 "--doorstop-enabled" "true" "--doorstop-target-assembly" "C:\Users\you\AppData\Roaming\r2modmanPlus-local\VoidCrew\profiles\Default\BepInEx\core\BepInEx.Preloader.dll" 
```

Nothing is set in the environment. No `steam_appid.txt` is written — **I grepped the whole repo for `steam_appid`; there are zero hits.** r2modman does not touch it.

### Direct (non-Steam) launch

`src/r2mm/launching/runners/multiplatform/DirectGameRunner.ts` — same args, run against the exe, with the game dir as CWD:

```ts
const childProcess = ChildProcess.exec(`"${gameExecutable}" ${mappedArgs} ${settings.getContext().gameSpecific.launchParameters}`, {
    cwd: gameDir,
    windowsHide: false,
}, ...);
```

### Linux launch — the Proton dance

`src/r2mm/launching/runners/linux/SteamGameRunner_Linux.ts`:

```ts
public async startModded(game: Game, profile: Profile): Promise<void | R2Error> {
    const settings = await ManagerSettings.getSingleton(game);
    const isProton = await getDeterminedLaunchType(game, settings.getLaunchType() || LaunchType.AUTO) === LaunchType.PROTON;

    let proxyArgs: Record<string, string> = {};

    if (isProton) {
        // BepInEx uses winhttp, GDWeave uses winmm. More can be added later.
        const proxyDll = game.packageLoader == PackageLoader.GDWEAVE ? "winmm" : "winhttp";
        const promise = await this.ensureWineWillLoadDllOverride(game, proxyDll);
        if (promise instanceof R2Error) {
            // We no longer want to display an error as launch args should be set correctly.
            // A console error still allows it to be discoverable.
            console.error(promise);
        }
        proxyArgs['WINEDLLOVERRIDES'] = `"${proxyDll}=n,b"`
    } else {
        // If sh files aren't executable then the wrapper will fail.
        ... chmod 0o755 every *.sh at the profile root ...
    }

    const args = await this.getGameArguments(game, profile);
    ...
    return this.start(game, args, proxyArgs);
}
```

and the registry edit it performs:

```ts
private async ensureWineWillLoadDllOverride(game: Game, proxyDll: string): Promise<void | R2Error>{
    const fs = FsProvider.instance;
    const compatDataDir = await (GameDirectoryResolverProvider.instance as LinuxGameDirectoryResolver).getCompatDataDirectory(game);
    if(compatDataDir instanceof R2Error)
        return compatDataDir;
    const userReg = path.join(compatDataDir, 'pfx', 'user.reg');
    const userRegData = (await fs.readFile(userReg)).toString();
    const ensuredUserRegData = this.regAddInSection(
        userRegData,
        "[Software\\\\Wine\\\\DllOverrides]",
        proxyDll,
        "native,builtin"
    );

    if(userRegData !== ensuredUserRegData){
        await fs.copyFile(userReg, path.join(path.dirname(userReg), 'user.reg.bak'));
        await fs.writeFile(userReg, ensuredUserRegData);
    }
}
```

i.e. it backs up `<compatdata>/<appid>/pfx/user.reg` and inserts `"winhttp"="native,builtin"` under `[Software\\Wine\\DllOverrides]`.

Then `start()`:

```ts
const executableNamePart = `"${steamExecutable}"`;          // = "<steamDir>/steam.sh"
const appLaunchPart = `-applaunch ${game.activePlatform.storeIdentifier}`;
const modLoaderArgumentsPart = args.map(value => `"${value}"`).join(' ');
const userDefinedArgsPart = `${settings.getContext().gameSpecific.launchParameters}`;

const executionParts = [
    executableNamePart,
    appLaunchPart,
    modLoaderArgumentsPart,
    userDefinedArgsPart,
];

const commandString = executionParts.join(" ").trim();
...
childProcess.execSync(
    commandString,
    {
        env: env,
        stdio: 'inherit'
    }
);
```

> **Important nuance:** `proxyArgs` (which holds `WINEDLLOVERRIDES`) is **never actually merged into the spawn environment**. I traced every use of `proxyArgs` in `src/` — it is populated at lines 43/74/75 of `SteamGameRunner_Linux.ts`, passed into `start()`, and then dropped on the floor; `execSync` receives `env: env`, the manager's own unmodified environment. Combined with the code comment *"We no longer want to display an error as launch args should be set correctly"*, the current design is: the `user.reg` edit does the real work, and the user is separately instructed to paste `WINEDLLOVERRIDES=...` into their Steam launch options. Do not assume r2modman injects that env var at spawn time — it doesn't.

### What r2modman tells the user to paste into Steam launch options

`src/components/computed/WrapperArguments.ts`:

```ts
export const WineDllOverridesValue = ref<string>(`WINEDLLOVERRIDES="winhttp,version=n,b"`);
```

```ts
function determineWrapperArguments() {
    ...
    let wrapperPath = "";
    if (isFlatpakExecutable.value) {
        wrapperPath = `${path.join(PathResolver.MOD_ROOT, 'web_start_wrapper.sh')}`
    } else {
        wrapperPath = path.join(PathResolver.MOD_ROOT, appWindow.getPlatform() === 'darwin' ? 'macos_proxy' : 'linux_wrapper.sh');
    }
    return `"${wrapperPath}" %command%`;
}
```

`src/pages/LinuxNativeGameSetup.vue` concatenates them: `WINEDLLOVERRIDES="winhttp,version=n,b" "<MOD_ROOT>/linux_wrapper.sh" %command%`.

Note this string overrides **both** `winhttp` and `version` — broader than the BepInEx docs' `winhttp.dll=n,b`.

And `src/pages/Help.vue` computes what a user should add to launch options to start modded outside the manager:

```ts
watchEffect(async () => {
    const loaderArgs = doorstopTarget.value;
    const prerequisiteText = ComputedWrapperLaunchArguments.value;
    if (appWindow.getPlatform() === 'win32') {
        launchArgs.value = loaderArgs;
        return;
    }
    const storedLaunchType = await getLaunchType(store.state.activeGame);
    const launchType = await getDeterminedLaunchType(store.state.activeGame, storedLaunchType);
    if (launchType === LaunchType.NATIVE) {
        launchArgs.value = `${prerequisiteText} ${loaderArgs}`;
    } else {
        launchArgs.value = `%command% ${loaderArgs}`;
    }
});
```

So:
- **Windows:** launch options = just the Doorstop args (Steam appends them after the exe).
- **Linux native:** `"<MOD_ROOT>/linux_wrapper.sh" %command% <doorstop args>`.
- **Linux Proton:** `%command% <doorstop args>` — i.e. args appended after the game command, exactly the Windows behaviour, since it's the Windows Doorstop running inside Wine. **This is the Void Crew case.**

### `linux_wrapper.sh` (native-Linux only, not used for Void Crew)

`public/linux_wrapper.sh`, verbatim tail:

```sh
if [ -z "$R2PROFILE" ]; then
    echo "[R2MODMAN LINUX WRAPPER] Launching vanilla!"
    exec "$@"
fi

[ -n "$R2STARTSERVER" ] && exec "$BASEDIR/profiles/$R2PROFILE/start_server_bepinex.sh" "$@" || true

if test -f "$BASEDIR/profiles/$R2PROFILE/run_bepinex.sh"; then
    exec "$BASEDIR/profiles/$R2PROFILE/run_bepinex.sh" "$@"
elif test -f "$BASEDIR/profiles/$R2PROFILE/run_umm.sh"; then
    exec "$BASEDIR/profiles/$R2PROFILE/run_umm.sh" "$@"
else
    exec "$BASEDIR/profiles/$R2PROFILE/start_game_bepinex.sh" "$@"
fi
```

It strips `--r2profile <name>` and `--server` from the arg list, then chain-loads the **profile's own** `run_bepinex.sh`, which is where the `LD_PRELOAD=libdoorstop.so` + `DOORSTOP_*` exports happen. That is why native Linux BepInEx needs **nothing copied into the game folder at all** (see `ModLinker.link` in §6).

### `run_bepinex.sh` (BepInEx's own, from the Linux/macOS dist)

The BepInEx `v5-lts` `doorstop/run_bepinex.sh` is Doorstop's `assets/nix/run.sh` with `target_assembly="BepInEx/core/BepInEx.Preloader.dll"` substituted for the default. Its operative section:

```sh
export DOORSTOP_ENABLED="$enabled"
export DOORSTOP_TARGET_ASSEMBLY="$target_assembly"
export DOORSTOP_BOOT_CONFIG_OVERRIDE="$boot_config_override"
export DOORSTOP_IGNORE_DISABLED_ENV="$ignore_disable_switch"
export DOORSTOP_MONO_DLL_SEARCH_PATH_OVERRIDE="$dll_search_path_override"
export DOORSTOP_MONO_DEBUG_ENABLED="$debug_enable"
export DOORSTOP_MONO_DEBUG_ADDRESS="$debug_address"
export DOORSTOP_MONO_DEBUG_SUSPEND="$debug_suspend"
export DOORSTOP_CLR_RUNTIME_CORECLR_PATH="$coreclr_path.$lib_extension"
export DOORSTOP_CLR_CORLIB_DIR="$corlib_dir"

# Final setup
doorstop_directory="${BASEDIR}/"
doorstop_name="libdoorstop.${lib_extension}"

export LD_LIBRARY_PATH="${doorstop_directory}:${corlib_dir}:${LD_LIBRARY_PATH}"
if [ -z "$LD_PRELOAD" ]; then
    export LD_PRELOAD="${doorstop_name}"
else
    export LD_PRELOAD="${doorstop_name}:${LD_PRELOAD}"
fi
```

It also contains a Steam-specific re-exec hack, worth knowing about if you ever do native Linux:

```sh
# Special case: program is launched via Steam on Linux
# In that case rerun the script via their bootstrapper to delay adding Doorstop to LD_PRELOAD
# This is required until https://github.com/NeighTools/UnityDoorstop/issues/88 is resolved
for a in "$@"; do
    if [ "$a" = "SteamLaunch" ]; then
        ...
```

and it explicitly refuses to run against a Windows exe:

```sh
    *PE32*)
        echo "The executable is a Windows executable file. You must use Wine/Proton and BepInEx for Windows with this executable." 1>&2
        echo "Uninstall BepInEx for *nix and install BepInEx for Windows instead." 1>&2
```

**This is exactly the Void Crew situation on Linux — Windows-only exe, so use the Windows BepInEx pack under Proton.**

---

## 6. Profile folder layout, and what must go in the game folder

### On-disk layout

`src/App.vue:120,128` and `src/model/game/GameManager.ts:54` and `src/model/Profile.ts`:

```
PathResolver.APPDATA_DIR = path.join(appData, 'r2modmanPlus-local');
PathResolver.ROOT        = settings.getContext().global.dataDirectory || PathResolver.APPDATA_DIR;
PathResolver.MOD_ROOT    = path.join(PathResolver.ROOT, game.internalFolderName);
// Profile.getRootDir()  = path.join(PathResolver.MOD_ROOT, 'profiles')
// profile.getProfilePath() = <that>/<profileName>
```

So for r2modman + Void Crew (`internalFolderName` = `VoidCrew`):

```
%APPDATA%\r2modmanPlus-local\
└── VoidCrew\
    ├── cache\<Author-Mod>\<version>\        <- extracted Thunderstore zips
    ├── linux_wrapper.sh                      (linux only)
    └── profiles\
        └── Default\
            ├── .doorstop_version             <- "4.5.0"
            ├── doorstop_config.ini
            ├── winhttp.dll
            ├── changelog.txt
            ├── mods.yml                      <- r2modman's own manifest, never copied
            └── BepInEx\
                ├── core\BepInEx.Preloader.dll, BepInEx.dll, 0Harmony.dll, Mono.Cecil*.dll, MonoMod.*.dll
                ├── config\BepInEx.cfg
                ├── patchers\
                └── plugins\
                    ├── NihilityShift-VoidManager\...
                    └── Jack_Modding-Quickstart\...
```

**TMM's data folder path** is `%APPDATA%\Thunderstore Mod Manager\DataFolder\<game>\profiles\<profile>\...` — the structure below the data-folder root is identical because it is the same code. **Unconfirmed:** the literal string `Thunderstore Mod Manager/DataFolder` does not appear in the open r2modmanPlus source; TMM overrides `PathResolver.APPDATA_DIR`/`ROOT` in its own (closed) wrapper. Treat the *shape* as confirmed and the *root name* as reported-not-verified.

### Thunderstore zip → profile

Void Crew's install rules, from `src/assets/data/ecosystem.json` (`games["void-crew"].r2modman[0].installRules`):

```json
"installRules": [
  { "route": "BepInEx/plugins",  "defaultFileExtensions": [".dll"],    "trackingMethod": "subdir", "subRoutes": [], "isDefaultLocation": true },
  { "route": "BepInEx/core",     "defaultFileExtensions": [],          "trackingMethod": "subdir", "subRoutes": [], "isDefaultLocation": false },
  { "route": "BepInEx/patchers", "defaultFileExtensions": [],          "trackingMethod": "subdir", "subRoutes": [], "isDefaultLocation": false },
  { "route": "BepInEx/monomod",  "defaultFileExtensions": [".mm.dll"], "trackingMethod": "subdir", "subRoutes": [], "isDefaultLocation": false },
  { "route": "BepInEx/config",   "defaultFileExtensions": [],          "trackingMethod": "none",   "subRoutes": [], "isDefaultLocation": false }
]
```

`trackingMethod: "subdir"` → `installSubDir` in `src/installers/InstallRulePluginInstaller.ts`:

```ts
const subDir = profile.joinToProfilePath(rule.route, mod.getName());
```

where `mod.getName()` is the Thunderstore **full name** (`<Author>-<Mod>`, set from `combo.getMod().getFullName()` in `ManifestV2.ts:60`). Hence `<profile>/BepInEx/plugins/<Author>-<Mod>/`. `trackingMethod: "none"` (config) → `installUntracked`, which flattens straight into `<profile>/BepInEx/config/` with no per-mod subfolder.

Default: any loose `.dll` in the zip lands in `BepInEx/plugins` (`isDefaultLocation: true`).

### The BepInExPack hoist

`src/installers/BepInExInstaller.ts` special-cases the loader pack:

```ts
const basePackageFiles = ["manifest.json", "readme.md", "icon.png"];
...
const mapping = MODLOADER_PACKAGES.find((entry) => entry.packageName.toLowerCase() == mod.getName().toLowerCase());
const mappingRoot = mapping ? mapping.rootFolder : "";

let bepInExRoot: string;
if (mappingRoot.trim().length > 0) {
    bepInExRoot = path.join(packagePath, mappingRoot);
} else {
    bepInExRoot = path.join(packagePath);
}
for (const item of (await FsProvider.instance.readdir(bepInExRoot))) {
    if (!basePackageFiles.includes(item.toLowerCase())) {
        if ((await FsProvider.instance.stat(path.join(bepInExRoot, item))).isFile()) {
            await FsProvider.instance.copyFile(path.join(bepInExRoot, item), profile.joinToProfilePath(item));
        } else {
            await FsProvider.instance.copyFolder(path.join(bepInExRoot, item), profile.joinToProfilePath(item));
        }
    }
}
```

`rootFolder` comes from `ecosystem.json → modloaderPackages`; for the pack Void Crew uses:

```json
{"packageId": "BepInEx-BepInExPack", "rootFolder": "BepInExPack", "loader": "bepinex"}
```

So the zip's `BepInExPack/` subtree is hoisted to the **profile root**, minus `manifest.json` / `readme.md` / `icon.png`. Verified against the real artifact — `BepInEx-BepInExPack-5.4.2305.zip` contains exactly:

```
BepInExPack/
BepInExPack/.doorstop_version
BepInExPack/doorstop_config.ini
BepInExPack/winhttp.dll
BepInExPack/BepInEx/config/BepInEx.cfg
BepInExPack/BepInEx/core/0Harmony.dll
BepInExPack/BepInEx/core/0Harmony20.dll
BepInExPack/BepInEx/core/BepInEx.dll
BepInExPack/BepInEx/core/BepInEx.Harmony.dll
BepInExPack/BepInEx/core/BepInEx.Preloader.dll
BepInExPack/BepInEx/core/HarmonyXInterop.dll
BepInExPack/BepInEx/core/Mono.Cecil.dll
BepInExPack/BepInEx/core/Mono.Cecil.Mdb.dll
BepInExPack/BepInEx/core/Mono.Cecil.Pdb.dll
BepInExPack/BepInEx/core/Mono.Cecil.Rocks.dll
BepInExPack/BepInEx/core/MonoMod.RuntimeDetour.dll
BepInExPack/BepInEx/core/MonoMod.Utils.dll
(+ .xml docs, icon.png, manifest.json, README.md)
```

Its `doorstop_config.ini`:

```ini
[General]
enabled = true
target_assembly=BepInEx\core\BepInEx.Preloader.dll
redirect_output_log = false
boot_config_override =
ignore_disable_switch = false
[UnityMono]
dll_search_path_override =
debug_enabled = false
debug_address = 127.0.0.1:10000
debug_suspend = false
```

### ⭐ What must be copied into the game folder — `ModLinker`

This is the crux the question asks about. `src/r2mm/manager/ModLinker.ts`:

```ts
public static async link(profile: ImmutableProfile, game: Game): Promise<string[] | R2Error> {
    if ([PackageLoader.BEPINEX, PackageLoader.BEPISLOADER].includes(game.packageLoader)) {
        if (['linux', 'darwin'].includes(appWindow.getPlatform())) {
            const settings = await ManagerSettings.getSingleton(game);
            const launchType = await getDeterminedLaunchType(game, settings.getLaunchType() || LaunchType.AUTO);
            if (launchType === LaunchType.NATIVE) {
                // Game is native, BepInEx doesn't require moving. No linked files.
                return [];
            }
        }
    }
    ...
    return this.performLink(profile, game, gameDirectory);
}
```

and `performLink` walks the **profile root**:

- **Every plain file at the profile root** is copied to the game dir (`getRootFilesDestination` returns `installDirectory` for BepInEx games), **except `mods.yml`**:
  ```ts
  const lowercased = filename.toLowerCase();
  if (lowercased == "mods.yml") {
      return null;
  }
  ```
  → for BepInEx that is `winhttp.dll`, `doorstop_config.ini`, `.doorstop_version`, `changelog.txt`.
- **Directories at the profile root are copied recursively too, UNLESS excluded:**
  ```ts
  const exclusionsList = [
      "bepinex", "bepinex_server", "mods",
      "melonloader", "plugins", "userdata",
      "_state", "userlibs", "qmods", "shimloader",
      "returnofmodding", "gdweave", "renderer", "umm",
      "rivet"
  ];
  ```
  `bepinex` is excluded → **the `BepInEx/` tree is never copied into the game folder.** That is the isolation.
- Copies are skipped when size + mtime match (`isFileIdentical`), and mtime is explicitly preserved (`fs.setModifiedTime`).

**Summary of the game-folder vs profile-folder split for Void Crew on Windows/Proton:**

| Must be in the **game folder** | Lives in the **profile folder** |
| --- | --- |
| `winhttp.dll` | `BepInEx/core/*` |
| `doorstop_config.ini` | `BepInEx/plugins/<Author>-<Mod>/*` |
| `.doorstop_version` (only for tooling, harmless) | `BepInEx/config/BepInEx.cfg` |
| `changelog.txt` (cosmetic, skip it) | `BepInEx/patchers/*`, `BepInEx/cache/*`, `BepInEx/LogOutput.log` |

Only `winhttp.dll` is *strictly* required. `doorstop_config.ini` is required only if you are not passing `--doorstop-enabled true` on the command line (see §9).

---

## 7. Concrete recipe

Given:
- `GAME` = Steam install dir, e.g. `C:\Program Files (x86)\Steam\steamapps\common\Void Crew`
- `PROFILE` = a directory your app owns, e.g. `C:\Users\you\AppData\Roaming\vct-sync\profiles\default`

### One-time setup (both platforms)

1. Download `BepInEx-BepInExPack-<version>.zip` from Thunderstore (`https://thunderstore.io/package/download/BepInEx/BepInExPack/5.4.2305/`).
2. Extract; hoist the contents of `BepInExPack/` (skipping `manifest.json`, `README.md`, `icon.png`) into `PROFILE/`. You now have `PROFILE/winhttp.dll`, `PROFILE/doorstop_config.ini`, `PROFILE/.doorstop_version`, `PROFILE/BepInEx/{core,config}`.
3. Create `PROFILE/BepInEx/plugins/` and `PROFILE/BepInEx/patchers/`.
4. **Copy `PROFILE/winhttp.dll` → `GAME/winhttp.dll`.** Unavoidable: it's a DLL-proxy injector; Windows/Wine only resolves it from the exe's directory. Also copy `doorstop_config.ini` and `.doorstop_version` if you want to match r2modman exactly (recommended — it makes ini-based fallback and vanilla launches behave predictably).
5. Install mods: extract each Thunderstore zip so loose `.dll`s land in `PROFILE/BepInEx/plugins/<Author>-<Mod>/`, and any `BepInEx/config`, `BepInEx/patchers`, `BepInEx/core` subtrees in the zip go to the corresponding `PROFILE/BepInEx/<route>/`.

### Windows launch

Via Steam (what TMM does — gets Steam auth, overlay, playtime):

```
"<STEAMDIR>\Steam.exe" -applaunch 1063420 "--doorstop-enabled" "true" "--doorstop-target-assembly" "<PROFILE>\BepInEx\core\BepInEx.Preloader.dll"
```

Direct (simpler to control and to observe exit codes; Steam must be running for the game's own Steamworks init):

```
cwd = <GAME>
exec "<GAME>\Void Crew.exe" "--doorstop-enabled" "true" "--doorstop-target-assembly" "<PROFILE>\BepInEx\core\BepInEx.Preloader.dll"
```

Vanilla launch: pass `"--doorstop-enabled" "false"` (note: r2modman's `--doorstop-enable false` is the v3 name and is a no-op on Doorstop 4).

Rust sketch:

```rust
use std::process::Command;

let preloader = profile_dir.join("BepInEx").join("core").join("BepInEx.Preloader.dll");

Command::new(steam_dir.join("Steam.exe"))
    .arg("-applaunch").arg("1063420")
    .arg("--doorstop-enabled").arg("true")
    .arg("--doorstop-target-assembly").arg(&preloader)
    .spawn()?;
```

(`std::process::Command` quotes each arg itself on Windows; do not pre-wrap them in `"` the way r2modman's string-concatenating `exec()` has to.)

### Linux / Proton launch

Void Crew ships **no native Linux build**, so this is always Proton. Use the **Windows** BepInEx pack; never `run_bepinex.sh`.

1. Do the same file copies as above (into the game dir on the Linux filesystem).
2. Make Wine prefer the game-local `winhttp.dll`. Either:
   - set `WINEDLLOVERRIDES=winhttp=n,b` in the environment of the Steam launch, **or**
   - edit `<STEAM_LIBRARY>/steamapps/compatdata/1063420/pfx/user.reg`, adding under `[Software\\Wine\\DllOverrides]`:
     ```
     "winhttp"="native,builtin"
     ```
     (back the file up first — r2modman writes `user.reg.bak`).
   The BepInEx docs' documented launch-option form is `WINEDLLOVERRIDES="winhttp.dll=n,b" %command%`; r2modman's UI hands out `WINEDLLOVERRIDES="winhttp,version=n,b"`.
3. Launch with the Doorstop args, prefixing Linux paths with `Z:` so Wine can resolve them:

```sh
WINEDLLOVERRIDES="winhttp=n,b" \
  "$STEAMDIR/steam.sh" -applaunch 1063420 \
  "--doorstop-enabled" "true" \
  "--doorstop-target-assembly" "Z:$PROFILE/BepInEx/core/BepInEx.Preloader.dll"
```

Because `WINEDLLOVERRIDES` must reach the Proton process and `steam.sh` does not reliably forward the caller's environment, the robust route — and the one r2modman actually depends on — is to put it in the game's **Steam launch options** once:

```
WINEDLLOVERRIDES="winhttp=n,b" %command%
```

and then pass only the Doorstop args on the `-applaunch` line. Setting the `user.reg` override instead makes launch options unnecessary.

`Z:` maps to `/` in the default Proton prefix, so `Z:/home/you/...` is the Wine view of `/home/you/...`. Forward slashes are fine.

---

## 8. Void Crew specifics

| Fact | Value | Source |
| --- | --- | --- |
| Steam AppID | **1063420** | `ecosystem.json` `games["void-crew"].distributions[0] = {"platform":"steam","identifier":"1063420"}`; [store page](https://store.steampowered.com/app/1063420/Void_Crew/) |
| Executable | **`Void Crew.exe`** | `ecosystem.json` `exeNames: ["Void Crew.exe"]` |
| Data folder | `VoidCrew_Data` | `ecosystem.json` `dataFolderName` |
| Steam folder name | `Void Crew` | `ecosystem.json` `steamFolderName` |
| r2modman internal folder | `VoidCrew` | `ecosystem.json` `internalFolderName` |
| Loader | `bepinex` | `ecosystem.json` `packageLoader: "bepinex"` |
| Platforms | **Windows only** (System Requirements list Windows 10 64-bit; no macOS or SteamOS+Linux tab) | [store page](https://store.steampowered.com/app/1063420/Void_Crew/) |
| Runtime | **Unity Mono** | Inference from the pack the community depends on — see below |
| Community BepInEx pack | **`BepInEx-BepInExPack`**, currently `5.4.2305` = BepInEx **5.4.23.5** | [Thunderstore package page](https://thunderstore.io/c/void-crew/p/BepInEx/BepInExPack/); [Void Crew API](https://thunderstore.io/c/void-crew/api/v1/package/) |
| Doorstop shipped in that pack | **4.5.0** | `BepInExPack/.doorstop_version` inside the downloaded zip |

Mono vs IL2CPP: I did not have a Void Crew install to inspect for `GameAssembly.dll`, so this is **inferred, not directly verified**. The inference is strong: the pack every Void Crew mod depends on is `BepInEx-BepInExPack`, whose own description is *"BepInEx pack for Mono Unity games. Preconfigured and ready to use."* — an IL2CPP game would require `BepInEx-BepInExPack_IL2CPP` or a BepInEx 6 pack, and neither appears in the Void Crew dependency graph. The pack ships `BepInEx.Preloader.dll` (the Mono preloader), not `BepInEx.Unity.IL2CPP.dll`.

Dependency strings from actual Void Crew mods (Thunderstore API v1):

```
Jack_Modding-Quickstart 1.0.1          -> ["BepInEx-BepInExPack-5.4.2305","NihilityShift-VoidManager-1.2.11"]
Jack_Modding-Void_Tunnel_Decay 0.9.2   -> ["BepInEx-BepInExPack-5.4.2305","NihilityShift-VoidManager-1.2.11"]
Jack_Modding-Insanity_Mutator 1.0.2    -> ["BepInEx-BepInExPack-5.4.2305","NihilityShift-VoidManager-1.2.11"]
NihilityShift-Block_Game_Invites 1.0.0 -> ["BepInEx-BepInExPack-5.4.2304","NihilityShift-VoidManager-1.2.11"]
```

Practical note: `NihilityShift-VoidManager` is a near-universal second dependency for Void Crew — your installer should resolve the dependency graph, not just install BepInEx.

**Unconfirmed:** SteamDB was not reachable as a cross-check for the AppID; the store page plus r2modman's ecosystem data agree, which I consider sufficient.

---

## 9. Gotchas and open questions

**Doorstop is disabled by default when there is no ini.** `init_config_defaults()` sets `config.enabled = FALSE`, and `init_config_file()` returns early if `doorstop_config.ini` is missing. So: ini present → `enabled` defaults to `true` (the ini lookup's default string is `"true"`); ini absent → `false` unless you pass `--doorstop-enabled true`. **Always pass the arg explicitly** rather than relying on either.

**Windows Doorstop ignores `DOORSTOP_*` config env vars.** Repeated because it's the single most likely thing to burn a launcher author. If you set `DOORSTOP_TARGET_ASSEMBLY` on Windows and it "doesn't work", that's why. Args or ini only.

**Doorstop sets `DOORSTOP_DISABLE=TRUE` for child processes.** `src/windows/entrypoint.c`:

```c
        LOG("Hooks installed, marking DOORSTOP_DISALBE = TRUE");
        setenv(TEXT("DOORSTOP_DISABLE"), TEXT("TRUE"), TRUE);
```
and on unload:
```c
    if (reasonForDllLoad == DLL_PROCESS_DETACH)
        SetEnvironmentVariableW(L"DOORSTOP_DISABLE", NULL);
```
This is deliberate: it stops Doorstop re-injecting into subprocesses the game spawns. But if the game **relaunches itself** (some Unity titles re-exec for a 32/64-bit switch, or a launcher exe hands off to the real exe), the second process inherits `DOORSTOP_DISABLE` and **BepInEx will not load**. The escape hatch is `ignore_disable_switch=true` in the ini — note there is **no CLI argument for it**, so this specific case forces you to write the ini. The config comment says *"USE THIS ONLY WHEN ASKED TO OR YOU KNOW WHAT THIS MEANS."* Doorstop 3 additionally bailed on `DOORSTOP_INITIALIZED` being set (`Proxy/main.c:47`). **Unconfirmed:** whether Void Crew re-execs itself.

**`winhttp.dll` vs `version.dll`.** Doorstop 4 generates proxies from `src/windows/proxy/proxylist.txt`, which covers `version.dll` exports (`GetFileVersionInfo*`, `VerQueryValue*`, …), `winhttp.dll` exports (`WinHttp*`), and DXGI (`CreateDXGIFactory*`). BepInEx 5's `build.cake` ships **only** `winhttp.dll` (`PackageBepin("win", "x64", "winhttp.dll", "doorstop_config.ini");`). Use `winhttp` unless something else in the game already proxies it; `version.dll` is the classic alternative but you'd have to build it yourself. r2modman hardcodes `winhttp` for BepInEx (`winmm` only for GDWeave).

**Proton and DLL overrides.** Under Wine, the game-local `winhttp.dll` is *not* preferred by default. Per [BepInEx docs](https://docs.bepinex.dev/articles/advanced/proton_wine.html): *"UnityDoorstop relies on dll files inside the game directory being loaded instead of system dlls"*, which doesn't happen automatically under Proton/Wine. You must set the override via `WINEDLLOVERRIDES` or `winecfg`/`user.reg`. Symptom of forgetting: game launches perfectly, zero mods, no BepInEx console, no `LogOutput.log`.

**Proton paths.** Doorstop running under Wine sees Windows-style paths. A bare `/home/you/...` will fail; use `Z:/home/you/...`. r2modman does exactly this (`${isProton ? 'Z:' : ''}${preloaderPath}`).

**Steam overlay.** Launching the exe directly (`DirectGameRunner` style) means the overlay and playtime tracking may not attach, and games that call `SteamAPI_RestartAppIfNecessary` will bounce you through Steam. r2modman's default on every platform is `-applaunch` via Steam, which sidesteps this. **`steam_appid.txt` is not written by r2modman** (zero hits repo-wide) — but if you go the direct-exec route and Void Crew restarts itself through Steam, dropping a `steam_appid.txt` containing `1063420` next to the exe is the conventional workaround **[unverified — secondary source; not something r2modman does]**.

**Unity player args.** Doorstop's `--doorstop-*` args are *left in* `argv` — they are parsed but not stripped, so Unity also sees them. Unity ignores unknown args, so this is fine in practice; it's why r2modman's `--r2profile` also passes through harmlessly. Note that `redirect_output_log=true` makes Doorstop *append* `-logFile "<dir>\output_log.txt"` to the command line via a `GetCommandLineW` IAT hook (`src/windows/entrypoint.c`), which will conflict if you pass your own `-logFile`.

**Argument quoting.** Doorstop uses `CommandLineToArgv` on Windows, so normal Windows quoting rules apply. Profile paths with spaces (e.g. under `Documents`) need quoting — r2modman quotes every token unconditionally, which is a reasonable habit.

**Antivirus.** A DLL named `winhttp.dll` dropped next to a game exe, proxying a system library, is textbook DLL-hijack behaviour and is a recurring false-positive source for Defender and others. Not something you can fix from the launcher; worth a documented note for users. **[unverified — no primary source; widely reported]**

**BepInEx first run.** On first modded launch BepInEx creates `<profile>/BepInEx/config/BepInEx.cfg` (if absent), `cache/`, and `LogOutput.log`. If you see nothing appear under the profile, Doorstop didn't load or pointed elsewhere. `LogOutput.log`'s location is itself the best diagnostic for whether the profile redirect worked.

**Enabling Doorstop logging.** Doorstop must be built `-with_logging` to emit its own diagnostics; the release binaries in BepInEx packs are not. There is no runtime verbosity switch. **Unconfirmed:** whether any BepInEx-distributed `winhttp.dll` is a logging build.

**r2modman's vanilla-launch arg is wrong on Doorstop 4.** `vanillaParameterList: ['--doorstop-enable', 'false']` uses the v3 name in both the v3 and v4 code paths. Don't copy that; use `--doorstop-enabled false`.

**`proxyArgs` is dead code in `SteamGameRunner_Linux`.** As noted in §5 — don't read it as "r2modman injects WINEDLLOVERRIDES".

### Open questions I could not resolve

- Whether Void Crew's exe re-launches itself (affects `DOORSTOP_DISABLE` / `ignore_disable_switch`).
- Direct confirmation that Void Crew is Mono by inspecting the install (no `GameAssembly.dll` check performed). Inference is strong but indirect.
- The exact BepInEx 5.4.x release where Doorstop 3 → 4 happened.
- A first-party Valve document confirming `-applaunch <appid> <args>` forwards arguments to the game.
- The literal TMM data-folder root string (`Thunderstore Mod Manager/DataFolder`) — not present in the open-source repo.
- Whether Steam's `steam://run/<appid>//<args>` URL form forwards args; r2modman deliberately avoids it, using `steam://run/<appid>/` plus a `wrapper_args.txt` side-channel in its Flatpak path, which suggests the URL form is unreliable — but I could not confirm the reason.

---

## Sources

**UnityDoorstop** (repo cloned at `71c5e43`, 2026-09-01; v3 files from tag `v3.4.0.0`)
- <https://github.com/NeighTools/UnityDoorstop> — README
- <https://github.com/NeighTools/UnityDoorstop/blob/master/CHANGES.md> — v3→v4 breaking changes
- <https://github.com/NeighTools/UnityDoorstop/blob/master/src/config/config.h> — `Config` struct
- <https://github.com/NeighTools/UnityDoorstop/blob/master/src/config/common.c> — defaults
- <https://github.com/NeighTools/UnityDoorstop/blob/master/src/windows/config.c> — ini keys, CLI args, `DOORSTOP_DISABLE`, precedence
- <https://github.com/NeighTools/UnityDoorstop/blob/master/src/nix/config.c> — env vars
- <https://github.com/NeighTools/UnityDoorstop/blob/master/src/windows/entrypoint.c> — `fix_cwd`, IAT hooks, `DOORSTOP_DISABLE` propagation
- <https://github.com/NeighTools/UnityDoorstop/blob/master/src/bootstrap.c> — exported env vars
- <https://github.com/NeighTools/UnityDoorstop/blob/master/src/windows/proxy/proxylist.txt> — proxyable DLLs
- <https://github.com/NeighTools/UnityDoorstop/blob/master/assets/windows/doorstop_config.ini>
- <https://github.com/NeighTools/UnityDoorstop/blob/master/assets/nix/run.sh>
- <https://github.com/NeighTools/UnityDoorstop/tree/v3.4.0.0/docs/doorstop_config.ini>, `.../Proxy/config.c`, `.../Proxy/main.c`

**BepInEx** (`v5-lts` and `master`, cloned 2026-09-10)
- <https://github.com/BepInEx/BepInEx/blob/v5-lts/BepInEx/Paths.cs>
- <https://github.com/BepInEx/BepInEx/blob/v5-lts/BepInEx.Preloader/Entrypoint.cs>
- <https://github.com/BepInEx/BepInEx/blob/v5-lts/BepInEx.Preloader/EnvVars.cs>
- <https://github.com/BepInEx/BepInEx/blob/v5-lts/BepInEx/Bootstrap/Chainloader.cs>
- <https://github.com/BepInEx/BepInEx/blob/v5-lts/BepInEx/Logging/DiskLogListener.cs>
- <https://github.com/BepInEx/BepInEx/blob/v5-lts/build.cake> — `DOORSTOP_VER`, packaging
- <https://github.com/BepInEx/BepInEx/blob/v5-lts/doorstop/run_bepinex.sh>
- <https://github.com/BepInEx/BepInEx/blob/master/BepInEx.Core/Paths.cs>
- <https://github.com/BepInEx/BepInEx/blob/master/BepInEx.Preloader.Core/EnvVars.cs>
- <https://github.com/BepInEx/BepInEx/blob/master/Runtimes/Unity/BepInEx.Unity.Mono.Preloader/UnityPreloaderRunner.cs>
- <https://github.com/BepInEx/BepInEx/blob/master/Runtimes/Unity/BepInEx.Unity.IL2CPP/Il2CppInteropManager.cs>
- <https://docs.bepinex.dev/articles/advanced/steam_interop.html>
- <https://docs.bepinex.dev/articles/advanced/proton_wine.html>

**r2modmanPlus** (`develop` @ `b0b95371`, 2026-09-09)
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/r2mm/launching/instructions/instructions/loader/BepInExGameInstructions.ts>
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/r2mm/launching/instructions/GameInstructionParser.ts>
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/r2mm/launching/instructions/DynamicGameInstruction.ts>
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/utils/UnityDoorstopUtils.ts>
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/r2mm/launching/runners/windows/SteamGameRunner_Windows.ts>
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/r2mm/launching/runners/linux/SteamGameRunner_Linux.ts>
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/r2mm/launching/runners/multiplatform/DirectGameRunner.ts>
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/r2mm/manager/ModLinker.ts>
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/installers/BepInExInstaller.ts>
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/installers/InstallRulePluginInstaller.ts>
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/model/Profile.ts>
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/r2mm/manager/PathResolver.ts>
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/utils/LaunchUtils.ts>
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/components/computed/WrapperArguments.ts>
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/pages/Help.vue>, `.../src/pages/LinuxNativeGameSetup.vue`
- <https://github.com/ebkr/r2modmanPlus/blob/develop/public/linux_wrapper.sh>, `.../public/web_start_wrapper.sh`
- <https://github.com/ebkr/r2modmanPlus/blob/develop/src/assets/data/ecosystem.json> — Void Crew entry, `modloaderPackages`

**Thunderstore / Steam**
- <https://thunderstore.io/c/void-crew/api/v1/package/> — package list + dependency strings
- <https://thunderstore.io/c/void-crew/p/BepInEx/BepInExPack/> — pack page
- <https://thunderstore.io/package/download/BepInEx/BepInExPack/5.4.2305/> — the zip, inspected directly
- <https://store.steampowered.com/app/1063420/Void_Crew/> — AppID, Windows-only

**Secondary (flagged inline)**
- <https://www.howtogeek.com/825209/add-command-line-arguments-to-steam-gog-epic-games-store/> — `-applaunch` arg forwarding **[unverified — secondary source]**
