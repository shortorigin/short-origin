job "wasmcloud-host" {
  datacenters = ["dc1"]
  namespace   = "runtime"
  type        = "service"

  group "host" {
    network {
      mode = "bridge"
      port "nats" {
        static = 4222
      }
    }

    task "host" {
      driver = "docker"

      config {
        image = "ghcr.io/wasmcloud/wasmcloud:1.2.1"
      }

      env {
        WASMCLOUD_CTL_HOST                   = "nats"
        WASMCLOUD_CTL_PORT                   = "4222"
        WASMCLOUD_JS_DOMAIN                  = "institutional"
        WASMCLOUD_LATTICE                    = "institutional-lattice"
        WASMCLOUD_POLICY_TOPIC               = "policy.runtime"
        WASMCLOUD_RPC_HOST                   = "nats"
        WASMCLOUD_RPC_PORT                   = "4222"
        WASMCLOUD_STRUCTURED_LOGGING_ENABLED = "true"
      }

      resources {
        cpu    = 500
        memory = 512
      }
    }
  }
}
