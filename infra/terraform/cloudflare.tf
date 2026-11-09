locals {
  cloudflare_configured         = var.cloudflare_mode != "off"
  cloudflare_autographs_proxied = var.cloudflare_mode == "proxied"
}

resource "cloudflare_zone" "domain" {
  count = local.cloudflare_configured ? 1 : 0

  account = {
    id = var.cloudflare_account_id
  }
  name = "jetsaredim.net"
  type = "full"
}

resource "cloudflare_dns_record" "autographs" {
  count = local.cloudflare_configured ? 1 : 0

  zone_id = cloudflare_zone.domain[0].id
  name    = "autographs.jetsaredim.net"
  type    = "A"
  content = module.compute.public_ip
  ttl     = local.cloudflare_autographs_proxied ? 1 : 600
  proxied = local.cloudflare_autographs_proxied
}

resource "cloudflare_dns_record" "mail" {
  count = local.cloudflare_configured ? 1 : 0

  zone_id  = cloudflare_zone.domain[0].id
  name     = "jetsaredim.net"
  type     = "MX"
  content  = "smtp.google.com"
  priority = 1
  ttl      = 600
  proxied  = false
}

resource "cloudflare_dns_record" "spf" {
  count = local.cloudflare_configured ? 1 : 0

  zone_id = cloudflare_zone.domain[0].id
  name    = "jetsaredim.net"
  type    = "TXT"
  content = "\"v=spf1 include:_spf.google.com ~all\""
  ttl     = 600
  proxied = false
}

resource "cloudflare_zone_setting" "strict_tls" {
  count = local.cloudflare_configured ? 1 : 0

  zone_id    = cloudflare_zone.domain[0].id
  setting_id = "ssl"
  value      = "strict"
}

resource "cloudflare_ruleset" "autographs_cache" {
  count = local.cloudflare_configured ? 1 : 0

  zone_id     = cloudflare_zone.domain[0].id
  name        = "Autographs CDN cache contract"
  description = "Bypass admin and short-lived documents; cache fingerprinted public media"
  kind        = "zone"
  phase       = "http_request_cache_settings"

  rules = [
    {
      ref         = "bypass_admin_and_api"
      enabled     = true
      description = "Bypass admin shell and private API"
      expression  = "(http.host eq \"autographs.jetsaredim.net\" and (http.request.uri.path eq \"/admin\" or starts_with(http.request.uri.path, \"/admin/\")))"
      action      = "set_cache_settings"
      action_parameters = {
        cache = false
      }
    },
    {
      ref         = "respect_rollback_documents"
      enabled     = true
      description = "Bypass edge cache for rollback-sensitive public HTML and JSON"
      expression  = "(http.host eq \"autographs.jetsaredim.net\" and (http.request.uri.path eq \"/\" or http.request.uri.path eq \"/index.html\" or http.request.uri.path eq \"/404.html\" or http.request.uri.path eq \"/manifest.json\" or starts_with(http.request.uri.path, \"/collection/\") or starts_with(http.request.uri.path, \"/items/\") or starts_with(http.request.uri.path, \"/architecture/\") or starts_with(http.request.uri.path, \"/data/\")))"
      action      = "set_cache_settings"
      action_parameters = {
        cache = false
      }
    },
    {
      ref         = "cache_fingerprinted_media_and_assets"
      enabled     = true
      description = "Cache public media and assets using origin freshness"
      expression  = "(http.host eq \"autographs.jetsaredim.net\" and (starts_with(http.request.uri.path, \"/media/\") or starts_with(http.request.uri.path, \"/assets/\")))"
      action      = "set_cache_settings"
      action_parameters = {
        cache = true
        edge_ttl = {
          mode = "respect_origin"
        }
        browser_ttl = {
          mode = "respect_origin"
        }
      }
    },
  ]
}
