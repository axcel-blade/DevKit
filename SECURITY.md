# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.4.x   | Yes |
| 0.3.x   | Security fixes only |
| 0.2.x   | Security fixes only |
| < 0.2   | No |

## Reporting a vulnerability

Please **do not** open a public issue for security problems.

1. Use GitHub **Security Advisories** on this repository, or
2. Contact the maintainer privately via their GitHub profile: [axcel-blade](https://github.com/axcel-blade).

Include:

- Description of the issue
- Steps to reproduce
- Affected version (`python main.py --version`)
- Impact assessment if known

You should receive an acknowledgment within a reasonable time. Please allow
time for investigation before any public disclosure.

## Security notes for users

- DevKit downloads third-party SDK archives (Flutter, Temurin, Mono, Composer, Git for Windows / MinGit, Gradle, PHP, MySQL, Node.js, Android cmdline-tools).
  Only use trusted networks when installing.
- Plugins may modify **user** environment variables (Windows registry or shell profiles).
- Review plugin code before running `install` on untrusted forks.
