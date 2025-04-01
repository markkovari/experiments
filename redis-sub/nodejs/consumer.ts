import { createClient } from 'redis';
import { topic } from './consts'

const run = async () => {
    const client = await createClient()
        .on('error', err => console.log('Redis Client Error', err))
        .connect();

    client.pSubscribe(topic, (event) => {
        console.log({ event })
    });
}


run().then(() => console.log(`consumer is running and listening on ${topic}`)).catch(console.error)
