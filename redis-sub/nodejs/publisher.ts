import { createClient } from 'redis';
import { topic, interval } from './consts'


const run = async () => {
    const client = await createClient()
        .on('error', err => console.log('Redis Client Error', err))
        .connect();

    setInterval(() => {
        client.publish(topic, "some message");
        console.log("just published...")
    }, interval)
}


run().then(() => console.log(`producer is running and publishint on ${topic} every ${interval}ms`)).catch(console.error)
