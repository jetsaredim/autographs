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

## TLS

The deployed edge container uses Caddy, which automatically obtains and renews a
Let's Encrypt certificate for `autographs.jetsaredim.net`. Certificate issuance
requires all of the following before the deploy runs:

- the Porkbun `A` record resolves to the runtime VM public IP
- OCI ingress allows ports 80 and 443
- the VM firewall allows HTTP and HTTPS

OCI NSG and VM firewall rules are already managed for ports 80 and 443 by the
committed Terraform and bootstrap scripts.

## Verification

After Porkbun is updated and the deploy workflow has completed, verify DNS,
HTTP, and HTTPS:

```bash
dig A autographs.jetsaredim.net
curl --fail --silent http://autographs.jetsaredim.net/health
curl --fail --silent https://autographs.jetsaredim.net/health
```

Expected response:

```json
{"ok":true,"service":"autographs","scope":"proof-of-life"}
```

DNS changes can take time to propagate through recursive resolver caches, but a
low TTL such as 300 seconds should make routine updates settle quickly.
