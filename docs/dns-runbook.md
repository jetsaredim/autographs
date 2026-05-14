# DNS Runbook

This project uses OCI DNS for the app hostname while Porkbun remains the registrar
for `jetsaredim.net`.

## Delegation Model

Delegate only the app subdomain to OCI:

```text
autographs.jetsaredim.net
```

This keeps the rest of `jetsaredim.net` in Porkbun and lets Terraform manage only
the DNS zone needed by this app.

## Terraform Resources

The runtime Terraform root can create:

- OCI public DNS zone: `autographs.jetsaredim.net`
- A record: `autographs.jetsaredim.net -> runtime_public_ip`

DNS is disabled by default so IAM permissions can be applied first.

Enable it with:

```hcl
create_public_dns_zone = true
public_dns_zone_name   = "autographs.jetsaredim.net"
public_dns_record_name = "autographs.jetsaredim.net"
public_dns_record_ttl  = 300
```

The deploy workflow reads the equivalent GitHub Variables:

```text
OCI_CREATE_PUBLIC_DNS_ZONE=true
OCI_PUBLIC_DNS_ZONE_NAME=autographs.jetsaredim.net
OCI_PUBLIC_DNS_RECORD_NAME=autographs.jetsaredim.net
OCI_PUBLIC_DNS_RECORD_TTL=300
```

## Required Order

1. Apply the tenancy root so the deploy and operator policies include OCI DNS
   permissions.
2. Enable DNS in the runtime root and apply it.
3. Read the Terraform output:

```bash
terraform -chdir=infra/terraform output public_dns_nameservers
```

4. In Porkbun DNS for `jetsaredim.net`, add NS records for the subdomain
   `autographs` using the OCI nameserver hostnames from the output.
5. Verify delegation:

```bash
dig NS autographs.jetsaredim.net
dig A autographs.jetsaredim.net
curl --fail --silent http://autographs.jetsaredim.net/health
```

OCI notes that resolver caches can take 24-48 hours to fully recognize a
delegation change, although direct `dig` checks against authoritative servers can
show the configuration sooner.
