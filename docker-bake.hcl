variable "CHANNEL" {
  default = "dev"
}

variable "VERSION" {
  default = ""
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
        channel_tag = "${CHANNEL}-debug"
        version_tag = "${VERSION}-debug"
      },
      {
        type = "release"
        channel_tag = "${CHANNEL}"
        version_tag = "${VERSION}"
      }
    ]
  }
  args = {
    BUILD_TYPE = build_type.type
  }
  platforms = ["linux/amd64", "linux/arm64"]
  target = build_type.type
  tags = [
    notequal("", CHANNEL)
      ? "flecspublic.azurecr.io/flecs/floxy:${build_type.channel_tag}"
      : "",
    notequal("", VERSION)
      ? "flecspublic.azurecr.io/flecs/floxy:${build_type.version_tag}"
      : "",
  ]
}
