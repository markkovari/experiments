import { GenericContainer, type StartedTestContainer, Wait } from 'testcontainers';

let container: StartedTestContainer;

export async function setup() {
  container = await new GenericContainer('surrealdb/surrealdb:latest')
    .withExposedPorts(8000)
    .withCommand(['start', '--bind', '0.0.0.0:8000', '--user', 'root', '--pass', 'root', 'memory'])
    .withWaitStrategy(Wait.forHttp('/', 8000).forStatusCode(200).withStartupTimeout(30000))
    .withStartupTimeout(60000)
    .start();

  const host = container.getHost();
  const port = container.getMappedPort(8000);
  const url = `http://${host}:${port}`;

  // Store container info in environment for test files to access
  process.env.SURREAL_TEST_URL = url;

  return () => container.stop();
}
