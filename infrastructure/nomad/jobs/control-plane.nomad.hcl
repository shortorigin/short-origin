job "institutional-control-plane" {
  datacenters = ["dc1"]
  namespace   = "control-plane"
  type        = "service"

  group "messaging" {
    network {
      mode = "bridge"

      port "client" {
        static = 4222
      }

      port "monitor" {
        static = 8222
      }
    }

    task "nats" {
      driver = "docker"

      config {
        image = "nats:2.10-alpine"
        args = [
          "--jetstream",
          "--http_port",
          "${NOMAD_PORT_monitor}",
          "--port",
          "${NOMAD_PORT_client}",
        ]
        ports = ["client", "monitor"]
      }

      service {
        name = "nats"
        port = "client"
      }

      resources {
        cpu    = 300
        memory = 256
      }
    }

    task "wadm" {
      driver = "docker"

      config {
        image = "ghcr.io/wasmcloud/wadm:0.20.2"
      }

      env {
        RUST_LOG              = "info"
        WADM_NATS_SERVER      = "nats://127.0.0.1:${NOMAD_PORT_client}"
        WADM_LATTICE          = "institutional-lattice"
        WADM_JS_DOMAIN        = "institutional"
        WADM_ALLOW_FILE_LOADS = "false"
      }

      resources {
        cpu    = 300
        memory = 256
      }
    }
  }
}
