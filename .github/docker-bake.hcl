variable "GHCR_IMAGE_REPOSITORY" {}
variable "GITHUB_SHA" {}

group "default" {
  targets = ["controller"]
}

target "controller" {
  context = "."
  dockerfile = "controller/Containerfile"

  cache-from = [
    "type=gha,scope=controller-image"
  ]

  cache-to = [
    "type=gha,scope=controller-image,mode=max"
  ]

  tags = [
    "${GHCR_IMAGE_REPOSITORY}-controller:${GITHUB_SHA}",
    "${GHCR_IMAGE_REPOSITORY}-controller:production",
    "${GHCR_IMAGE_REPOSITORY}-controller:latest"
  ]
}
