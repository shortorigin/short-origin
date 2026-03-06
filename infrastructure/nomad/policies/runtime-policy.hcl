namespace "runtime" {
  capabilities = ["read-job", "submit-job", "dispatch-job"]
}

namespace "control-plane" {
  capabilities = ["read-job", "submit-job"]
}

namespace "data-plane" {
  capabilities = ["read-job"]
}
