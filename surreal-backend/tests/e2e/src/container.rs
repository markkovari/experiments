use testcontainers::{core::{ContainerPort, WaitFor}, Container, GenericImage, ImageExt, Client};

pub struct SurrealContainer {
    _container: Container<GenericImage>,
    pub url: String,
}

impl SurrealContainer {
    pub fn new(docker: &Client) -> Self {
        let image = GenericImage::new("surrealdb/surrealdb", "latest")
            .with_exposed_port(ContainerPort::Tcp(8000))
            .with_wait_for(WaitFor::message_on_stdout("Started web server"))
            .with_cmd(vec!["start", "--user", "root", "--pass", "root", "memory"]);

        let container = docker.run(image);
        let port = container.get_host_port_ipv4(8000).unwrap();
        let url = format!("http://127.0.0.1:{}", port);

        Self {
            _container: container,
            url,
        }
    }
}
