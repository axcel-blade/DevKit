# Getting started

## Requirements

- Python **3.12+**
- Network access for SDK downloads
- Write access to the `dev` install root (or set `DEVKIT_HOME`)

## Run DevKit

From the repository root:

```bash
python main.py --version
python main.py doctor
python main.py list
```

Windows shortcut:

```bat
devkit.bat doctor
```

macOS / Linux:

```bash
chmod +x devkit.sh
./devkit.sh doctor
```

## Install a tool

```bash
python main.py install jdk
python main.py install flutter
python main.py status jdk
```

After install, **open a new terminal** so PATH and env vars reload.

### Environment backends

| OS | How env is stored |
|----|-------------------|
| Windows | Current-user registry |
| macOS | `~/.devkit/env.sh` sourced from `~/.zshrc` |
| Linux | `~/.devkit/env.sh` sourced from `~/.bashrc` |

## Custom install root

```bash
# Windows PowerShell
$env:DEVKIT_HOME = "D:\dev"
python main.py install hello

# Unix
export DEVKIT_HOME="$HOME/my-dev"
python main.py install hello
```

## Uninstall

```bash
python main.py uninstall hello
```

Removes files under the `dev` folder and reverses PATH/env entries DevKit added.
