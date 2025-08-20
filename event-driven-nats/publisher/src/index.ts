import { connect, StringCodec } from 'nats';

const sc = StringCodec();

async function publisher() {
  const nc = await connect({ servers: 'nats://localhost:4222' });

  setInterval(() => {
    const event = {
      id: `user_${Date.now()}`,
      name: 'John Doe',
      email: `john.doe.${Date.now()}@example.com`,
    };
    nc.publish('UserCreated', sc.encode(JSON.stringify(event)));
    console.log(`Published event: ${JSON.stringify(event)}`);
  }, 2000);
}

publisher().catch(console.error);
