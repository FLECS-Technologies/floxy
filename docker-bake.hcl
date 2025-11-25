variable "TAG" {
  default = "dev"
}

group "default" {
  targets = ["all"]
}

group "debug" {
  targets = ["floxy-debug"]
}

group "release" {
  targets = ["floxy-release"]
}

target "all" {
  name = "floxy-${build_type.type}"
  context = "."
  dockerfile = "docker/Dockerfile"
  matrix = {
    build_type = [
      {
        type = "debug"
        tag = "${TAG}-debug"
      },
      {
        type = "release"
        tag = "${TAG}"
      }
    ]
  }
  args = {
    BUILD_TYPE = build_type.type
  }
  platforms = ["linux/amd64", "linux/arm64"]
  target = build_type.type
  tags = ["flecspublic.azurecr.io/flecs/floxy:${build_type.tag}"]
}
