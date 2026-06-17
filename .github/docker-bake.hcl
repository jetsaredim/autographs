variable "GHCR_CONTROLLER_IMAGE_REPOSITORY" {}
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
    "${GHCR_CONTROLLER_IMAGE_REPOSITORY}:${GITHUB_SHA}",
    "${GHCR_CONTROLLER_IMAGE_REPOSITORY}:production",
    "${GHCR_CONTROLLER_IMAGE_REPOSITORY}:latest"
  ]
}
