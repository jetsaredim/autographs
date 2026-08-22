#!/usr/bin/env python3
"""Compare secret exposure for candidate Autographs runtime delivery models."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import stat
import tempfile
from typing import Any


FAKE_SECRET = "probe-value-that-must-never-appear-in-output"
SCALAR_NAMES = (
    "ORACLE_DB_PASSWORD",
    "ORACLE_DB_WALLET_PASSWORD",
    "AUTOGRAPHS_ADMIN_PASSWORD_HASH",
)
PUBLIC_CONFIG = {
    "AUTOGRAPHS_CONTROLLER_DB_PROVIDER": "oracle",
    "OCI_AUTH_MODE": "instance_principal",
    "OCI_REGION": "us-ashburn-1",
}


def contains_secret(root: Path) -> list[str]:
    exposed = []
    if not root.exists():
        return exposed
    for path in root.rglob("*"):
        if path.is_file() and FAKE_SECRET in path.read_text(encoding="utf-8"):
            exposed.append(path.relative_to(root).as_posix())
    return exposed


def write_private(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    path.chmod(0o400)


def persistent_env_model(root: Path) -> dict[str, Any]:
    persistent = root / "persistent"
    runtime = root / "runtime"
    persistent.mkdir()
    runtime.mkdir()
    env = {**PUBLIC_CONFIG, **{name: FAKE_SECRET for name in SCALAR_NAMES}}
    env_path = persistent / "app.env"
    env_path.write_text("".join(f"{name}={value}\n" for name, value in env.items()), encoding="utf-8")
    env_path.chmod(0o600)
    return summarize("persistent-env", root, process_env_secret_count=len(SCALAR_NAMES))


def startup_materialization_model(root: Path) -> dict[str, Any]:
    persistent = root / "persistent"
    runtime = root / "runtime"
    persistent.mkdir()
    runtime.mkdir()
    config = {
        **PUBLIC_CONFIG,
        **{f"{name}_SECRET_OCID": f"ocid1.vaultsecret.example.{index}" for index, name in enumerate(SCALAR_NAMES)},
    }
    (persistent / "app.env").write_text(
        "".join(f"{name}={value}\n" for name, value in config.items()), encoding="utf-8"
    )
    for name in SCALAR_NAMES:
        write_private(runtime / "secrets" / name.lower(), FAKE_SECRET)
    write_private(runtime / "wallet" / "ewallet.pem", FAKE_SECRET)
    return summarize("startup-materialization", root, process_env_secret_count=0)


def direct_application_model(root: Path) -> dict[str, Any]:
    persistent = root / "persistent"
    runtime = root / "runtime"
    persistent.mkdir()
    runtime.mkdir()
    config = {
        **PUBLIC_CONFIG,
        **{f"{name}_SECRET_OCID": f"ocid1.vaultsecret.example.{index}" for index, name in enumerate(SCALAR_NAMES)},
    }
    (persistent / "app.env").write_text(
        "".join(f"{name}={value}\n" for name, value in config.items()), encoding="utf-8"
    )
    in_memory_scalars = {name: FAKE_SECRET for name in SCALAR_NAMES}
    assert len(in_memory_scalars) == len(SCALAR_NAMES)
    write_private(runtime / "wallet" / "ewallet.pem", FAKE_SECRET)
    return summarize("direct-application", root, process_env_secret_count=0)


def summarize(model: str, root: Path, process_env_secret_count: int) -> dict[str, Any]:
    persistent_exposure = contains_secret(root / "persistent")
    runtime_exposure = contains_secret(root / "runtime")
    runtime_modes = {}
    for path in (root / "runtime").rglob("*"):
        if path.is_file():
            runtime_modes[path.relative_to(root / "runtime").as_posix()] = stat.S_IMODE(path.stat().st_mode)
    return {
        "model": model,
        "persistent_secret_file_count": len(persistent_exposure),
        "ephemeral_secret_file_count": len(runtime_exposure),
        "process_env_secret_count": process_env_secret_count,
        "runtime_file_modes": runtime_modes,
        "requires_vault_at_restart": model != "persistent-env",
        "supports_wallet_file_requirement": bool(runtime_exposure) or model == "persistent-env",
    }


def run_probe() -> dict[str, Any]:
    models = []
    for name, probe in (
        ("persistent", persistent_env_model),
        ("materialized", startup_materialization_model),
        ("direct", direct_application_model),
    ):
        with tempfile.TemporaryDirectory(prefix=f"autographs-{name}-") as directory:
            models.append(probe(Path(directory)))
    return {
        "redaction_contract": "Counts and modes only; fake and real secret values are never emitted.",
        "scope": "Application-managed persistent and runtime files plus process-environment placement.",
        "excluded_surfaces": [
            "core dumps",
            "kernel crash dumps",
            "process memory paging",
            "tmpfs paging to swap",
        ],
        "models": models,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    report = run_probe()
    encoded = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if FAKE_SECRET in encoded:
        raise RuntimeError("probe output violated its redaction contract")
    if args.output:
        args.output.write_text(encoded, encoding="utf-8")
    else:
        print(encoded, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
