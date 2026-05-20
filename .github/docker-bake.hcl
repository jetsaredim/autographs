variable "GHCR_IMAGE_REPOSITORY" {}
variable "GITHUB_SHA" {}

group "default" {
  targets = ["app", "tools"]
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

target "app" {
  inherits = ["common"]
  target = "runner"
  tags = [
    "${GHCR_IMAGE_REPOSITORY}:${GITHUB_SHA}",
    "${GHCR_IMAGE_REPOSITORY}:production",
    "${GHCR_IMAGE_REPOSITORY}:latest"
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
