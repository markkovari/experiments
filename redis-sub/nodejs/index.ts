import { createClient } from 'redis';

const client = createClient({
    url: "localhost:6379"
});

client.pSubscribe("", (event) => {

});