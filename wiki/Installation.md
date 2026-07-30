# Installation

**DevKit 0.6.1**

Clone the repository, then from the root:

```bash
python main.py doctor
python main.py list
```

No pip install is required to run the application. Python 3.12+ must be on PATH.

Optional flags for selected plugins:

```bash
python main.py install flutter --channel beta
python main.py install jdk --version 17
```

See [docs/getting-started.md](../docs/getting-started.md) for full details.

Docker image:

```bash
docker compose run --rm devkit doctor
```

See [docs/docker.md](../docs/docker.md).
