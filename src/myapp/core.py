"""Core API for myapp."""


def greet(name: str = "world") -> str:
    """Return a greeting."""
    return f"Hello, {name}!"
