# DNS Runbook

This project uses Porkbun DNS directly for the public app hostname. OCI public
DNS zones are intentionally not managed by Terraform because OCI public DNS is
not available in the current free-tier setup.

## DNS Model

Keep `jetsaredim.net` hosted at Porkbun and create a direct `A` record for the
app hostname:

```text
autographs.jetsaredim.net -> runtime_public_ip
```

The runtime VM public IP is produced by Terraform and currently also mirrored in
the `VM_PUBLIC_IP` GitHub Variable for deploy fallback behavior.

## Porkbun Record

In Porkbun DNS for `jetsaredim.net`, create or update this record:

```text
Type: A
Host: autographs
Answer: <runtime_public_ip>
TTL: 300
```

Use the current Terraform output for the address:

```bash
terraform -chdir=infra/terraform output -raw runtime_public_ip
```

Or check the GitHub Variable fallback:

```bash
gh variable get VM_PUBLIC_IP --repo jetsaredim/autographs
```

## Verification

After Porkbun is updated, verify resolution and the app health endpoint:

```bash
dig A autographs.jetsaredim.net
curl --fail --silent http://autographs.jetsaredim.net/health
```

Expected response:

```json
{"ok":true,"service":"autographs","scope":"proof-of-life"}
```

DNS changes can take time to propagate through recursive resolver caches, but a
low TTL such as 300 seconds should make routine updates settle quickly.
