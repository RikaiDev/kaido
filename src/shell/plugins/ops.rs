use crate::shell::plugin::{
    DiagnosticCommand, DiagnosticContext, Plugin, PluginResponse, ShellEvent,
};

pub struct NginxPlugin;

impl NginxPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for NginxPlugin {
    fn name(&self) -> &str {
        "nginx"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn event(&self, event: &ShellEvent) -> PluginResponse {
        match event {
            ShellEvent::ConfigEdited { file, error } => {
                let file_lower = file.to_lowercase();
                if !file_lower.contains("nginx") && !file_lower.contains("conf") {
                    return PluginResponse::default();
                }

                let mut commands = vec![DiagnosticCommand {
                    cmd: "nginx -t".to_string(),
                    purpose: "Validate nginx configuration syntax".to_string(),
                }];

                if error.as_ref().map(|e| e.contains("bind")).unwrap_or(false) {
                    commands.push(DiagnosticCommand {
                        cmd: "ss -tlnp | grep -E '80|443'".to_string(),
                        purpose: "Check if port 80/443 is already in use".to_string(),
                    });
                    commands.push(DiagnosticCommand {
                        cmd: "systemctl status nginx".to_string(),
                        purpose: "Check nginx service status".to_string(),
                    });
                }

                PluginResponse {
                    handled: true,
                    context: Some(DiagnosticContext {
                        category: "nginx".to_string(),
                        commands,
                        explanation: if let Some(e) = error {
                            if e.contains("bind") || e.contains("80") || e.contains("443") {
                                "Port conflict detected - another service is using port 80/443"
                                    .to_string()
                            } else {
                                "Nginx configuration error".to_string()
                            }
                        } else {
                            "Nginx config controls web request routing, load balancing, and reverse proxy".to_string()
                        },
                        learn: Some(
                            "Use 'nginx -t' to validate config before reloading".to_string(),
                        ),
                    }),
                    message: None,
                }
            }

            ShellEvent::ErrorOccurred { cmd, error, .. } => {
                let cmd_lower = cmd.to_lowercase();
                let error_lower = error.to_lowercase();

                if !cmd_lower.contains("nginx") && !error_lower.contains("nginx") {
                    return PluginResponse::default();
                }

                let mut commands = vec![];

                if error_lower.contains("502") || error_lower.contains("bad gateway") {
                    commands.push(DiagnosticCommand {
                        cmd: "tail -n 50 /var/log/nginx/error.log".to_string(),
                        purpose: "Check nginx error log for upstream failures".to_string(),
                    });
                    commands.push(DiagnosticCommand {
                        cmd: "systemctl status php-fpm".to_string(),
                        purpose: "Check if PHP-FPM (or upstream) is running".to_string(),
                    });
                    commands.push(DiagnosticCommand {
                        cmd: "netstat -tlnp | grep -E '9000|8000'".to_string(),
                        purpose: "Check if upstream service is listening".to_string(),
                    });

                    return PluginResponse {
                        handled: true,
                        context: Some(DiagnosticContext {
                            category: "nginx".to_string(),
                            commands,
                            explanation: "502 Bad Gateway means the upstream server (PHP-FPM, Node, etc) failed to respond".to_string(),
                            learn: Some("502 is an upstream problem, not nginx itself - check the upstream service".to_string()),
                        }),
                        message: None,
                    };
                }

                if error_lower.contains("403") {
                    commands.push(DiagnosticCommand {
                        cmd: "ls -la /var/www/html/".to_string(),
                        purpose: "Check file permissions and index file existence".to_string(),
                    });
                    commands.push(DiagnosticCommand {
                        cmd: "cat /etc/nginx/sites-enabled/*".to_string(),
                        purpose: "Check nginx site configuration".to_string(),
                    });

                    return PluginResponse {
                        handled: true,
                        context: Some(DiagnosticContext {
                            category: "nginx".to_string(),
                            commands,
                            explanation: "403 Forbidden - nginx cannot serve the requested file"
                                .to_string(),
                            learn: Some(
                                "403 usually means permission denied or missing index file"
                                    .to_string(),
                            ),
                        }),
                        message: None,
                    };
                }

                if error_lower.contains("failed") || error_lower.contains("error") {
                    commands.push(DiagnosticCommand {
                        cmd: "nginx -t".to_string(),
                        purpose: "Test nginx configuration".to_string(),
                    });
                    commands.push(DiagnosticCommand {
                        cmd: "journalctl -u nginx -n 50".to_string(),
                        purpose: "Check nginx service logs".to_string(),
                    });
                }

                PluginResponse {
                    handled: !commands.is_empty(),
                    context: if !commands.is_empty() {
                        Some(DiagnosticContext {
                            category: "nginx".to_string(),
                            commands,
                            explanation: "Nginx error detected".to_string(),
                            learn: None,
                        })
                    } else {
                        None
                    },
                    message: None,
                }
            }

            _ => PluginResponse::default(),
        }
    }
}

pub struct DockerPlugin;

impl DockerPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for DockerPlugin {
    fn name(&self) -> &str {
        "docker"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn event(&self, event: &ShellEvent) -> PluginResponse {
        match event {
            ShellEvent::ConfigEdited { file, error: _ } => {
                let file_lower = file.to_lowercase();
                if !file_lower.contains("docker")
                    && !file_lower.contains("dockerfile")
                    && !file_lower.contains("docker-compose")
                    && !file_lower.contains("compose")
                {
                    return PluginResponse::default();
                }

                let mut commands = vec![DiagnosticCommand {
                    cmd: "docker build --dry-run .".to_string(),
                    purpose: "Validate Dockerfile syntax".to_string(),
                }];

                if file_lower.contains("compose") {
                    commands.push(DiagnosticCommand {
                        cmd: "docker compose config".to_string(),
                        purpose: "Validate docker-compose.yml".to_string(),
                    });
                }

                PluginResponse {
                    handled: true,
                    context: Some(DiagnosticContext {
                        category: "docker".to_string(),
                        commands,
                        explanation: "Docker configuration defines container build and runtime"
                            .to_string(),
                        learn: None,
                    }),
                    message: None,
                }
            }

            ShellEvent::ErrorOccurred { cmd, error, .. } => {
                let cmd_lower = cmd.to_lowercase();
                let error_lower = error.to_lowercase();

                if !cmd_lower.contains("docker") && !error_lower.contains("docker") {
                    return PluginResponse::default();
                }

                let mut commands = vec![];

                if error_lower.contains("connection refused") {
                    commands.push(DiagnosticCommand {
                        cmd: "docker ps".to_string(),
                        purpose: "Check if Docker daemon is running".to_string(),
                    });
                    commands.push(DiagnosticCommand {
                        cmd: "docker info".to_string(),
                        purpose: "Get Docker daemon status".to_string(),
                    });

                    return PluginResponse {
                        handled: true,
                        context: Some(DiagnosticContext {
                            category: "docker".to_string(),
                            commands,
                            explanation: "Docker daemon is not running or not accessible"
                                .to_string(),
                            learn: Some(
                                "Start Docker daemon with 'dockerd' or 'systemctl start docker'"
                                    .to_string(),
                            ),
                        }),
                        message: None,
                    };
                }

                if error_lower.contains("no such container") || error_lower.contains("not found") {
                    commands.push(DiagnosticCommand {
                        cmd: "docker ps -a".to_string(),
                        purpose: "List all containers (including stopped)".to_string(),
                    });
                    commands.push(DiagnosticCommand {
                        cmd: "docker images".to_string(),
                        purpose: "List available images".to_string(),
                    });

                    return PluginResponse {
                        handled: true,
                        context: Some(DiagnosticContext {
                            category: "docker".to_string(),
                            commands,
                            explanation: "The specified container or image does not exist"
                                .to_string(),
                            learn: Some(
                                "Use 'docker ps -a' to see stopped containers too".to_string(),
                            ),
                        }),
                        message: None,
                    };
                }

                if error_lower.contains("permission denied") || error_lower.contains("denied") {
                    commands.push(DiagnosticCommand {
                        cmd: "id".to_string(),
                        purpose: "Check current user and groups".to_string(),
                    });
                    commands.push(DiagnosticCommand {
                        cmd: "ls -la /var/run/docker.sock".to_string(),
                        purpose: "Check Docker socket permissions".to_string(),
                    });

                    return PluginResponse {
                        handled: true,
                        context: Some(DiagnosticContext {
                            category: "docker".to_string(),
                            commands,
                            explanation: "Permission denied - user lacks Docker access".to_string(),
                            learn: Some(
                                "Add user to 'docker' group: sudo usermod -aG docker $USER"
                                    .to_string(),
                            ),
                        }),
                        message: None,
                    };
                }

                commands.push(DiagnosticCommand {
                    cmd: "docker logs $(docker ps -lq)".to_string(),
                    purpose: "Get logs from last container".to_string(),
                });

                PluginResponse {
                    handled: !commands.is_empty(),
                    context: if !commands.is_empty() {
                        Some(DiagnosticContext {
                            category: "docker".to_string(),
                            commands,
                            explanation: "Docker error detected".to_string(),
                            learn: None,
                        })
                    } else {
                        None
                    },
                    message: None,
                }
            }

            _ => PluginResponse::default(),
        }
    }
}

impl Default for PluginResponse {
    fn default() -> Self {
        Self {
            handled: false,
            context: None,
            message: None,
        }
    }
}
