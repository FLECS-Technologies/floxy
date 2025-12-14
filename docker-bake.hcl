variable "CHANNEL" {
  default = "dev"
}

variable "GIT_SHA" {
  default = ""
}

variable "VERSIONS" {
  type = list(string)
  default = []
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
        version_suffix = "-debug"
      },
      {
        type = "release"
        channel_tag = "${CHANNEL}"
        version_suffix = ""
      }
    ]
  }
  args = {
    BUILD_TYPE = build_type.type
    GIT_SHA = GIT_SHA
  }
  platforms = ["linux/amd64", "linux/arm64"]
  target = build_type.type
  tags = flatten([
    notequal("", CHANNEL)
      ? ["flecspublic.azurecr.io/flecs/floxy:${build_type.channel_tag}"]
      : [],
    [
    for V in VERSIONS :
      "flecspublic.azurecr.io/flecs/floxy:${V}${build_type.version_suffix}"
    ]
  ])
}
