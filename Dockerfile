# DevKit CLI image — run the installer app without a local Python install.
# Build:  docker build -t devkit .
# Run:    docker run --rm -v devkit-data:/devkit devkit doctor
#         docker run --rm -v devkit-data:/devkit -e DEVKIT_HOME=/devkit devkit list

FROM python:3.12-slim-bookworm

LABEL org.opencontainers.image.title="DevKit" \
      org.opencontainers.image.description="CLI to install developer SDKs into a machine dev folder" \
      org.opencontainers.image.source="https://github.com/axcel-blade/DevKit"

# Installs and caches live under DEVKIT_HOME (mount a volume in production use).
ENV DEVKIT_HOME=/devkit \
    PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1

WORKDIR /app

# Copy only what the CLI needs to run (see .dockerignore).
COPY main.py VERSION pyproject.toml ./
COPY src ./src

# Optional editable install so `devkit` entrypoint exists; main.py also works via PYTHONPATH.
RUN pip install --no-cache-dir -e . \
    && mkdir -p /devkit

VOLUME ["/devkit"]

ENTRYPOINT ["python", "main.py"]
CMD ["doctor"]
