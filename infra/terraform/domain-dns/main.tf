resource "cloudflare_zone" "domain" {
  account = {
    id = var.cloudflare_account_id
  }
  name = var.domain_name
  type = "full"
}

resource "cloudflare_dns_record" "domain" {
  for_each = var.dns_records

  zone_id  = cloudflare_zone.domain.id
  name     = each.value.name
  type     = each.value.type
  content  = each.value.content
  ttl      = each.value.ttl
  priority = each.value.priority
  proxied  = false
}

resource "cloudflare_zone_setting" "strict_tls" {
  zone_id    = cloudflare_zone.domain.id
  setting_id = "ssl"
  value      = "strict"
}

resource "cloudflare_ruleset" "autographs_cache" {
  zone_id     = cloudflare_zone.domain.id
  name        = "Autographs CDN cache contract"
  description = "Bypass admin and short-lived documents; cache fingerprinted public media"
  kind        = "zone"
  phase       = "http_request_cache_settings"

  rules = [
    {
      ref         = "bypass_admin_and_api"
      description = "Bypass admin shell and private API"
      expression  = "(http.host eq \"autographs.jetsaredim.net\" and (http.request.uri.path eq \"/admin\" or starts_with(http.request.uri.path, \"/admin/\")))"
      action      = "set_cache_settings"
      action_parameters = {
        cache = false
      }
    },
    {
      ref         = "respect_rollback_documents"
      description = "Bypass edge cache for rollback-sensitive public HTML and JSON"
      expression  = "(http.host eq \"autographs.jetsaredim.net\" and (http.request.uri.path eq \"/\" or http.request.uri.path eq \"/index.html\" or http.request.uri.path eq \"/404.html\" or http.request.uri.path eq \"/manifest.json\" or starts_with(http.request.uri.path, \"/collection/\") or starts_with(http.request.uri.path, \"/items/\") or starts_with(http.request.uri.path, \"/architecture/\") or starts_with(http.request.uri.path, \"/data/\")))"
      action      = "set_cache_settings"
      action_parameters = {
        cache = false
      }
    },
    {
      ref         = "cache_fingerprinted_media_and_assets"
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
