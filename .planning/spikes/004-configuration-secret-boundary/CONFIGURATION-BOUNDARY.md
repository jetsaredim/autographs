# Proposed Production Configuration Boundary

This is a spike recommendation, not the implemented runtime contract.

## Durable Non-Secret Configuration

Keep one Ansible-managed `/opt/autographs/env/controller.env` containing only:

- Controller image/repository/source versions
- Domain, public origin, bind address, HTTP/HTTPS ports, and secure-cookie mode
- Database/media provider selections
- Oracle user, connect string, wallet runtime directory, and heartbeat interval
- OCI region, realm domain, media namespace, media bucket, and
  `OCI_AUTH_MODE=instance_principal`
- Static release root and release-retention counts
- Secret OCIDs used to retrieve the current Vault versions

Sensitive coordinates such as connect strings, user names, and secret OCIDs are
not authentication secrets, but the file should remain mode `0600` because they
increase reconnaissance value.

## OCI Vault Secrets

| Proposed reference | Secret content | Runtime handling |
|--------------------|----------------|------------------|
| `AUTOGRAPHS_ORACLE_DB_PASSWORD_SECRET_OCID` | Oracle application-user password | Fetch once at startup; retain in process memory |
| `AUTOGRAPHS_ORACLE_WALLET_PASSWORD_SECRET_OCID` | PEM wallet password | Fetch once at startup; retain in process memory |
| `AUTOGRAPHS_ORACLE_WALLET_PEM_SECRET_OCID` | Base64 or text `ewallet.pem` | Fetch at startup; materialize mode `0400` in container tmpfs |
| `AUTOGRAPHS_ORACLE_TNSNAMES_SECRET_OCID` | `tnsnames.ora` when an alias is used | Fetch at startup; materialize mode `0400` in container tmpfs |
| `AUTOGRAPHS_ADMIN_PASSWORD_HASH_SECRET_OCID` | Argon2 admin password hash | Fetch once at startup; retain in process memory |
| `AUTOGRAPHS_OPERATOR_TOKEN_SECRET_OCID` | Compatibility diagnostic token | Only if retained; fetch once at startup |

`sqlnet.ora` may remain committed deployment configuration if it contains no
credential material. Measure every wallet component against OCI Vault's 25 KB
secret-bundle limit before implementation.

## Remove From Production Runtime

- `OCI_TENANCY_OCID`
- `OCI_CLI_USER_OCID`
- `OCI_FINGERPRINT`
- `OCI_PRIVATE_KEY_PATH`
- `/opt/autographs/secrets/oci_api_key.pem`
- `AUTOGRAPHS_ADMIN_PASSWORD`
- Persistent scalar values replaced by the Vault reference variables above
- Encoded and decoded wallet archives after required runtime files are
  materialized

These OCI API-key fields remain valid in the GitHub/Terraform deployment
boundary. They are removed only from the runtime controller boundary.

## Keep Separate

- GitHub deployment secrets remain GitHub secrets because the GitHub runner is
  a different principal from the runtime VM.
- Local development and live-smoke env files remain operator-managed,
  short-lived exceptions. They must be mode `0600`, excluded from source
  control, purpose-named, and removed when their test campaign ends.
- Production runtime and smoke-test configuration must not share a catch-all env
  file. A smoke file may reference the same non-secret coordinates, but its
  temporary secret values are not a production source of truth.

## Fail-Closed Contract

- A configured secret OCID with an unavailable, malformed, empty, or
  unauthorized secret prevents controller readiness.
- No fallback reads the retired plaintext environment variable.
- Secret contents never appear in logs, health responses, errors, generated
  static output, or inventory reports.
- Initial rotation takes effect on controlled controller restart. Hot reload is
  deferred until a demonstrated operational need justifies it.
- Running containers retain their already-loaded scalar values through a Vault
  outage; a fresh start requires Vault availability and valid IAM.
