import { NextApiRequest, NextApiResponse } from 'next';

export default function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method === 'GET') {
        res.status(200).json({ message: 'Hello, world!' });
    } else if (req.method === 'POST') {
        const { name } = req.body; // TypeScript will infer the type of `name` as `any` by default
        res.status(200).json({ message: `Hello, ${name}!` });
    } else {
        res.status(405).json({ message: 'Method not allowed' });
    }
}