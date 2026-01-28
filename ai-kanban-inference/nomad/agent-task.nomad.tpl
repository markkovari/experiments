# Template for isolated agent task execution
# This file is used by Claude-Flow to spawn agent tasks as Nomad batch jobs
#
# Variables (replaced at runtime):
#   ${TASK_ID}          - Unique task identifier
#   ${AGENT_TYPE}       - Type of agent (coder, reviewer, architect, etc.)
#   ${TASK_DESCRIPTION} - Description of the task to perform
#   ${WORKSPACE_PATH}   - Optional: specific workspace path
#   ${MODEL_NAME}       - Optional: specific model to use
#   ${TIMEOUT}          - Optional: task timeout (default: 30m)

job "agent-${TASK_ID}" {
  datacenters = ["home"]
  type        = "batch"

  # Run agents on data-plane (MacBook with GPU/Ollama)
  constraint {
    attribute = "${meta.role}"
    value     = "data-plane"
  }

  # Prefer nodes with more memory
  affinity {
    attribute = "${meta.memory}"
    value     = "96gb"
    weight    = 100
  }

  # Metadata for tracking
  meta {
    task_id    = "${TASK_ID}"
    agent_type = "${AGENT_TYPE}"
    created_at = "${NOMAD_JOB_SUBMITTED_AT}"
  }

  group "agent" {
    count = 1

    # Ephemeral disk for workspace isolation
    ephemeral_disk {
      size    = 2000   # 2GB workspace
      migrate = false  # Don't migrate on reschedule
      sticky  = false  # Fresh workspace each time
    }

    # Don't reschedule failed tasks by default
    reschedule {
      attempts  = 1
      unlimited = false
    }

    # Restart policy for transient failures
    restart {
      attempts = 2
      interval = "5m"
      delay    = "15s"
      mode     = "fail"
    }

    task "execute" {
      driver = "raw_exec"

      config {
        command = "/usr/local/bin/npx"
        args    = [
          "claude-flow@alpha",
          "run",
          "--agent", "${AGENT_TYPE}",
          "--task", "${TASK_DESCRIPTION}",
          "--workspace", "${NOMAD_ALLOC_DIR}/workspace",
          "--output", "${NOMAD_ALLOC_DIR}/output",
          "--json"
        ]
      }

      env {
        # Ollama endpoint (local on data-plane)
        OLLAMA_HOST = "http://localhost:11434"

        # Task identification
        TASK_ID     = "${TASK_ID}"
        AGENT_TYPE  = "${AGENT_TYPE}"

        # Model selection (can be overridden)
        MODEL_NAME  = "${MODEL_NAME:-codellama:34b}"

        # Workspace paths
        WORKSPACE   = "${NOMAD_ALLOC_DIR}/workspace"
        OUTPUT_DIR  = "${NOMAD_ALLOC_DIR}/output"

        # Node.js settings
        NODE_ENV    = "production"
        NODE_OPTIONS = "--max-old-space-size=4096"

        # Timeouts
        TASK_TIMEOUT = "${TIMEOUT:-30m}"
      }

      # Resource allocation per agent task
      resources {
        cpu    = 4000   # 4 CPU cores
        memory = 8192   # 8GB RAM
      }

      # Task timeout - auto-kill runaway processes
      kill_timeout = "60s"

      # Logging
      logs {
        max_files     = 3
        max_file_size = 10
      }

      # Artifact collection (optional)
      # template {
      #   data = <<EOF
      # Task completed at: {{ timestamp }}
      # Agent: ${AGENT_TYPE}
      # Task ID: ${TASK_ID}
      # EOF
      #   destination = "${NOMAD_ALLOC_DIR}/output/metadata.txt"
      # }
    }
  }
}
