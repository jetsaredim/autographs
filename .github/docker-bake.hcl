variable "GHCR_CONTROLLER_IMAGE_REPOSITORY" {}
variable "RELEASE_VERSION" {}
variable "SOURCE_REVISION" {}

group "default" {
  targets = ["controller"]
}

target "controller" {
  context = "."
  dockerfile = "controller/Dockerfile"

  cache-from = [
    "type=gha,scope=controller-image"
  ]

  cache-to = [
    "type=gha,scope=controller-image,mode=max"
  ]

  tags = [
    "${GHCR_CONTROLLER_IMAGE_REPOSITORY}:${RELEASE_VERSION}",
    "${GHCR_CONTROLLER_IMAGE_REPOSITORY}:production",
    "${GHCR_CONTROLLER_IMAGE_REPOSITORY}:latest"
  ]

  labels = {
    "org.opencontainers.image.source" = "https://github.com/jetsaredim/autographs"
    "org.opencontainers.image.revision" = "${SOURCE_REVISION}"
    "org.opencontainers.image.version" = "${RELEASE_VERSION}"
  }
}
