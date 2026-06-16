variable "GHCR_IMAGE_REPOSITORY" {}
variable "GITHUB_SHA" {}

group "default" {
  targets = ["tools", "controller"]
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

target "common" {
  context = "."
  dockerfile = "app/Dockerfile"

  cache-from = [
    "type=gha,scope=app-image"
  ]

  cache-to = [
    "type=gha,scope=app-image,mode=max"
  ]
}

target "tools" {
  inherits = ["common"]
  target = "tools"
  tags = [
    "${GHCR_IMAGE_REPOSITORY}-tools:${GITHUB_SHA}",
    "${GHCR_IMAGE_REPOSITORY}-tools:production",
    "${GHCR_IMAGE_REPOSITORY}-tools:latest"
  ]
}
