job "surrealdb" {
  datacenters = ["dc1"]
  namespace   = "data-plane"
  type        = "service"

  group "database" {
    volume "surrealdb_data" {
      type      = "host"
      read_only = false
      source    = "surrealdb_data"
    }

    network {
      mode = "bridge"
      port "db" {
        static = 8000
      }
    }

    task "surrealdb" {
      driver = "docker"

      config {
        image = "surrealdb/surrealdb:v2.2.1"
        ports = ["db"]
      }

      volume_mount {
        volume      = "surrealdb_data"
        destination = "/var/lib/surrealdb"
      }

      resources {
        cpu    = 500
        memory = 1024
      }
    }
  }
}
